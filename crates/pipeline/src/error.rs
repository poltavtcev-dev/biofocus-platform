//! Typed errors for the Observation pipeline.

use thiserror::Error;

/// Fallible pipeline operations.
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Errors raised while accepting or processing Observation batches.
///
/// Intake / dedupe default paths do not reject; variants below reserve structured
/// failure paths for policy / later stages (normalize).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PipelineError {
    /// Batch rejected at intake (validation / policy). Unused by default intake path.
    #[error("observation intake rejected: {reason}")]
    IntakeRejected {
        /// Human-readable rejection reason (static for now).
        reason: &'static str,
    },

    /// A downstream stage failed (dedupe / normalize). Reserved; default T2 dedupe
    /// always succeeds ([`Ok`]).
    #[error("pipeline stage `{stage}` failed: {message}")]
    StageFailed {
        /// Stage name (e.g. `dedupe`, `normalize`).
        stage: &'static str,
        /// Failure detail.
        message: String,
    },
}
