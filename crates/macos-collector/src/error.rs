//! Collector errors.

use thiserror::Error;

/// Result alias for collector operations.
pub type CollectorResult<T> = Result<T, CollectorError>;

/// Failures while probing OS or building Observations.
#[derive(Debug, Error)]
pub enum CollectorError {
    /// Frontmost-app probe failed.
    #[error("frontmost app probe failed: {0}")]
    Probe(String),
    /// Observation construction failed (e.g. invalid confidence).
    #[error(transparent)]
    Spec(#[from] bio_spec::SpecError),
}
