//! Ingest server configuration (loopback-only).

use std::net::Ipv4Addr;

use crate::error::IngestResult;
use crate::token::resolve_ingest_token;

/// Loopback host — ingest must never bind beyond this in Phase 2 E1.
pub const INGEST_BIND_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Default TCP port for local ingest (`127.0.0.1:8787`).
pub const DEFAULT_INGEST_PORT: u16 = 8787;

/// Environment variable that overrides the on-disk pairing token.
pub use crate::token::INGEST_TOKEN_ENV;

/// Explicit Bearer token for unit tests / [`IngestConfig::with_token`].
///
/// Production / host startup should use [`IngestConfig::load`] (persisted or env).
pub const DEFAULT_TEST_TOKEN: &str = "biofocus-dev-ingest-token";

/// T1 name for [`DEFAULT_TEST_TOKEN`] (kept for existing imports).
pub const DEFAULT_SKELETON_TOKEN: &str = DEFAULT_TEST_TOKEN;

/// Configuration for the local ingest HTTP server.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// TCP port on [`INGEST_BIND_HOST`]. Use `0` in tests for an ephemeral port.
    pub port: u16,
    /// Expected `Authorization: Bearer <token>` value.
    pub token: String,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_INGEST_PORT,
            token: DEFAULT_TEST_TOKEN.to_owned(),
        }
    }
}

impl IngestConfig {
    /// Loads port default + token from env override or `~/.biofocus/pairing_token`.
    ///
    /// Token resolution: non-empty `BIOFOCUS_INGEST_TOKEN` → else load-or-create file
    /// (see `docs/10-security.md`).
    pub fn load() -> IngestResult<Self> {
        Ok(Self {
            port: DEFAULT_INGEST_PORT,
            token: resolve_ingest_token()?,
        })
    }

    /// Alias for [`Self::load`] (T1 name). Prefer `load` in new code.
    pub fn from_env() -> IngestResult<Self> {
        Self::load()
    }

    /// Explicit token (tests / host wiring). Port defaults to [`DEFAULT_INGEST_PORT`].
    #[must_use]
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            port: DEFAULT_INGEST_PORT,
            token: token.into(),
        }
    }
}
