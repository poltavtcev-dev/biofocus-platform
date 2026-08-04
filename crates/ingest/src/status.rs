//! HTTP status payload for companion/debug (`GET /v1/status`).
//!
//! Soft-fails DB problems into `db_status: "error"` (same spirit as IPC `get_status`).
//! Never includes Observation or biometric fields.

use std::path::Path;

use serde::{Deserialize, Serialize};

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
}

impl StatusResponse {
    /// Builds a status payload from a soft-fail probe result.
    #[must_use]
    pub fn from_probe(version: impl Into<String>, probe: Result<(), String>) -> Self {
        let version = version.into();
        match probe {
            Ok(()) => Self {
                version,
                db_status: "ok".to_owned(),
                db_error: None,
            },
            Err(message) => Self {
                version,
                db_status: "error".to_owned(),
                db_error: Some(message),
            },
        }
    }
}

/// Opens (or creates) a DB at `path` with WAL + migrate; soft-fails as `Err(String)`.
pub fn probe_db_at(path: impl AsRef<Path>) -> Result<(), String> {
    storage::Database::open(path.as_ref())
        .map(|_| ())
        .map_err(|err| err.to_string())
}
