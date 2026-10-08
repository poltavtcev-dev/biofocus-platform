//! Typed storage errors.

use std::path::PathBuf;

use bio_spec::{ObservationId, SpecError};
use thiserror::Error;

/// Fallible storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Errors from opening, migrating, or querying the local SQLite database.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Parent directories for the DB file could not be created.
    #[error("failed to create data directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Home directory could not be resolved for the default DB path.
    #[error("cannot resolve home directory for default BioFocus data path")]
    HomeDirUnavailable,

    /// SQLite open, pragma, or query failed.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A pragma returned an unexpected value.
    #[error("expected pragma {pragma} = {expected}, got {actual}")]
    PragmaMismatch {
        pragma: &'static str,
        expected: String,
        actual: String,
    },

    /// System clock is unavailable for `created_at` / migration timestamps.
    #[error("system clock error: {source}")]
    Clock {
        #[source]
        source: std::time::SystemTimeError,
    },

    /// Insert rejected because an Observation with this id already exists.
    #[error("observation already exists: {id}")]
    DuplicateObservation { id: ObservationId },

    /// `list_by_time_range` called with `end < start`.
    #[error("time range end ({end}) must be >= start ({start})")]
    InvalidTimeRange { start: i64, end: i64 },

    /// Stored id is not a valid UUID.
    #[error("invalid observation id `{value}`: {source}")]
    InvalidObservationId {
        value: String,
        #[source]
        source: uuid::Error,
    },

    /// Failed to serialize Observation payload to JSON text.
    #[error("failed to serialize observation payload: {source}")]
    PayloadSerialize {
        #[source]
        source: serde_json::Error,
    },

    /// Failed to deserialize stored payload JSON.
    #[error("failed to deserialize observation payload: {source}")]
    PayloadDeserialize {
        #[source]
        source: serde_json::Error,
    },

    /// Domain validation failed while mapping a row (e.g. confidence bounds).
    #[error(transparent)]
    Domain(#[from] SpecError),
}

impl StorageError {
    /// Short UI / IPC / HTTP status message **without filesystem paths**.
    ///
    /// Use for surfaces that may reach the shell (`dbError`) or companions
    /// (`db_error`). Full [`Display`] may still include paths for logs.
    #[must_use]
    pub fn public_message(&self) -> String {
        match self {
            Self::CreateDir { .. } => "Could not create local data directory.".into(),
            Self::HomeDirUnavailable => "Could not locate local data directory.".into(),
            Self::Sqlite(_) => "Could not open local database.".into(),
            Self::PragmaMismatch { .. } => "Database configuration check failed.".into(),
            Self::Clock { .. } => "System clock unavailable.".into(),
            Self::DuplicateObservation { .. } => "Observation already exists.".into(),
            Self::InvalidTimeRange { .. } => "Invalid time range.".into(),
            Self::InvalidObservationId { .. } => "Invalid observation id.".into(),
            Self::PayloadSerialize { .. } => "Could not serialize observation payload.".into(),
            Self::PayloadDeserialize { .. } => "Could not read observation payload.".into(),
            Self::Domain(_) => "Invalid observation data.".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn public_message_create_dir_hides_path() {
        let err = StorageError::CreateDir {
            path: PathBuf::from("/Users/secret/.biofocus/data"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let msg = err.public_message();
        assert_eq!(msg, "Could not create local data directory.");
        assert!(!msg.contains("/Users"));
        assert!(!msg.contains(".biofocus"));
        // Display may still leak for logs — public surface must not.
        assert!(err.to_string().contains("/Users/secret"));
    }

    #[test]
    fn public_message_home_and_sqlite_are_short() {
        assert_eq!(
            StorageError::HomeDirUnavailable.public_message(),
            "Could not locate local data directory."
        );
        let sqlite = StorageError::Sqlite(rusqlite::Error::InvalidPath(PathBuf::from(
            "/tmp/hidden/biofocus_main.db",
        )));
        let msg = sqlite.public_message();
        assert_eq!(msg, "Could not open local database.");
        assert!(!msg.contains("/tmp"));
        assert!(!msg.contains("biofocus_main"));
    }
}
