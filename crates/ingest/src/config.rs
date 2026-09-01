//! Ingest server configuration (default loopback; opt-in LAN bind).

use std::net::Ipv4Addr;

use crate::error::{IngestError, IngestResult};
use crate::token::resolve_ingest_token;

/// Default loopback host when LAN opt-in is off.
pub const INGEST_BIND_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Unspecified IPv4 (`0.0.0.0`) — all interfaces; used when `BIOFOCUS_INGEST_LAN=1`.
pub const INGEST_LAN_BIND_HOST: Ipv4Addr = Ipv4Addr::UNSPECIFIED;

/// Default TCP port for local ingest (`127.0.0.1:8787`).
pub const DEFAULT_INGEST_PORT: u16 = 8787;

/// Environment variable that overrides the on-disk pairing token.
pub use crate::token::INGEST_TOKEN_ENV;

/// Opt-in LAN-reachable bind (`1` / `true` / `yes` / `on`, case-insensitive).
///
/// When set and [`INGEST_BIND_HOST_ENV`] is unset, bind host becomes [`INGEST_LAN_BIND_HOST`].
pub const INGEST_LAN_ENV: &str = "BIOFOCUS_INGEST_LAN";

/// Optional explicit IPv4 bind-host override (e.g. `0.0.0.0` or a LAN NIC address).
///
/// When set to a non-loopback address, this is an explicit opt-in away from default
/// local-only bind. Takes precedence over [`INGEST_LAN_ENV`].
pub const INGEST_BIND_HOST_ENV: &str = "BIOFOCUS_INGEST_BIND_HOST";

/// Explicit Bearer token for unit tests / [`IngestConfig::with_token`].
///
/// Production / host startup should use [`IngestConfig::load`] (persisted or env).
pub const DEFAULT_TEST_TOKEN: &str = "biofocus-dev-ingest-token";

/// T1 name for [`DEFAULT_TEST_TOKEN`] (kept for existing imports).
pub const DEFAULT_SKELETON_TOKEN: &str = DEFAULT_TEST_TOKEN;

/// Configuration for the local ingest HTTP server.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// TCP port. Use `0` in tests for an ephemeral port.
    pub port: u16,
    /// Expected `Authorization: Bearer <token>` value.
    pub token: String,
    /// Bind host (default [`INGEST_BIND_HOST`]; LAN opt-in → [`INGEST_LAN_BIND_HOST`] or override).
    pub bind_host: Ipv4Addr,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_INGEST_PORT,
            token: DEFAULT_TEST_TOKEN.to_owned(),
            bind_host: INGEST_BIND_HOST,
        }
    }
}

impl IngestConfig {
    /// Loads port default, bind host (env knobs), and token from env / pairing file.
    ///
    /// Token resolution: non-empty `BIOFOCUS_INGEST_TOKEN` → else load-or-create file
    /// (see `docs/10-security.md`).
    ///
    /// Bind host: see [`resolve_bind_host`].
    pub fn load() -> IngestResult<Self> {
        Ok(Self {
            port: DEFAULT_INGEST_PORT,
            token: resolve_ingest_token()?,
            bind_host: resolve_bind_host()?,
        })
    }

    /// Alias for [`Self::load`] (T1 name). Prefer `load` in new code.
    pub fn from_env() -> IngestResult<Self> {
        Self::load()
    }

    /// Explicit token (tests / host wiring). Port defaults to [`DEFAULT_INGEST_PORT`];
    /// bind host defaults to loopback.
    #[must_use]
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            port: DEFAULT_INGEST_PORT,
            token: token.into(),
            bind_host: INGEST_BIND_HOST,
        }
    }

    /// `true` when bind is not loopback (LAN opt-in or non-loopback host override).
    #[must_use]
    pub fn is_lan_bind(&self) -> bool {
        !self.bind_host.is_loopback()
    }
}

/// Resolves bind host from env knobs.
///
/// Order:
/// 1. Non-empty `BIOFOCUS_INGEST_BIND_HOST` → parse as IPv4 (invalid → error).
/// 2. Else if `BIOFOCUS_INGEST_LAN` is truthy → [`INGEST_LAN_BIND_HOST`] (`0.0.0.0`).
/// 3. Else if persisted `~/.biofocus/ingest_lan_enabled` → [`INGEST_LAN_BIND_HOST`].
/// 4. Else → [`INGEST_BIND_HOST`] (`127.0.0.1`).
pub fn resolve_bind_host() -> IngestResult<Ipv4Addr> {
    resolve_bind_host_from_env(|key| std::env::var_os(key))
}

