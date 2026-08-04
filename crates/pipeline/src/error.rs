//! Typed errors for the Observation pipeline.

use thiserror::Error;

/// Fallible pipeline operations.
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Errors raised while accepting or processing Observation batches.
///
/// Intake (`accept_observations`) does not reject batches in T1; variants below
/// reserve structured failure paths for later stages (dedupe / normalize / policy).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PipelineError {
    /// Batch rejected at intake (validation / policy). Unused by default T1 path.
    #[error("observation intake rejected: {reason}")]
    IntakeRejected {
        /// Human-readable rejection reason (static for now).
        reason: &'static str,
    },

    /// A downstream stage failed (dedupe / normalize). Reserved for T2+.
    #[error("pipeline stage `{stage}` failed: {message}")]
    StageFailed {
        /// Stage name (e.g. `dedupe`, `normalize`).
        stage: &'static str,
        /// Failure detail.
        message: String,
    },
}
