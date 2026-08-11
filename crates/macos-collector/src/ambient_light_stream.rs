//! Poll loop: emit `ambient_light` Observation when light identity changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::ambient_light_probe::{AmbientLightProbe, AmbientLightSample};
use crate::error::CollectorResult;
use crate::payload::{
    ambient_light_payload, AMBIENT_LIGHT_DATA_TYPE, MACOS_AMBIENT_LIGHT_PROVIDER_ID,
};

/// Builds an Observation for an ambient light change.
pub fn observation_from_ambient_light(sample: &AmbientLightSample) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let confidence = if sample.light_kind == bio_spec::LIGHT_KIND_UNKNOWN {
        0.5
    } else {
        1.0
    };

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_AMBIENT_LIGHT_PROVIDER_ID,
        AMBIENT_LIGHT_DATA_TYPE,
        ambient_light_payload(sample),
        confidence,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct AmbientLightHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl AmbientLightHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "ambient_light collector task join failed"),
            }
        }
    }
}

impl Drop for AmbientLightHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when light identity changes.
pub fn spawn_ambient_light_loop(
    tx: ObservationSender,
    probe: Arc<dyn AmbientLightProbe>,
    interval: Duration,
) -> AmbientLightHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("ambient_light collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.current() {
                        Ok(Some(sample)) => {
                            let key = sample.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_ambient_light(&sample) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping ambient_light"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping ambient_light loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build ambient_light Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "ambient_light probe failed");
                        }
                    }
                }
            }
        }
    });

    AmbientLightHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
