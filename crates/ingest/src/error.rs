//! Typed errors for the ingest HTTP server.

use thiserror::Error;

/// Fallible ingest bootstrap / bind operations.
pub type IngestResult<T> = Result<T, IngestError>;

/// Errors raised while binding or running the ingest server.
#[derive(Debug, Error)]
pub enum IngestError {
    /// Failed to bind the TCP listener (port in use, permissions, etc.).
    #[error("failed to bind ingest listener on {addr}: {source}")]
    Bind {
        /// Address that was requested.
        addr: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed while serving HTTP connections.
    #[error("ingest server error: {0}")]
    Serve(#[source] std::io::Error),

    /// `$HOME` (or equivalent) is unavailable when resolving `~/.biofocus/`.
    #[error("home directory unavailable; cannot resolve pairing token path")]
    HomeDirUnavailable,

    /// Failed to read or write the pairing token file.
    #[error("pairing token I/O at {path}: {source}")]
    TokenIo {
        /// Path that was accessed.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Pairing token file exists but contains only whitespace.
    #[error("pairing token file is empty: {path}")]
    EmptyTokenFile {
        /// Path of the empty file.
        path: String,
    },

    /// OS entropy source failed while generating a pairing token.
    #[error("failed to gather entropy for pairing token: {0}")]
    TokenEntropy(String),
}
