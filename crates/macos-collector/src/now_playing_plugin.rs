//! [`plugin_sdk::BioFocusPlugin`] for opt-in Now Playing → Observations.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::now_playing_probe::{NowPlayingProbe, SystemNowPlayingProbe};
use crate::now_playing_stream::{spawn_now_playing_loop, NowPlayingHandle};
use crate::payload::NOW_PLAYING_DATA_TYPE;

/// Default rare poll interval (≥5s; emit on change; no busy-loop).
pub const DEFAULT_NOW_PLAYING_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Env flag to enable this collector (default **off**).
pub const NOW_PLAYING_ENABLE_ENV: &str = "BIOFOCUS_NOW_PLAYING";

/// Returns true when `BIOFOCUS_NOW_PLAYING` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn now_playing_enabled() -> bool {
    match std::env::var(NOW_PLAYING_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Now Playing → `now_playing` plugin (opt-in, coarse media_kind + is_playing only).
pub struct NowPlayingPlugin {
    probe: Arc<dyn NowPlayingProbe>,
    interval: Duration,
    handle: Mutex<Option<NowPlayingHandle>>,
}

impl NowPlayingPlugin {
    /// Production plugin using [`SystemNowPlayingProbe`] (soft-fail when OS mapping unavailable).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(Arc::new(SystemNowPlayingProbe), DEFAULT_NOW_PLAYING_POLL_INTERVAL)
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn NowPlayingProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for NowPlayingPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_NOW_PLAYING_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Now Playing"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "now_playing".to_string(),
            data_types: vec![NOW_PLAYING_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_now_playing_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "now_playing collector started (coarse media_kind + is_playing; no titles in logs)"
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
        info!(plugin = self.id(), "now_playing collector stopped");
        Ok(())
    }
}
