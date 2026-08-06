//! [`plugin_sdk::BioFocusPlugin`] for opt-in local Calendar → Observations.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::calendar_probe::{CalendarProbe, IcsFileCalendarProbe};
use crate::calendar_stream::{
    spawn_calendar_loop, CalendarHandle, DEFAULT_HORIZON_FUTURE, DEFAULT_HORIZON_PAST,
};
use crate::payload::CALENDAR_EVENT_DATA_TYPE;

/// Default rare poll interval (≥60s; no busy-loop).
pub const DEFAULT_CALENDAR_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Env flag to enable this collector (default **off**).
pub const CALENDAR_ENABLE_ENV: &str = "BIOFOCUS_CALENDAR";

/// Env path to a local `.ics` file (required for system dogfood when enabled).
pub const CALENDAR_ICS_ENV: &str = "BIOFOCUS_CALENDAR_ICS";

/// Returns true when `BIOFOCUS_CALENDAR` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn calendar_enabled() -> bool {
    match std::env::var(CALENDAR_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Reads `BIOFOCUS_CALENDAR_ICS` if set and non-empty.
#[must_use]
pub fn calendar_ics_path_from_env() -> Option<PathBuf> {
    match std::env::var(CALENDAR_ICS_ENV) {
        Ok(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        }
        Err(_) => None,
    }
}

/// Local Calendar → `calendar_event` plugin (opt-in, ICS dogfood).
pub struct CalendarPlugin {
    probe: Arc<dyn CalendarProbe>,
    interval: Duration,
    horizon_past: Duration,
    horizon_future: Duration,
    handle: Mutex<Option<CalendarHandle>>,
}

impl CalendarPlugin {
    /// Production plugin reading a local ICS path (no cloud OAuth).
    #[must_use]
    pub fn from_ics_path(path: PathBuf) -> Self {
        Self::with_probe(
            Arc::new(IcsFileCalendarProbe::new(path)),
            DEFAULT_CALENDAR_POLL_INTERVAL,
        )
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn CalendarProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            horizon_past: DEFAULT_HORIZON_PAST,
            horizon_future: DEFAULT_HORIZON_FUTURE,
            handle: Mutex::new(None),
        }
    }

    /// Override lookaround horizon (tests).
    #[must_use]
    pub fn with_horizon(mut self, past: Duration, future: Duration) -> Self {
        self.horizon_past = past;
        self.horizon_future = future;
        self
    }
}

impl BioFocusPlugin for CalendarPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_CALENDAR_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Calendar (local ICS)"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "calendar_events".to_string(),
            data_types: vec![CALENDAR_EVENT_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_calendar_loop(
            tx,
            Arc::clone(&self.probe),
            self.interval,
            self.horizon_past,
            self.horizon_future,
            None,
        );
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "calendar collector started (local ICS; no titles in logs)"
        );
        *guard = Some(handle);
        Ok(())
    }

    async fn stop_stream(&self) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        let Some(handle) = guard.take() else {
            return Err(PluginError::NotRunning);
        };
        handle.stop().await;
        info!(plugin = self.id(), "calendar collector stopped");
        Ok(())
    }
}
