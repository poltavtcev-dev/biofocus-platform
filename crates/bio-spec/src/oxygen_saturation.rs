//! `oxygen_saturation` Observation payload (ADR-018 / P17-E2 soft-optional).

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for SpO2 Observations.
pub const DATA_TYPE_OXYGEN_SATURATION: &str = "oxygen_saturation";

/// Inclusive upper bound for `spo2_percent` (0–100).
pub const SPO2_PERCENT_MAX: f64 = 100.0;

/// Validates an `oxygen_saturation` payload object.
///
/// Required: `spo2_percent` finite number in **0–100**.
/// Soft-optional at Companion: emit only when HealthKit has samples — never invent.
pub fn validate_oxygen_saturation_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidOxygenSaturationPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("spo2_percent") {
        None => Err(SpecError::InvalidOxygenSaturationPayload {
            reason: "missing required field spo2_percent".to_owned(),
        }),
        Some(JsonValue::Number(n)) => {
            let pct = n.as_f64().ok_or_else(|| SpecError::InvalidOxygenSaturationPayload {
                reason: "spo2_percent must be a finite number 0–100".to_owned(),
            })?;
            if !pct.is_finite() || pct < 0.0 || pct > SPO2_PERCENT_MAX {
                return Err(SpecError::InvalidOxygenSaturationPayload {
                    reason: "spo2_percent must be a finite number 0–100".to_owned(),
                });
            }
            Ok(())
        }
        Some(_) => Err(SpecError::InvalidOxygenSaturationPayload {
            reason: "spo2_percent must be a finite number 0–100".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_percent() {
        validate_oxygen_saturation_payload(&json!({ "spo2_percent": 97 })).expect("ok");
    }

    #[test]
    fn rejects_above_100() {
        assert!(matches!(
            validate_oxygen_saturation_payload(&json!({ "spo2_percent": 101 })),
            Err(SpecError::InvalidOxygenSaturationPayload { .. })
        ));
    }

    #[test]
    fn rejects_missing() {
        assert!(matches!(
            validate_oxygen_saturation_payload(&json!({})),
            Err(SpecError::InvalidOxygenSaturationPayload { .. })
        ));
    }
}
