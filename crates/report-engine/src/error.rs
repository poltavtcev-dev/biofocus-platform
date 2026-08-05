//! Typed errors for report / prompt generation.

use thiserror::Error;

/// Fallible report-engine operations.
pub type ReportResult<T> = Result<T, ReportEngineError>;

/// Errors raised while building a markdown report or LLM prompt.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ReportEngineError {
    /// Report assembly failed (reserved for future validation / formatting).
    #[error("report build failed: {message}")]
    BuildFailed {
        /// Failure detail.
        message: String,
    },
}
