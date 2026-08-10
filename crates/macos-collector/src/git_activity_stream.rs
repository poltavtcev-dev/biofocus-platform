//! Poll loop: emit `git_activity` Observation when activity identity changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::CollectorResult;
use crate::git_activity_probe::{GitActivityProbe, GitActivitySample};
use crate::payload::{git_activity_payload, GIT_ACTIVITY_DATA_TYPE, MACOS_GIT_PROVIDER_ID};

/// Builds an Observation for a Git activity change.
pub fn observation_from_git_activity(sample: &GitActivitySample) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let confidence = if sample.activity_kind == bio_spec::ACTIVITY_KIND_UNKNOWN {
        0.5
    } else {
        1.0
    };

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_GIT_PROVIDER_ID,
        GIT_ACTIVITY_DATA_TYPE,
        git_activity_payload(sample),
        confidence,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct GitActivityHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl GitActivityHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "git_activity collector task join failed"),
            }
        }
    }
}

impl Drop for GitActivityHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when activity identity changes.
pub fn spawn_git_activity_loop(
    tx: ObservationSender,
    probe: Arc<dyn GitActivityProbe>,
    interval: Duration,
) -> GitActivityHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("git_activity collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.current() {
                        Ok(Some(sample)) => {
                            let key = sample.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_git_activity(&sample) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping git_activity"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping git_activity loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build git_activity Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "git_activity probe failed");
                        }
                    }
                }
            }
        }
    });

    GitActivityHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
