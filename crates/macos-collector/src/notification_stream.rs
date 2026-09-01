//! Poll loop: emit `notification_event` Observation when sample identity changes.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::CollectorResult;
use crate::notification_probe::{NotificationEventProbe, NotificationEventSample};
use crate::payload::{
    notification_event_payload, MACOS_NOTIFICATIONS_PROVIDER_ID, NOTIFICATION_EVENT_DATA_TYPE,
};

/// Builds an Observation for a notification event sample.
pub fn observation_from_notification_event(
    sample: &NotificationEventSample,
) -> CollectorResult<Observation> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let confidence = if sample.category.as_deref() == Some("unknown")
        || sample.interruption_level.as_deref() == Some("unknown")
        || sample.app_kind.as_deref() == Some("unknown")
    {
        0.5
    } else {
        1.0
    };

    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(timestamp),
        MACOS_NOTIFICATIONS_PROVIDER_ID,
        NOTIFICATION_EVENT_DATA_TYPE,
        notification_event_payload(sample),
        confidence,
    )
    .map_err(Into::into)
}

/// Handle that stops the poll loop and waits for the task to finish.
pub struct NotificationEventHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl NotificationEventHandle {
    /// Signals stop and awaits the loop task (idle exit via `tokio::select!`).
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "notification_event collector task join failed"),
            }
        }
    }
}

impl Drop for NotificationEventHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a ≥`interval` poll loop. Emits only when sample identity changes.
pub fn spawn_notification_event_loop(
    tx: ObservationSender,
    probe: Arc<dyn NotificationEventProbe>,
    interval: Duration,
) -> NotificationEventHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let mut last_key: Option<String> = None;

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("notification_event collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    match probe.current() {
                        Ok(Some(sample)) => {
                            if sample.count < 1 {
                                continue;
                            }
                            let key = sample.identity_key();
                            if last_key.as_ref() == Some(&key) {
                                continue;
                            }
                            match observation_from_notification_event(&sample) {
                                Ok(obs) => {
                                    match tx.try_send(obs) {
                                        Ok(()) => {
                                            last_key = Some(key);
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                            warn!(
                                                id = %obs.id,
                                                "observation channel full; dropping notification_event"
                                            );
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            debug!(
                                                "observation channel closed; stopping notification_event loop"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(error = %err, "failed to build notification_event Observation");
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(err) => {
                            warn!(error = %err, "notification_event probe failed");
                        }
                    }
                }
            }
        }
    });

    NotificationEventHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
