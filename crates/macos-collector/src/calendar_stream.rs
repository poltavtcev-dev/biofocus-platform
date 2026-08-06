//! Rare poll loop: emit `calendar_event` Observations from a local Calendar probe.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bio_spec::Observation;
use runtime::ObservationSender;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::calendar_probe::{CalendarEvent, CalendarProbe};
use crate::error::CollectorResult;
use crate::payload::{calendar_event_payload, CALENDAR_EVENT_DATA_TYPE, MACOS_CALENDAR_PROVIDER_ID};

/// Default lookback for dogfood horizon (past).
pub const DEFAULT_HORIZON_PAST: Duration = Duration::from_secs(24 * 3600);
/// Default lookahead for dogfood horizon (future).
pub const DEFAULT_HORIZON_FUTURE: Duration = Duration::from_secs(48 * 3600);

/// Builds an Observation for a privacy-safe calendar event.
///
/// `Observation.timestamp` = event `start` (useful for MeetingDensity windows).
pub fn observation_from_calendar_event(event: &CalendarEvent) -> CollectorResult<Observation> {
    Observation::try_new(
        Uuid::now_v7(),
        bio_spec::UnixTimestamp(event.start),
        MACOS_CALENDAR_PROVIDER_ID,
        CALENDAR_EVENT_DATA_TYPE,
        calendar_event_payload(event),
        1.0,
    )
    .map_err(Into::into)
}

/// Handle that stops the calendar poll loop.
pub struct CalendarHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: Option<tokio::task::JoinHandle<()>>,
}

impl CalendarHandle {
    /// Signals stop and awaits the loop task.
    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            match join.await {
                Ok(()) => {}
                Err(err) if err.is_cancelled() => {}
                Err(err) => warn!(error = %err, "calendar collector task join failed"),
            }
        }
    }
}

impl Drop for CalendarHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(join) = self.join.take() {
            join.abort();
        }
    }
}

/// Spawns a rare poll loop (≥`interval`). Emits each event once (by uid+start+end).
///
/// Idle-safe: waits on `tokio::time::interval` + `select!` stop — no busy-loop.
pub fn spawn_calendar_loop(
    tx: ObservationSender,
    probe: Arc<dyn CalendarProbe>,
    interval: Duration,
    horizon_past: Duration,
    horizon_future: Duration,
    poll_counter: Option<Arc<AtomicUsize>>,
) -> CalendarHandle {
    let (stop_tx, mut stop_rx) = oneshot::channel();

    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // First tick completes immediately — emit once at start, then wait.
        let mut emitted: HashSet<String> = HashSet::new();

        loop {
            tokio::select! {
                biased;
                _ = &mut stop_rx => {
                    debug!("calendar collector stop requested");
                    break;
                }
                _ = ticker.tick() => {
                    if let Some(counter) = &poll_counter {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let horizon_start = now.saturating_sub(horizon_past.as_secs() as i64);
                    let horizon_end = now.saturating_add(horizon_future.as_secs() as i64);

                    match probe.events_in_range(horizon_start, horizon_end) {
                        Ok(events) => {
                            for event in events {
                                let key = event.emit_key();
                                if emitted.contains(&key) {
                                    continue;
                                }
                                match observation_from_calendar_event(&event) {
                                    Ok(obs) => {
                                        match tx.try_send(obs) {
                                            Ok(()) => {
                                                emitted.insert(key);
                                            }
                                            Err(tokio::sync::mpsc::error::TrySendError::Full(obs)) => {
                                                // Log id only — never uid/title/body.
                                                warn!(
                                                    id = %obs.id,
                                                    "observation channel full; dropping calendar_event"
                                                );
                                            }
                                            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                                debug!(
                                                    "observation channel closed; stopping calendar loop"
                                                );
                                                return;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        warn!(error = %err, "failed to build calendar_event Observation");
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            // Probe failures must not include event titles (ICS path errors are path+IO).
                            warn!(error = %err, "calendar probe failed");
                        }
                    }
                }
            }
        }
    });

    CalendarHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}