pub(crate) fn resolve_bind_host_from_env<F>(mut getenv: F) -> IngestResult<Ipv4Addr>
where
    F: FnMut(&str) -> Option<std::ffi::OsString>,
{
    if let Some(raw) = getenv(INGEST_BIND_HOST_ENV) {
        let value = raw.to_string_lossy();
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.parse::<Ipv4Addr>().map_err(|_| IngestError::InvalidBindHost {
                value: trimmed.to_owned(),
            });
        }
    }

    if lan_flag_enabled(getenv(INGEST_LAN_ENV)) {
        return Ok(INGEST_LAN_BIND_HOST);
    }

    if crate::ingest_prefs::read_persisted_lan_enabled() {
        return Ok(INGEST_LAN_BIND_HOST);
    }

    Ok(INGEST_BIND_HOST)
}

pub(crate) fn lan_flag_enabled(raw: Option<std::ffi::OsString>) -> bool {
    let Some(raw) = raw else {
        return false;
    };
    matches!(
        raw.to_string_lossy().trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ffi::OsString;

    fn with_env(map: HashMap<&'static str, &str>) -> impl FnMut(&str) -> Option<OsString> {
        move |key| map.get(key).map(|v| OsString::from(*v))
    }

    #[test]
    fn resolve_bind_host_defaults_to_loopback() {
        let host = resolve_bind_host_from_env(with_env(HashMap::new())).expect("ok");
        assert_eq!(host, INGEST_BIND_HOST);
        assert!(host.is_loopback());
    }

    #[test]
    fn resolve_bind_host_lan_flag_binds_unspecified() {
        let mut map = HashMap::new();
        map.insert(INGEST_LAN_ENV, "1");
        let host = resolve_bind_host_from_env(with_env(map)).expect("ok");
        assert_eq!(host, INGEST_LAN_BIND_HOST);
        assert!(!host.is_loopback());
    }

    #[test]
    fn resolve_bind_host_lan_flag_case_insensitive() {
        let mut map = HashMap::new();
        map.insert(INGEST_LAN_ENV, "True");
        let host = resolve_bind_host_from_env(with_env(map)).expect("ok");
        assert_eq!(host, INGEST_LAN_BIND_HOST);
    }

    #[test]
    fn resolve_bind_host_explicit_override_wins_over_lan_flag() {
        let mut map = HashMap::new();
        map.insert(INGEST_LAN_ENV, "1");
        map.insert(INGEST_BIND_HOST_ENV, "192.168.1.40");
        let host = resolve_bind_host_from_env(with_env(map)).expect("ok");
        assert_eq!(host, Ipv4Addr::new(192, 168, 1, 40));
    }

    #[test]
    fn resolve_bind_host_explicit_loopback_keeps_local_only() {
        let mut map = HashMap::new();
        map.insert(INGEST_BIND_HOST_ENV, "127.0.0.1");
        let host = resolve_bind_host_from_env(with_env(map)).expect("ok");
        assert_eq!(host, INGEST_BIND_HOST);
    }

    #[test]
    fn resolve_bind_host_rejects_invalid_override() {
        let mut map = HashMap::new();
        map.insert(INGEST_BIND_HOST_ENV, "not-an-ip");
        let err = resolve_bind_host_from_env(with_env(map)).expect_err("invalid");
        assert!(matches!(err, IngestError::InvalidBindHost { .. }));
    }

    #[test]
    fn lan_flag_off_values_do_not_enable() {
        for value in ["0", "false", "no", "off", ""] {
            let mut map = HashMap::new();
            map.insert(INGEST_LAN_ENV, value);
            let host = resolve_bind_host_from_env(with_env(map)).expect("ok");
            assert_eq!(host, INGEST_BIND_HOST, "value={value:?}");
        }
    }

    #[test]
    fn default_config_is_loopback() {
        let cfg = IngestConfig::default();
        assert!(!cfg.is_lan_bind());
        assert_eq!(cfg.bind_host, INGEST_BIND_HOST);
    }
}
