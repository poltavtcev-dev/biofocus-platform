//! Calendar / meeting Observation kinds (P6-E3-T1 dogfood).
//!
//! Calendar events are ordinary [`Observation`] values with
//! `data_type == "calendar_event"`. Persistence uses the existing Observation
//! repository. Local-only ICS / fixture sources — no cloud OAuth.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for Calendar / meeting Observations.
pub const DATA_TYPE_CALENDAR_EVENT: &str = "calendar_event";

/// Validates a Calendar Event `payload` object.
///
/// Required: `uid` (non-empty string), `start` / `end` (i64 unix secs, `end >= start`).
/// Optional: `all_day` (bool), `busy` (bool).
/// Titles / bodies / attendees must **not** be required; unknown keys allowed
/// (forward-compatible) but collectors must not emit them.
pub fn validate_calendar_event_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidCalendarEventPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("uid") {
        None => {
            return Err(SpecError::InvalidCalendarEventPayload {
                reason: "missing required field uid".to_owned(),
            });
        }
        Some(JsonValue::String(s)) if !s.trim().is_empty() => {}
        Some(JsonValue::String(_)) => {
            return Err(SpecError::InvalidCalendarEventPayload {
                reason: "uid must be a non-empty string".to_owned(),
            });
        }
        Some(_) => {
            return Err(SpecError::InvalidCalendarEventPayload {
                reason: "uid must be a string".to_owned(),
            });
        }
    }

    let start = required_i64(obj, "start")?;
    let end = required_i64(obj, "end")?;
    if end < start {
        return Err(SpecError::InvalidCalendarEventPayload {
            reason: format!("end ({end}) must be >= start ({start})"),
        });
    }

    if let Some(all_day) = obj.get("all_day") {
        if !all_day.is_boolean() {
            return Err(SpecError::InvalidCalendarEventPayload {
                reason: "all_day must be a boolean when present".to_owned(),
            });
        }
    }

    if let Some(busy) = obj.get("busy") {
        if !busy.is_boolean() {
            return Err(SpecError::InvalidCalendarEventPayload {
                reason: "busy must be a boolean when present".to_owned(),
            });
        }
    }

    Ok(())
}

fn required_i64(
    obj: &serde_json::Map<String, JsonValue>,
    key: &str,
) -> SpecResult<i64> {
    match obj.get(key) {
        None => Err(SpecError::InvalidCalendarEventPayload {
            reason: format!("missing required field {key}"),
        }),
        Some(JsonValue::Number(n)) => n.as_i64().ok_or_else(|| {
            SpecError::InvalidCalendarEventPayload {
                reason: format!("{key} must be an integer unix timestamp"),
            }
        }),
        Some(_) => Err(SpecError::InvalidCalendarEventPayload {
            reason: format!("{key} must be an integer unix timestamp"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_minimal_busy_meeting() {
        validate_calendar_event_payload(&json!({
            "uid": "meet-1",
            "start": 1_721_990_400,
            "end": 1_721_994_000,
            "busy": true,
            "all_day": false
        }))
        .expect("valid");
    }

    #[test]
    fn rejects_end_before_start() {
        let err = validate_calendar_event_payload(&json!({
            "uid": "x",
            "start": 100,
            "end": 99
        }))
        .expect_err("end < start");
        assert!(matches!(err, SpecError::InvalidCalendarEventPayload { .. }));
    }

    #[test]
    fn rejects_empty_uid() {
        let err = validate_calendar_event_payload(&json!({
            "uid": "  ",
            "start": 1,
            "end": 2
        }))
        .expect_err("empty uid");
        assert!(matches!(err, SpecError::InvalidCalendarEventPayload { .. }));
    }
}
