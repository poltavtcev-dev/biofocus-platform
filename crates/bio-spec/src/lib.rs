//! Canonical domain types and serialization contracts for BioFocus.
//!
//! Ubiquitous language (`docs/16-glossary.md`):
//! - [`Observation`] — immutable biometric/context fact
//! - [`Signal`] — transient change / anomaly
//! - [`Feature`] — windowed metric with provenance
//! - [`Insight`] — analytical conclusion with evidence
//! - [`Recommendation`] — calm suggested action with evidence (L4 / ADR-009)
//!
//! Identifiers intended for new Observations use **UUIDv7**.
//! Timestamps are **Unix seconds UTC** (`i64`).

#![forbid(unsafe_code)]

mod browser_category;
mod calendar_event;
mod error;
mod feature;
mod git_activity;
mod insight;
mod life_event;
mod now_playing;
mod observation;
mod recommendation;
mod signal;
mod time;

pub use browser_category::{
    is_v1_browser_category, validate_browser_category_payload, BROWSER_CATEGORY_COMMUNICATION,
    BROWSER_CATEGORY_ENTERTAINMENT, BROWSER_CATEGORY_REFERENCE, BROWSER_CATEGORY_SHOPPING,
    BROWSER_CATEGORY_UNKNOWN, BROWSER_CATEGORY_WORK, DATA_TYPE_BROWSER_CATEGORY,
    V1_BROWSER_CATEGORIES,
};
pub use calendar_event::{
    validate_calendar_event_payload, DATA_TYPE_CALENDAR_EVENT,
};
pub use error::{SpecError, SpecResult};
pub use feature::{ExplanationFactor, Feature, FeatureId, FeatureValue, Provenance};
pub use git_activity::{
    is_v1_activity_kind, validate_git_activity_payload, ACTIVITY_KIND_CHECKOUT,
    ACTIVITY_KIND_COMMIT, ACTIVITY_KIND_IDLE, ACTIVITY_KIND_OTHER, ACTIVITY_KIND_SYNC,
    ACTIVITY_KIND_UNKNOWN, DATA_TYPE_GIT_ACTIVITY, V1_ACTIVITY_KINDS,
};
pub use insight::{EvidenceRef, Insight, InsightId};
pub use life_event::{
    is_v1_life_event_kind, validate_life_event_payload, validate_observation_payload,
    DATA_TYPE_LIFE_EVENT, LIFE_EVENT_KIND_COFFEE, LIFE_EVENT_KIND_LUNCH, LIFE_EVENT_KIND_WALK,
    LIFE_EVENT_KIND_WORKOUT, V1_LIFE_EVENT_KINDS,
};
pub use now_playing::{
    is_v1_media_kind, validate_now_playing_payload, DATA_TYPE_NOW_PLAYING, MEDIA_KIND_MUSIC,
    MEDIA_KIND_NONE, MEDIA_KIND_OTHER, MEDIA_KIND_PODCAST, MEDIA_KIND_UNKNOWN, V1_MEDIA_KINDS,
};
pub use observation::{Confidence, DataType, Observation, ObservationId, ProviderId};
pub use recommendation::{Recommendation, RecommendationId};
pub use signal::{Severity, Signal, SignalId, SignalType};
pub use time::{TimeWindow, UnixTimestamp};

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "bio-spec";
