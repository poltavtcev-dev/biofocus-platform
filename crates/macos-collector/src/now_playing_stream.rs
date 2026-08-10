//! Poll loop: emit `now_playing` Observation when media identity changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::CollectorResult;
use crate::now_playing_probe::{NowPlayingProbe, NowPlayingSample};
use crate::payload::{now_playing_payload, MACOS_NOW_PLAYING_PROVIDER_ID, NOW_PLAYING_DATA_TYPE};

/// Builds an Observation for a Now Playing change.
pub fn observation_from_now_playing(sample: &NowPlayingSample) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let confidence = if sample.media_kind == bio_spec::MEDIA_KIND_UNKNOWN {
        0.5
    } else {
        1.0
    };

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_NOW_PLAYING_PROVIDER_ID,
        NOW_PLAYING_DATA_TYPE,
        now_playing_payload(sample),
        confidence,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct NowPlayingHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl NowPlayingHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "now_playing collector task join failed"),
            }
        }
    }
}

impl Drop for NowPlayingHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when media identity changes.
pub fn spawn_now_playing_loop(
    tx: ObservationSender,
    probe: Arc<dyn NowPlayingProbe>,
    interval: Duration,
) -> NowPlayingHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("now_playing collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.current() {
                        Ok(Some(sample)) => {
                            let key = sample.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_now_playing(&sample) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping now_playing"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping now_playing loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build now_playing Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "now_playing probe failed");
                        }
                    }
                }
            }
        }
    });

    NowPlayingHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
