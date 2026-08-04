//! Typed errors for domain validation in `bio-spec`.

use thiserror::Error;

/// Fallible operations that validate domain invariants.
pub type SpecResult<T> = Result<T, SpecError>;

/// Validation and contract errors for BioFocus domain types.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SpecError {
    /// `confidence` must be in the closed range `[0.0, 1.0]`.
    #[error("confidence must be between 0.0 and 1.0 inclusive, got {0}")]
    InvalidConfidence(f64),

    /// Time window end must be >= start.
    #[error("time window end ({end}) must be >= start ({start})")]
    InvalidTimeWindow { start: i64, end: i64 },
}
