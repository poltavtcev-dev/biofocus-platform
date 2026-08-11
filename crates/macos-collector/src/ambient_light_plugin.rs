//! [`plugin_sdk::BioFocusPlugin`] for opt-in ambient light → Observations.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::ambient_light_probe::{AmbientLightProbe, SystemAmbientLightProbe};
use crate::ambient_light_stream::{spawn_ambient_light_loop, AmbientLightHandle};
use crate::payload::AMBIENT_LIGHT_DATA_TYPE;

/// Default rare poll interval (≥5s; emit on change; no busy-loop).
pub const DEFAULT_AMBIENT_LIGHT_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Env flag to enable this collector (default **off**).
pub const AMBIENT_LIGHT_ENABLE_ENV: &str = "BIOFOCUS_AMBIENT_LIGHT";

/// Returns true when `BIOFOCUS_AMBIENT_LIGHT` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn ambient_light_enabled() -> bool {
    match std::env::var(AMBIENT_LIGHT_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Ambient light → `ambient_light` plugin (opt-in, coarse light_kind + optional level).
pub struct AmbientLightPlugin {
    probe: Arc<dyn AmbientLightProbe>,
    interval: Duration,
    handle: Mutex<Option<AmbientLightHandle>>,
}

impl AmbientLightPlugin {
    /// Production plugin using [`SystemAmbientLightProbe`] (soft-fail when OS mapping unavailable).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(
            Arc::new(SystemAmbientLightProbe),
            DEFAULT_AMBIENT_LIGHT_POLL_INTERVAL,
        )
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn AmbientLightProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for AmbientLightPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_AMBIENT_LIGHT_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Ambient Light"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "ambient_light".to_string(),
            data_types: vec![AMBIENT_LIGHT_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_ambient_light_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "ambient_light collector started (coarse light_kind + optional level; no frames in logs)"
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
        info!(plugin = self.id(), "ambient_light collector stopped");
        Ok(())
    }
}
