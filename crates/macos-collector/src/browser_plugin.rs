//! [`plugin_sdk::BioFocusPlugin`] for opt-in Browser categories → Observations.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::browser_probe::{BrowserCategoryProbe, SystemBrowserProbe};
use crate::browser_stream::{spawn_browser_category_loop, BrowserCategoryHandle};
use crate::payload::BROWSER_CATEGORY_DATA_TYPE;

/// Default rare poll interval (≥5s; emit on change; no busy-loop).
pub const DEFAULT_BROWSER_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Env flag to enable this collector (default **off**).
pub const BROWSER_ENABLE_ENV: &str = "BIOFOCUS_BROWSER_CATEGORIES";

/// Returns true when `BIOFOCUS_BROWSER_CATEGORIES` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn browser_categories_enabled() -> bool {
    match std::env::var(BROWSER_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Browser categories → `browser_category` plugin (opt-in, coarse labels only).
pub struct BrowserCategoryPlugin {
    probe: Arc<dyn BrowserCategoryProbe>,
    interval: Duration,
    handle: Mutex<Option<BrowserCategoryHandle>>,
}

impl BrowserCategoryPlugin {
    /// Production plugin using [`SystemBrowserProbe`] (soft-fail without URL mapping).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(Arc::new(SystemBrowserProbe), DEFAULT_BROWSER_POLL_INTERVAL)
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn BrowserCategoryProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for BrowserCategoryPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_BROWSER_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Browser Categories"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "browser_categories".to_string(),
            data_types: vec![BROWSER_CATEGORY_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_browser_category_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "browser category collector started (coarse labels only; no URLs in logs)"
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
        info!(plugin = self.id(), "browser category collector stopped");
        Ok(())
    }
}
