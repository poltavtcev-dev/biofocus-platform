//! Typed errors for the Core runtime host.

use thiserror::Error;

/// Fallible runtime bootstrap operations.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Errors raised while building or configuring the Core runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Tokio multi-thread runtime failed to build.
    #[error("failed to build Tokio runtime: {0}")]
    RuntimeBuild(#[source] std::io::Error),

    /// Observation channel capacity must be > 0.
    #[error("observation channel capacity must be greater than 0")]
    InvalidChannelCapacity,

    /// Tracing subscriber could not be installed.
    #[error("failed to initialize tracing subscriber: {0}")]
    TracingInit(String),

    /// Feature Worker poll interval must be > 0.
    #[error("feature worker poll interval must be greater than 0")]
    InvalidFeaturePollInterval,

    /// Observation source for the Feature Worker failed.
    #[error("feature worker observation source: {0}")]
    FeatureWorkerSource(String),
}
