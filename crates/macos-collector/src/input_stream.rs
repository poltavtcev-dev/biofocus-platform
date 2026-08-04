//! Periodic keystroke aggregate → `keystrokes` Observation (counts/rates only).

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::CollectorResult;
use crate::input_probe::InputCountProbe;
use crate::payload::{
    keystrokes_payload, KEYSTROKES_DATA_TYPE, MACOS_INPUT_PROVIDER_ID,
};

/// Builds a privacy-safe aggregate Observation.
pub fn observation_from_aggregate(
    count: u64,
    window: Duration,
) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_INPUT_PROVIDER_ID,
        KEYSTROKES_DATA_TYPE,
        keystrokes_payload(count, window),
        1.0,
    )
    .map_err(Into::into)
}

/// Handle that stops the aggregate loop and waits for the task to finish.
pub struct KeystrokeAggregateHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl KeystrokeAggregateHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "keystroke aggregate task join failed"),
            }
        }
    }
}

impl Drop for KeystrokeAggregateHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`window` tick loop. Emits when `take_count()` > 0 for the window.
pub fn spawn_keystroke_aggregate_loop(
    tx: ObservationSender,
    probe: Arc<dyn InputCountProbe>,
    window: Duration,
) -> KeystrokeAggregateHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(window);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Skip the immediate first tick so the first window is a full interval.
        ticker.tick().await;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("keystroke aggregate collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    if !probe.accessibility_trusted() {
                        // Drain any stale counts; stay idle without emitting.
                        let _ = probe.take_count();
                        continue;
                    }
                    match probe.take_count() {
                        Ok(0) => {}
                        Ok(count) => {
                            match observation_from_aggregate(count, window) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {}
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping keystrokes"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping keystroke loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build keystrokes Observation");
                                }
                            }
                        }
                        Err(err) => {
                            warn!(error = %err, "input count probe failed");
                        }
                    }
                }
            }
        }
    });

    KeystrokeAggregateHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
