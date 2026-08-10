//! Poll loop: emit `browser_category` Observation when coarse category changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::browser_probe::{BrowserCategoryProbe, BrowserCategorySample};
use crate::error::CollectorResult;
use crate::payload::{
    browser_category_payload, BROWSER_CATEGORY_DATA_TYPE, MACOS_BROWSER_PROVIDER_ID,
};

/// Builds an Observation for a browser-category change.
pub fn observation_from_browser_category(
    sample: &BrowserCategorySample,
) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // OS-only mapping without URL → lower confidence when category is unknown.
    let confidence = if sample.category == bio_spec::BROWSER_CATEGORY_UNKNOWN {
        0.5
    } else {
        1.0
    };

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_BROWSER_PROVIDER_ID,
        BROWSER_CATEGORY_DATA_TYPE,
        browser_category_payload(sample),
        confidence,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct BrowserCategoryHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl BrowserCategoryHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "browser category collector task join failed"),
            }
        }
    }
}

impl Drop for BrowserCategoryHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when category identity changes.
pub fn spawn_browser_category_loop(
    tx: ObservationSender,
    probe: Arc<dyn BrowserCategoryProbe>,
    interval: Duration,
) -> BrowserCategoryHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("browser category collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.current() {
                        Ok(Some(sample)) => {
                            let key = sample.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_browser_category(&sample) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping browser_category"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping browser category loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build browser_category Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "browser category probe failed");
                        }
                    }
                }
            }
        }
    });

    BrowserCategoryHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
