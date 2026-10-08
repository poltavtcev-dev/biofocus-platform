//! `step_count` Observation payload (ADR-018 / P17-E2).

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for step count Observations.
pub const DATA_TYPE_STEP_COUNT: &str = "step_count";

/// Validates a `step_count` payload object.
///
/// Required: `count` non-negative integer.
/// Optional: `window_secs` integer ≥ 1.
pub fn validate_step_count_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidStepCountPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    let count = match obj.get("count") {
        None => {
            return Err(SpecError::InvalidStepCountPayload {
                reason: "missing required field count".to_owned(),
            });
        }
        Some(v) => nonneg_u64(v).ok_or_else(|| SpecError::InvalidStepCountPayload {
            reason: "count must be a non-negative integer".to_owned(),
        })?,
    };
    let _ = count;

    if let Some(v) = obj.get("window_secs") {
        let window = nonneg_u64(v).ok_or_else(|| SpecError::InvalidStepCountPayload {
            reason: "window_secs must be an integer ≥ 1".to_owned(),
        })?;
        if window < 1 {
            return Err(SpecError::InvalidStepCountPayload {
                reason: "window_secs must be an integer ≥ 1".to_owned(),
            });
        }
    }

    Ok(())
}

fn nonneg_u64(v: &JsonValue) -> Option<u64> {
    match v {
        JsonValue::Number(n) => n.as_u64().or_else(|| {
            n.as_i64().and_then(|i| u64::try_from(i).ok()).or_else(|| {
                n.as_f64()
                    .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                    .map(|f| f as u64)
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
    fn accepts_count_only() {
        validate_step_count_payload(&json!({ "count": 120 })).expect("ok");
    }

    #[test]
    fn accepts_with_window() {
        validate_step_count_payload(&json!({ "count": 2400, "window_secs": 3600 })).expect("ok");
    }

    #[test]
    fn rejects_missing_count() {
        assert!(matches!(
            validate_step_count_payload(&json!({})),
            Err(SpecError::InvalidStepCountPayload { .. })
        ));
    }

    #[test]
    fn rejects_negative_count() {
        assert!(matches!(
            validate_step_count_payload(&json!({ "count": -1 })),
            Err(SpecError::InvalidStepCountPayload { .. })
        ));
    }

    #[test]
    fn rejects_zero_window() {
        assert!(matches!(
            validate_step_count_payload(&json!({ "count": 10, "window_secs": 0 })),
            Err(SpecError::InvalidStepCountPayload { .. })
        ));
    }
}
