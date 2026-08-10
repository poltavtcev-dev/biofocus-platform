//! [`plugin_sdk::BioFocusPlugin`] for opt-in Git activity → Observations.

use std::sync::Arc;
use std::time::Duration;

use plugin_sdk::{BioFocusPlugin, Capability, PluginError};
use runtime::ObservationSender;
use tokio::sync::Mutex;
use tracing::info;

use crate::git_activity_probe::{GitActivityProbe, SystemGitActivityProbe};
use crate::git_activity_stream::{spawn_git_activity_loop, GitActivityHandle};
use crate::payload::GIT_ACTIVITY_DATA_TYPE;

/// Default rare poll interval (≥5s; emit on change; no busy-loop).
pub const DEFAULT_GIT_ACTIVITY_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Env flag to enable this collector (default **off**).
pub const GIT_ACTIVITY_ENABLE_ENV: &str = "BIOFOCUS_GIT_ACTIVITY";

/// Returns true when `BIOFOCUS_GIT_ACTIVITY` is `1`/`true`/`yes`/`on`.
#[must_use]
pub fn git_activity_enabled() -> bool {
    match std::env::var(GIT_ACTIVITY_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Git activity → `git_activity` plugin (opt-in, coarse activity_kind only).
pub struct GitActivityPlugin {
    probe: Arc<dyn GitActivityProbe>,
    interval: Duration,
    handle: Mutex<Option<GitActivityHandle>>,
}

impl GitActivityPlugin {
    /// Production plugin using [`SystemGitActivityProbe`] (soft-fail without allowlist).
    #[must_use]
    pub fn system_default() -> Self {
        Self::with_probe(
            Arc::new(SystemGitActivityProbe),
            DEFAULT_GIT_ACTIVITY_POLL_INTERVAL,
        )
    }

    /// Injectable probe (tests / alternate backends).
    #[must_use]
    pub fn with_probe(probe: Arc<dyn GitActivityProbe>, interval: Duration) -> Self {
        Self {
            probe,
            interval,
            handle: Mutex::new(None),
        }
    }
}

impl BioFocusPlugin for GitActivityPlugin {
    fn id(&self) -> &str {
        crate::payload::MACOS_GIT_PROVIDER_ID
    }

    fn name(&self) -> &str {
        "macOS Git Activity"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "git_activity".to_string(),
            data_types: vec![GIT_ACTIVITY_DATA_TYPE.to_string()],
        }]
    }

    async fn start_stream(&self, tx: ObservationSender) -> Result<(), PluginError> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Err(PluginError::AlreadyStarted);
        }

        let handle = spawn_git_activity_loop(tx, Arc::clone(&self.probe), self.interval);
        info!(
            plugin = self.id(),
            interval_ms = self.interval.as_millis() as u64,
            "git_activity collector started (coarse activity_kind only; no paths in logs)"
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
        info!(plugin = self.id(), "git_activity collector stopped");
        Ok(())
    }
}
