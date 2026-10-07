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

/// Append-only "undo" marker for a Life Event. Payload: `{ "target_id": "<uuid>" }`.
///
/// The original Observation row is never mutated or deleted; readers drop both
/// the target and the marker (see [`apply_life_event_retractions`]).
pub const DATA_TYPE_LIFE_EVENT_RETRACTION: &str = "life_event_retraction";

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
/// - `data_type == "browser_category"` → [`crate::validate_browser_category_payload`]
/// - `data_type == "now_playing"` → [`crate::validate_now_playing_payload`]
/// - `data_type == "git_activity"` → [`crate::validate_git_activity_payload`]
/// - other types → accepted (no extra payload schema at this layer)
pub fn validate_observation_payload(obs: &Observation) -> SpecResult<()> {
    match obs.data_type.as_str() {
        DATA_TYPE_LIFE_EVENT => validate_life_event_payload(&obs.payload),
        DATA_TYPE_LIFE_EVENT_RETRACTION => validate_life_event_retraction_payload(&obs.payload),
        crate::calendar_event::DATA_TYPE_CALENDAR_EVENT => {
            crate::calendar_event::validate_calendar_event_payload(&obs.payload)
        }
        crate::browser_category::DATA_TYPE_BROWSER_CATEGORY => {
            crate::browser_category::validate_browser_category_payload(&obs.payload)
        }
        crate::now_playing::DATA_TYPE_NOW_PLAYING => {
            crate::now_playing::validate_now_playing_payload(&obs.payload)
        }
        crate::git_activity::DATA_TYPE_GIT_ACTIVITY => {
            crate::git_activity::validate_git_activity_payload(&obs.payload)
        }
        crate::ambient_light::DATA_TYPE_AMBIENT_LIGHT => {
            crate::ambient_light::validate_ambient_light_payload(&obs.payload)
        }
        crate::notification_event::DATA_TYPE_NOTIFICATION_EVENT => {
            crate::notification_event::validate_notification_event_payload(&obs.payload)
        }
        crate::step_count::DATA_TYPE_STEP_COUNT => {
            crate::step_count::validate_step_count_payload(&obs.payload)
        }
        crate::active_energy::DATA_TYPE_ACTIVE_ENERGY => {
            crate::active_energy::validate_active_energy_payload(&obs.payload)
        }
        crate::sleep_interval::DATA_TYPE_SLEEP_INTERVAL => {
            crate::sleep_interval::validate_sleep_interval_payload(&obs.payload)
        }
        crate::oxygen_saturation::DATA_TYPE_OXYGEN_SATURATION => {
            crate::oxygen_saturation::validate_oxygen_saturation_payload(&obs.payload)
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

    // When the user back-dates an event, `Observation.timestamp` is "happened at"
    // and `logged_at` keeps the moment it was recorded (Unix secs).
    for key in ["logged_at", "edited_from_logged_at"] {
        if let Some(v) = obj.get(key) {
            if !v.as_i64().is_some_and(|n| n >= 0) {
                return Err(SpecError::InvalidLifeEventPayload {
                    reason: format!("{key} must be a non-negative integer when present"),
                });
            }
        }
    }
    if let Some(v) = obj.get("edited_from") {
        if !v.as_str().is_some_and(|s| uuid::Uuid::parse_str(s).is_ok()) {
            return Err(SpecError::InvalidLifeEventPayload {
                reason: "edited_from must be a UUID string when present".to_owned(),
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

/// Validates a [`DATA_TYPE_LIFE_EVENT_RETRACTION`] payload (`target_id` UUID).
pub fn validate_life_event_retraction_payload(payload: &JsonValue) -> SpecResult<()> {
    retraction_target(payload).map(|_| ()).ok_or_else(|| SpecError::InvalidLifeEventPayload {
        reason: "retraction needs target_id (UUID string)".to_owned(),
    })
}

fn retraction_target(payload: &JsonValue) -> Option<uuid::Uuid> {
    payload
        .get("target_id")
        .and_then(JsonValue::as_str)
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
}

/// Removes retracted Life Events **and** the retraction markers themselves.
///
/// Pure, order-independent; use wherever Observations are read for Features,
/// Insights, series or reports so an undone event is excluded everywhere.
#[must_use]
pub fn apply_life_event_retractions(observations: Vec<Observation>) -> Vec<Observation> {
    let retracted: std::collections::HashSet<uuid::Uuid> = observations
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_LIFE_EVENT_RETRACTION)
        .filter_map(|o| retraction_target(&o.payload))
        .collect();
    observations
        .into_iter()
        .filter(|o| o.data_type != DATA_TYPE_LIFE_EVENT_RETRACTION && !retracted.contains(&o.id))
        .collect()
}

/// Target id if `obs` is a valid retraction marker.
#[must_use]
pub fn life_event_retraction_target(obs: &Observation) -> Option<uuid::Uuid> {
    (obs.data_type == DATA_TYPE_LIFE_EVENT_RETRACTION)
        .then(|| retraction_target(&obs.payload))
        .flatten()
}

#[cfg(test)]
mod retraction_tests {
    use super::*;
    use crate::UnixTimestamp;
    use serde_json::json;
    use uuid::Uuid;

    fn obs(id: u128, data_type: &str, payload: JsonValue) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(1_000 + id as i64),
            "test",
            data_type,
            payload,
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn retraction_drops_target_and_marker_only() {
        let target = Uuid::from_u128(1);
        let rows = vec![
            obs(1, DATA_TYPE_LIFE_EVENT, json!({ "kind": "coffee" })),
            obs(2, DATA_TYPE_LIFE_EVENT, json!({ "kind": "walk" })),
            obs(3, "hrv", json!({ "rmssd_ms": 40.0 })),
            obs(4, DATA_TYPE_LIFE_EVENT_RETRACTION, json!({ "target_id": target.to_string() })),
        ];
        let out = apply_life_event_retractions(rows);
        let ids: Vec<u128> = out.iter().map(|o| o.id.as_u128()).collect();
        assert_eq!(ids, vec![2, 3]);
    }

    #[test]
    fn retraction_before_target_in_list_still_applies() {
        let rows = vec![
            obs(9, DATA_TYPE_LIFE_EVENT_RETRACTION, json!({ "target_id": Uuid::from_u128(10).to_string() })),
            obs(10, DATA_TYPE_LIFE_EVENT, json!({ "kind": "lunch" })),
        ];
        assert!(apply_life_event_retractions(rows).is_empty());
    }

    #[test]
    fn retraction_payload_validation() {
        assert!(validate_life_event_retraction_payload(&json!({ "target_id": Uuid::from_u128(1).to_string() })).is_ok());
        assert!(validate_life_event_retraction_payload(&json!({ "target_id": "nope" })).is_err());
        assert!(validate_life_event_retraction_payload(&json!({})).is_err());
    }

    #[test]
    fn life_event_accepts_logged_at_and_edited_from() {
        let ok = json!({ "kind": "walk", "logged_at": 1700000000, "edited_from": Uuid::from_u128(5).to_string(), "edited_from_logged_at": 1699999000 });
        assert!(validate_life_event_payload(&ok).is_ok());
        assert!(validate_life_event_payload(&json!({ "kind": "walk", "logged_at": "x" })).is_err());
        assert!(validate_life_event_payload(&json!({ "kind": "walk", "edited_from": "x" })).is_err());
    }
}
