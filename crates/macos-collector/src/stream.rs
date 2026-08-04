//! Poll loop: emit `context_window` Observation when frontmost app changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::CollectorResult;
use crate::payload::{
    context_window_payload, CONTEXT_WINDOW_DATA_TYPE, MACOS_CONTEXT_PROVIDER_ID,
};
use crate::probe::{FrontmostApp, FrontmostProbe};

/// Builds an Observation for a frontmost-app change.
pub fn observation_from_frontmost(app: &FrontmostApp) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_CONTEXT_PROVIDER_ID,
        CONTEXT_WINDOW_DATA_TYPE,
        context_window_payload(app),
        1.0,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct ActiveWindowHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl ActiveWindowHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "active window collector task join failed"),
            }
        }
    }
}

impl Drop for ActiveWindowHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when frontmost identity changes.
pub fn spawn_active_window_loop(
    tx: ObservationSender,
    probe: Arc<dyn FrontmostProbe>,
    interval: Duration,
) -> ActiveWindowHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("active window collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.frontmost() {
                        Ok(Some(app)) => {
                            let key = app.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_frontmost(&app) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping context_window"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping active window loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build context_window Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "frontmost probe failed");
                        }
                    }
                }
            }
        }
    });

    ActiveWindowHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
