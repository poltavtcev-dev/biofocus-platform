//! [`plugin_sdk::BioFocusPlugin`] for opt-in notification events → Observations.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::notification_probe::{NotificationEventProbe, SystemNotificationEventProbe};
use crate::notification_stream::{spawn_notification_event_loop, NotificationEventHandle};
use crate::payload::NOTIFICATION_EVENT_DATA_TYPE;

/// Default rare poll interval (≥5s; emit on change; no busy-loop).
pub const DEFAULT_NOTIFICATION_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Env flag to enable this collector (default **off**).
pub const NOTIFICATION_ENABLE_ENV: &str = "BIOFOCUS_NOTIFICATION_EVENTS";

/// Returns true when `BIOFOCUS_NOTIFICATION_EVENTS` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn notification_events_enabled() -> bool {
    match std::env::var(NOTIFICATION_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Notification deliveries → `notification_event` plugin (opt-in, coarse count + labels).
pub struct NotificationPlugin {
    probe: Arc<dyn NotificationEventProbe>,
    interval: Duration,
    handle: Mutex<Option<NotificationEventHandle>>,
}

impl NotificationPlugin {
    /// Production plugin using [`SystemNotificationEventProbe`] (soft-fail when OS mapping unavailable).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(
            Arc::new(SystemNotificationEventProbe),
            DEFAULT_NOTIFICATION_POLL_INTERVAL,
        )
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn NotificationEventProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for NotificationPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_NOTIFICATIONS_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Notifications"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "notification_events".to_string(),
            data_types: vec![NOTIFICATION_EVENT_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_notification_event_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "notification_event collector started (coarse count + labels; no body/title in logs)"
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
        info!(plugin = self.id(), "notification_event collector stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_poll_interval_is_at_least_five_seconds() {
        assert!(DEFAULT_NOTIFICATION_POLL_INTERVAL >= Duration::from_secs(5));
    }

    #[test]
    fn plugin_id_and_capability_match_contract() {
        let plugin = NotificationPlugin::system_default();
        assert_eq!(plugin.id(), crate::payload::MACOS_NOTIFICATIONS_PROVIDER_ID);
        let caps = plugin.capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].name, "notification_events");
        assert_eq!(
            caps[0].data_types,
            vec![NOTIFICATION_EVENT_DATA_TYPE.to_string()]
        );
    }
}
