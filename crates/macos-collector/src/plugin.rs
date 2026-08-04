//! [`plugin_sdk::BioFocusPlugin`] for active window streaming.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::payload::CONTEXT_WINDOW_DATA_TYPE;
use crate::probe::{FrontmostProbe, SystemFrontmostProbe};
use crate::stream::{spawn_active_window_loop, ActiveWindowHandle};

/// Default poll interval (≥1s per AC).
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Active window → `context_window` plugin.
pub struct ActiveWindowPlugin {
    probe: Arc<dyn FrontmostProbe>,
    interval: Duration,
    handle: Mutex<Option<ActiveWindowHandle>>,
}

impl ActiveWindowPlugin {
    /// Production plugin using [`SystemFrontmostProbe`].
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(Arc::new(SystemFrontmostProbe), DEFAULT_POLL_INTERVAL)
    }

    /// Injectable probe (tests / alternate OS backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn FrontmostProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for ActiveWindowPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_CONTEXT_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Active Window"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "active_window".to_string(),
            data_types: vec![CONTEXT_WINDOW_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_active_window_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "active window collector started"
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
        info!(plugin = self.id(), "active window collector stopped");
        Ok(())
    }
}
