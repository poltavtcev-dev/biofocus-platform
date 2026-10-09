//! `sleep_interval` Observation payload (ADR-018 / P17-E2).

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for sleep interval Observations.
pub const DATA_TYPE_SLEEP_INTERVAL: &str = "sleep_interval";

/// Sleep stage: asleep.
pub const SLEEP_STAGE_ASLEEP: &str = "asleep";
/// Sleep stage: in bed.
pub const SLEEP_STAGE_IN_BED: &str = "in_bed";
/// Sleep stage: awake.
pub const SLEEP_STAGE_AWAKE: &str = "awake";
/// Sleep stage: unknown.
pub const SLEEP_STAGE_UNKNOWN: &str = "unknown";
/// Sleep stage: light / core sleep (ADR-030). v1 `asleep` stays valid.
pub const SLEEP_STAGE_ASLEEP_CORE: &str = "asleep_core";
/// Sleep stage: deep sleep (ADR-030).
pub const SLEEP_STAGE_ASLEEP_DEEP: &str = "asleep_deep";
/// Sleep stage: REM sleep (ADR-030).
pub const SLEEP_STAGE_ASLEEP_REM: &str = "asleep_rem";

/// Allowed optional `payload.stage` values (ADR-018, stages added by ADR-030).
pub const V1_SLEEP_STAGES: &[&str] = &[
    SLEEP_STAGE_ASLEEP,
    SLEEP_STAGE_IN_BED,
    SLEEP_STAGE_AWAKE,
    SLEEP_STAGE_UNKNOWN,
    SLEEP_STAGE_ASLEEP_CORE,
    SLEEP_STAGE_ASLEEP_DEEP,
    SLEEP_STAGE_ASLEEP_REM,
];

/// Returns `true` when `stage` is a documented v1 sleep stage.
#[must_use]
pub fn is_v1_sleep_stage(stage: &str) -> bool {
    V1_SLEEP_STAGES.contains(&stage)
}

/// Validates a `sleep_interval` payload object.
///
/// Required: `start` / `end` Unix seconds with `end` ≥ `start`.
/// Optional: `stage` ∈ closed set.
pub fn validate_sleep_interval_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidSleepIntervalPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    let start = required_i64(obj, "start")?;
    let end = required_i64(obj, "end")?;
    if end < start {
        return Err(SpecError::InvalidSleepIntervalPayload {
            reason: "end must be ≥ start".to_owned(),
        });
    }

    if let Some(stage_val) = obj.get("stage") {
        match stage_val {
            JsonValue::String(s) if is_v1_sleep_stage(s) => {}
            JsonValue::String(s) => {
                return Err(SpecError::InvalidSleepIntervalPayload {
                    reason: format!("stage `{s}` is not a v1 sleep stage"),
                });
            }
            _ => {
                return Err(SpecError::InvalidSleepIntervalPayload {
                    reason: "stage must be a string".to_owned(),
                });
            }
        }
    }

    Ok(())
}

fn required_i64(obj: &serde_json::Map<String, JsonValue>, key: &str) -> SpecResult<i64> {
    match obj.get(key) {
        None => Err(SpecError::InvalidSleepIntervalPayload {
            reason: format!("missing required field {key}"),
        }),
        Some(v) => json_as_i64(v).ok_or_else(|| SpecError::InvalidSleepIntervalPayload {
            reason: format!("{key} must be a Unix timestamp integer"),
        }),
    }
}

fn json_as_i64(v: &JsonValue) -> Option<i64> {
    match v {
        JsonValue::Number(n) => n.as_i64().or_else(|| {
            n.as_u64().and_then(|u| i64::try_from(u).ok()).or_else(|| {
                n.as_f64()
                    .filter(|f| f.is_finite() && f.fract() == 0.0)
                    .map(|f| f as i64)
            })
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_interval_without_stage() {
        validate_sleep_interval_payload(&json!({
            "start": 100,
            "end": 200
        }))
        .expect("ok");
    }

    #[test]
    fn accepts_with_stage() {
        validate_sleep_interval_payload(&json!({
            "start": 100,
            "end": 200,
            "stage": "asleep"
        }))
        .expect("ok");
    }

    #[test]
    fn rejects_end_before_start() {
        assert!(matches!(
            validate_sleep_interval_payload(&json!({ "start": 200, "end": 100 })),
            Err(SpecError::InvalidSleepIntervalPayload { .. })
        ));
    }

    #[test]
    fn rejects_bad_stage() {
        assert!(matches!(
            validate_sleep_interval_payload(&json!({
                "start": 100,
                "end": 200,
                "stage": "rem"
            })),
            Err(SpecError::InvalidSleepIntervalPayload { .. })
        ));
    }

    #[test]
    fn rejects_missing_start() {
        assert!(matches!(
            validate_sleep_interval_payload(&json!({ "end": 200 })),
            Err(SpecError::InvalidSleepIntervalPayload { .. })
        ));
    }
}
