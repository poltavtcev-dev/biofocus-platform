//! Tracing subscriber bootstrap for Core.

use tracing_subscriber::EnvFilter;

use crate::RuntimeResult;

/// Initializes a global tracing subscriber.
///
/// Filter resolution order:
/// 1. `RUST_LOG` env var (via [`EnvFilter::try_from_default_env`])
/// 2. optional `default_filter` argument (e.g. `info`)
/// 3. fallback `warn`
///
/// If a global subscriber is already set, this returns `Ok(())` (idempotent for
/// hosts that embed Core more than once in tests).
pub fn init_tracing(default_filter: Option<&str>) -> RuntimeResult<()> {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new(default_filter.unwrap_or("warn")),
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .finish();

    match tracing::subscriber::set_global_default(subscriber) {
        Ok(()) => Ok(()),
        Err(_) => {
            tracing::debug!("tracing subscriber already installed");
            Ok(())
        }
    }
}
