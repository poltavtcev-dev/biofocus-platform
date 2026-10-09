//! `active_energy` Observation payload (ADR-018 / P17-E2).

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for active energy Observations.
pub const DATA_TYPE_ACTIVE_ENERGY: &str = "active_energy";

/// Validates an `active_energy` payload object.
///
/// Required: `kcal` finite number ≥ 0.
pub fn validate_active_energy_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidActiveEnergyPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("kcal") {
        None => Err(SpecError::InvalidActiveEnergyPayload {
            reason: "missing required field kcal".to_owned(),
        }),
        Some(JsonValue::Number(n)) => {
            let kcal = n
                .as_f64()
                .ok_or_else(|| SpecError::InvalidActiveEnergyPayload {
                    reason: "kcal must be a finite number ≥ 0".to_owned(),
                })?;
            if !kcal.is_finite() || kcal < 0.0 {
                return Err(SpecError::InvalidActiveEnergyPayload {
                    reason: "kcal must be a finite number ≥ 0".to_owned(),
                });
            }
            Ok(())
        }
        Some(_) => Err(SpecError::InvalidActiveEnergyPayload {
            reason: "kcal must be a finite number ≥ 0".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_kcal() {
        validate_active_energy_payload(&json!({ "kcal": 185.5 })).expect("ok");
    }

    #[test]
    fn accepts_zero() {
        validate_active_energy_payload(&json!({ "kcal": 0 })).expect("ok");
    }

    #[test]
    fn rejects_missing() {
        assert!(matches!(
            validate_active_energy_payload(&json!({})),
            Err(SpecError::InvalidActiveEnergyPayload { .. })
        ));
    }

    #[test]
    fn rejects_negative() {
        assert!(matches!(
            validate_active_energy_payload(&json!({ "kcal": -1.0 })),
            Err(SpecError::InvalidActiveEnergyPayload { .. })
        ));
    }
}
