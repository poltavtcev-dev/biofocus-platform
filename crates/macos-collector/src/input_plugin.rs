//! [`plugin_sdk::BioFocusPlugin`] for privacy-safe keystroke aggregates.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::input_probe::{InputCountProbe, SystemInputProbe};
use crate::input_stream::{spawn_keystroke_aggregate_loop, KeystrokeAggregateHandle};
use crate::payload::KEYSTROKES_DATA_TYPE;

/// Default aggregation window (≥1s; prefer ~60s in production).
pub const DEFAULT_AGGREGATE_WINDOW: Duration = Duration::from_secs(60);

/// Env flag to enable this collector (default **off**).
pub const ENABLE_ENV: &str = "BIOFOCUS_INPUT_AGGREGATES";

/// Returns true when `BIOFOCUS_INPUT_AGGREGATES` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn input_aggregates_enabled() -> bool {
    match std::env::var(ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Key-down aggregates → `keystrokes` plugin (opt-in).
pub struct KeystrokeAggregatePlugin {
    probe: Arc<dyn InputCountProbe>,
    window: Duration,
    handle: Mutex<Option<KeystrokeAggregateHandle>>,
}

impl KeystrokeAggregatePlugin {
    /// Production plugin using [`SystemInputProbe`] (Accessibility listen-only tap).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(Arc::new(SystemInputProbe::new()), DEFAULT_AGGREGATE_WINDOW)
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn InputCountProbe>, window: Duration) -> Self {
        Self {
            probe,
            window,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for KeystrokeAggregatePlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_INPUT_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Input Aggregates"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "input_aggregates".to_string(),
            data_types: vec![KEYSTROKES_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_keystroke_aggregate_loop(tx, Arc::clone(&self.probe), self.window);
        info!(
            plugin = self.id(),
            window_ms = self.window.as_millis() as u64,
            accessibility = self.probe.accessibility_trusted(),
            "keystroke aggregate collector started"
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
        info!(plugin = self.id(), "keystroke aggregate collector stopped");
        Ok(())
    }
}
