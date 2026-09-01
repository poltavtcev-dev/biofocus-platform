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

mod active_energy;
mod ambient_light;
mod browser_category;
mod calendar_event;
mod error;
mod feature;
mod git_activity;
mod insight;
mod life_event;
mod notification_event;
mod now_playing;
mod observation;
mod oxygen_saturation;
mod recommendation;
mod signal;
mod sleep_interval;
mod step_count;
mod time;

pub use active_energy::{validate_active_energy_payload, DATA_TYPE_ACTIVE_ENERGY};
pub use ambient_light::{
    is_v1_light_kind, validate_ambient_light_payload, DATA_TYPE_AMBIENT_LIGHT, LEVEL_MAX,
    LIGHT_KIND_BRIGHT, LIGHT_KIND_DARK, LIGHT_KIND_DIM, LIGHT_KIND_MODERATE, LIGHT_KIND_UNKNOWN,
    V1_LIGHT_KINDS,
};
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
pub use notification_event::{
    is_v1_interruption_level, is_v1_notification_app_kind, is_v1_notification_category,
    validate_notification_event_payload, DATA_TYPE_NOTIFICATION_EVENT, INTERRUPTION_LEVEL_ACTIVE,
    INTERRUPTION_LEVEL_CRITICAL, INTERRUPTION_LEVEL_PASSIVE, INTERRUPTION_LEVEL_TIME_SENSITIVE,
    INTERRUPTION_LEVEL_UNKNOWN, NOTIFICATION_APP_KIND_CALENDAR, NOTIFICATION_APP_KIND_MAIL,
    NOTIFICATION_APP_KIND_MESSAGING, NOTIFICATION_APP_KIND_OTHER, NOTIFICATION_APP_KIND_SOCIAL,
    NOTIFICATION_APP_KIND_SYSTEM, NOTIFICATION_APP_KIND_UNKNOWN, NOTIFICATION_CATEGORY_CALENDAR,
    NOTIFICATION_CATEGORY_COMMUNICATION, NOTIFICATION_CATEGORY_MEDIA, NOTIFICATION_CATEGORY_OTHER,
    NOTIFICATION_CATEGORY_SOCIAL, NOTIFICATION_CATEGORY_SYSTEM, NOTIFICATION_CATEGORY_UNKNOWN,
    V1_INTERRUPTION_LEVELS, V1_NOTIFICATION_APP_KINDS, V1_NOTIFICATION_CATEGORIES,
};
pub use now_playing::{
    is_v1_media_kind, validate_now_playing_payload, DATA_TYPE_NOW_PLAYING, MEDIA_KIND_MUSIC,
    MEDIA_KIND_NONE, MEDIA_KIND_OTHER, MEDIA_KIND_PODCAST, MEDIA_KIND_UNKNOWN, V1_MEDIA_KINDS,
};
pub use observation::{Confidence, DataType, Observation, ObservationId, ProviderId};
pub use oxygen_saturation::{
    validate_oxygen_saturation_payload, DATA_TYPE_OXYGEN_SATURATION, SPO2_PERCENT_MAX,
};
pub use recommendation::{Recommendation, RecommendationId};
pub use signal::{Severity, Signal, SignalId, SignalType};
pub use sleep_interval::{
    is_v1_sleep_stage, validate_sleep_interval_payload, DATA_TYPE_SLEEP_INTERVAL,
    SLEEP_STAGE_ASLEEP, SLEEP_STAGE_AWAKE, SLEEP_STAGE_IN_BED, SLEEP_STAGE_UNKNOWN, V1_SLEEP_STAGES,
};
pub use step_count::{validate_step_count_payload, DATA_TYPE_STEP_COUNT};
pub use time::{TimeWindow, UnixTimestamp};

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "bio-spec";
