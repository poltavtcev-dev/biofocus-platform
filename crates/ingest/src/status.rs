//! HTTP status payload for companion/debug (`GET /v1/status`).
//!
//! Soft-fails DB problems into `db_status: "error"` (same spirit as IPC `get_status`).
//! Never includes Observation or biometric fields, tokens, or absolute DB paths.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::advertise::{AdvertiseInfo, BindMode};
use crate::config::{DEFAULT_INGEST_PORT, INGEST_BIND_HOST};

/// Response body for `GET /v1/status` (`docs/09-api.md`). Snake_case JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusResponse {
    /// App / host version string.
    pub version: String,
    /// `"ok"` after successful DB probe; otherwise `"error"`.
    pub db_status: String,
    /// Short storage error when `db_status == "error"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_error: Option<String>,
    /// Loopback vs LAN opt-in bind (P5-E1-T2).
    pub bind_mode: BindMode,
    /// Usable ingest base URL hints (primary first). No Observation / path data.
    pub base_url_hints: Vec<String>,
}

impl StatusResponse {
    /// Builds a status payload from a soft-fail probe + advertise hints.
    #[must_use]
    pub fn from_probe(
        version: impl Into<String>,
        probe: Result<(), String>,
        advertise: &AdvertiseInfo,
    ) -> Self {
        let version = version.into();
        let (db_status, db_error) = match probe {
            Ok(()) => ("ok".to_owned(), None),
            Err(message) => ("error".to_owned(), Some(message)),
        };
        Self {
            version,
            db_status,
            db_error,
            bind_mode: advertise.bind_mode,
            base_url_hints: advertise.base_url_hints.clone(),
        }
    }

    /// Convenience for tests that only care about DB probe fields (loopback advertise).
    #[must_use]
    pub fn from_probe_loopback(version: impl Into<String>, probe: Result<(), String>) -> Self {
        let advertise = AdvertiseInfo::for_bind(INGEST_BIND_HOST, DEFAULT_INGEST_PORT);
        Self::from_probe(version, probe, &advertise)
    }
}

/// Opens (or creates) a DB at `path` with WAL + migrate; soft-fails as `Err(String)`.
///
/// Error strings use [`storage::StorageError::public_message`] (no filesystem paths).
pub fn probe_db_at(path: impl AsRef<Path>) -> Result<(), String> {
    storage::Database::open(path.as_ref())
        .map(|_| ())
        .map_err(|err| err.public_message())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn from_probe_error_keeps_safe_message() {
        let err = storage::StorageError::CreateDir {
            path: PathBuf::from("/Users/secret/.biofocus/data"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let advertise = AdvertiseInfo::for_bind(INGEST_BIND_HOST, DEFAULT_INGEST_PORT);
        let status = StatusResponse::from_probe("0.1.0", Err(err.public_message()), &advertise);
        assert_eq!(status.db_status, "error");
        let msg = status.db_error.as_deref().expect("db_error");
        assert_eq!(msg, "Could not create local data directory.");
        assert!(!msg.contains('/'));
        assert_eq!(status.bind_mode, BindMode::Loopback);
        assert_eq!(
            status.base_url_hints,
            vec!["http://127.0.0.1:8787".to_owned()]
        );
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(!json.contains("/Users"));
        assert!(!json.contains(".biofocus"));
        assert!(!json.contains("observation"));
    }

    #[test]
    fn from_probe_includes_lan_hints() {
        let advertise =
            AdvertiseInfo::for_bind_with(crate::config::INGEST_LAN_BIND_HOST, 8787, || {
                vec![std::net::Ipv4Addr::new(192, 168, 1, 10)]
            });
        let status = StatusResponse::from_probe("0.1.0", Ok(()), &advertise);
        assert_eq!(status.bind_mode, BindMode::Lan);
        assert_eq!(
            status.base_url_hints,
            vec!["http://192.168.1.10:8787".to_owned()]
        );
        assert!(status.db_error.is_none());
    }
}
