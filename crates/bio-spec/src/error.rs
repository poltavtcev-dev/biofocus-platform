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

    /// Life Event Observation payload failed contract validation (ADR-006).
    #[error("invalid life event payload: {reason}")]
    InvalidLifeEventPayload { reason: String },

    /// Calendar Event Observation payload failed contract validation (P6-E3-T1).
    #[error("invalid calendar event payload: {reason}")]
    InvalidCalendarEventPayload { reason: String },

    /// Browser category Observation payload failed contract validation (ADR-010).
    #[error("invalid browser category payload: {reason}")]
    InvalidBrowserCategoryPayload { reason: String },

    /// Now Playing Observation payload failed contract validation (ADR-012).
    #[error("invalid now playing payload: {reason}")]
    InvalidNowPlayingPayload { reason: String },

    /// Git activity Observation payload failed contract validation (ADR-013).
    #[error("invalid git activity payload: {reason}")]
    InvalidGitActivityPayload { reason: String },

    /// Ambient light Observation payload failed contract validation (ADR-015).
    #[error("invalid ambient light payload: {reason}")]
    InvalidAmbientLightPayload { reason: String },

    /// Notification event Observation payload failed contract validation (ADR-019).
    #[error("invalid notification_event payload: {reason}")]
    InvalidNotificationEventPayload { reason: String },

    /// Step count Observation payload failed contract validation (ADR-018).
    #[error("invalid step_count payload: {reason}")]
    InvalidStepCountPayload { reason: String },

    /// Active energy Observation payload failed contract validation (ADR-018).
    #[error("invalid active_energy payload: {reason}")]
    InvalidActiveEnergyPayload { reason: String },

    /// Sleep interval Observation payload failed contract validation (ADR-018).
    #[error("invalid sleep_interval payload: {reason}")]
    InvalidSleepIntervalPayload { reason: String },

    /// Oxygen saturation Observation payload failed contract validation (ADR-018).
    #[error("invalid oxygen_saturation payload: {reason}")]
    InvalidOxygenSaturationPayload { reason: String },
}
