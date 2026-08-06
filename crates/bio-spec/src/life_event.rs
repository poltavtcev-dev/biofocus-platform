//! Life Event Observation kinds (`docs/decision-log.md` ADR-006).
//!
//! Life Events are ordinary [`Observation`] values with `data_type == "life_event"`.
//! There is no parallel table or store — persistence uses the existing Observation
//! repository. Default local-only.

use serde_json::Value as JsonValue;

use crate::observation::Observation;
use crate::{SpecError, SpecResult};

/// Canonical `data_type` for all v1 Life Event Observations.
pub const DATA_TYPE_LIFE_EVENT: &str = "life_event";

/// v1 Life Event kind: coffee.
pub const LIFE_EVENT_KIND_COFFEE: &str = "coffee";
/// v1 Life Event kind: walk.
pub const LIFE_EVENT_KIND_WALK: &str = "walk";
/// v1 Life Event kind: lunch.
pub const LIFE_EVENT_KIND_LUNCH: &str = "lunch";
/// v1 Life Event kind: workout.
pub const LIFE_EVENT_KIND_WORKOUT: &str = "workout";

/// Allowed `payload.kind` values for v1 (`docs/07-contracts.md`).
pub const V1_LIFE_EVENT_KINDS: &[&str] = &[
    LIFE_EVENT_KIND_COFFEE,
    LIFE_EVENT_KIND_WALK,
    LIFE_EVENT_KIND_LUNCH,
    LIFE_EVENT_KIND_WORKOUT,
];

/// Returns `true` when `kind` is a documented v1 Life Event kind.
#[must_use]
pub fn is_v1_life_event_kind(kind: &str) -> bool {
    V1_LIFE_EVENT_KINDS.contains(&kind)
}

/// Validates ingest-time payload rules for an Observation.
///
/// - `data_type == "life_event"` → [`validate_life_event_payload`]
/// - `data_type == "calendar_event"` → [`crate::validate_calendar_event_payload`]
/// - other types → accepted (no extra payload schema at this layer)
pub fn validate_observation_payload(obs: &Observation) -> SpecResult<()> {
    match obs.data_type.as_str() {
        DATA_TYPE_LIFE_EVENT => validate_life_event_payload(&obs.payload),
        crate::calendar_event::DATA_TYPE_CALENDAR_EVENT => {
            crate::calendar_event::validate_calendar_event_payload(&obs.payload)
        }
        _ => Ok(()),
    }
}

/// Validates a Life Event `payload` object.
///
/// Required: `kind` ∈ v1 set (`coffee` / `walk` / `lunch` / `workout`).
/// Optional: `note` (string), `duration_secs` (finite number ≥ 0).
/// Unknown keys are allowed (forward-compatible).
pub fn validate_life_event_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload.as_object().ok_or_else(|| SpecError::InvalidLifeEventPayload {
        reason: "payload must be a JSON object".to_owned(),
    })?;

    let kind = match obj.get("kind") {
        None => {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "missing required field kind".to_owned(),
            });
        }
        Some(JsonValue::String(s)) => s.as_str(),
        Some(_) => {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "kind must be a string".to_owned(),
            });
        }
    };

    if !is_v1_life_event_kind(kind) {
        return Err(SpecError::InvalidLifeEventPayload {
            reason: format!(
                "unknown life event kind `{kind}`; expected one of: {}",
                V1_LIFE_EVENT_KINDS.join(", ")
            ),
        });
    }

    if let Some(note) = obj.get("note") {
        if !note.is_string() {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "note must be a string when present".to_owned(),
            });
        }
    }

    if let Some(duration) = obj.get("duration_secs") {
        let Some(secs) = duration.as_f64() else {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "duration_secs must be a number when present".to_owned(),
            });
        };
        if !secs.is_finite() || secs < 0.0 {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "duration_secs must be a finite number >= 0".to_owned(),
            });
        }
    }

    Ok(())
}
