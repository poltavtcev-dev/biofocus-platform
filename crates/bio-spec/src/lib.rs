//! Canonical domain types and serialization contracts for BioFocus.
//!
//! Ubiquitous language (`docs/16-glossary.md`):
//! - [`Observation`] — immutable biometric/context fact
//! - [`Signal`] — transient change / anomaly
//! - [`Feature`] — windowed metric with provenance
//! - [`Insight`] — analytical conclusion with evidence
//!
//! Identifiers intended for new Observations use **UUIDv7**.
//! Timestamps are **Unix seconds UTC** (`i64`).

#![forbid(unsafe_code)]

mod calendar_event;
mod error;
mod feature;
mod insight;
mod life_event;
mod observation;
mod signal;
mod time;

pub use calendar_event::{
    validate_calendar_event_payload, DATA_TYPE_CALENDAR_EVENT,
};
pub use error::{SpecError, SpecResult};
pub use feature::{Feature, FeatureId, FeatureValue, Provenance};
pub use insight::{EvidenceRef, Insight, InsightId};
pub use life_event::{
    is_v1_life_event_kind, validate_life_event_payload, validate_observation_payload,
    DATA_TYPE_LIFE_EVENT, LIFE_EVENT_KIND_COFFEE, LIFE_EVENT_KIND_LUNCH, LIFE_EVENT_KIND_WALK,
    LIFE_EVENT_KIND_WORKOUT, V1_LIFE_EVENT_KINDS,
};
pub use observation::{Confidence, DataType, Observation, ObservationId, ProviderId};
pub use signal::{Severity, Signal, SignalId, SignalType};
pub use time::{TimeWindow, UnixTimestamp};

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "bio-spec";
