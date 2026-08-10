//! Typed errors for report / prompt generation and optional local LLM.

use thiserror::Error;

/// Fallible report-engine operations.
pub type ReportResult<T> = Result<T, ReportEngineError>;

/// Errors raised while building a markdown report or calling the local LLM.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ReportEngineError {
    /// Report assembly failed (reserved for future validation / formatting).
    #[error("report build failed: {message}")]
    BuildFailed {
        /// Failure detail.
        message: String,
    },
    /// Requested prompt pack id/version is not in the in-process registry.
    #[error("unknown prompt pack: {id}@{version}")]
    UnknownPromptPack {
        /// Pack id (e.g. `biofocus.default`).
        id: String,
        /// Pack version string (e.g. `1`).
        version: String,
    },
    /// Local LLM path is opt-in and currently disabled (no network attempted).
    #[error("local LLM is disabled (set {env}=1 to opt in)", env = crate::llm::LOCAL_LLM_ENV)]
    LocalLlmDisabled,
    /// HTTP client / transport failure talking to the local LLM endpoint.
    #[error("local LLM HTTP error: {message}")]
    LocalLlmHttp {
        /// Failure detail.
        message: String,
    },
    /// Request exceeded the configured HTTP timeout.
    #[error("local LLM request timed out (limit {timeout_ms}ms)")]
    LocalLlmTimeout {
        /// Configured timeout in milliseconds.
        timeout_ms: u64,
    },
    /// Response JSON missing usable completion content.
    #[error("local LLM response error: {message}")]
    LocalLlmResponse {
        /// Failure detail.
        message: String,
    },
}
