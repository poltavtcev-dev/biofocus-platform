//! Ambient light Observation kinds (ADR-015 / P16-E1 shipped).
//!
//! Ambient light facts are ordinary [`Observation`] values with
//! `data_type == "ambient_light"`. Persistence uses the existing Observation
//! repository. Coarse `light_kind` + optional bounded `level` (0–100) only —
//! no camera frames, screen contents, precise geo, or mic. Opt-in local
//! collector; personal self-tracking only.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for ambient light Observations.
pub const DATA_TYPE_AMBIENT_LIGHT: &str = "ambient_light";

/// v1 light kind: dark.
pub const LIGHT_KIND_DARK: &str = "dark";
/// v1 light kind: dim.
pub const LIGHT_KIND_DIM: &str = "dim";
/// v1 light kind: moderate.
pub const LIGHT_KIND_MODERATE: &str = "moderate";
/// v1 light kind: bright.
pub const LIGHT_KIND_BRIGHT: &str = "bright";
/// v1 light kind: unknown (soft-fail mapping).
pub const LIGHT_KIND_UNKNOWN: &str = "unknown";

/// Allowed `payload.light_kind` values for v1 (`docs/07-contracts.md` / ADR-015).
pub const V1_LIGHT_KINDS: &[&str] = &[
    LIGHT_KIND_DARK,
    LIGHT_KIND_DIM,
    LIGHT_KIND_MODERATE,
    LIGHT_KIND_BRIGHT,
    LIGHT_KIND_UNKNOWN,
];

/// Inclusive upper bound for optional relative `level` (0–100).
pub const LEVEL_MAX: u64 = 100;

/// Returns `true` when `light_kind` is a documented v1 ambient light kind.
#[must_use]
pub fn is_v1_light_kind(light_kind: &str) -> bool {
    V1_LIGHT_KINDS.contains(&light_kind)
}

/// Validates an ambient light `payload` object.
///
/// Required: `light_kind` ∈ v1 closed set.
/// Optional: `level` integer in **0–100** (relative brightness band).
/// Unknown keys are allowed (forward-compatible) but collectors must not emit
/// camera frames, screen contents, precise geo, or mic/waveform — pipeline
/// normalize strips those extras.
pub fn validate_ambient_light_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidAmbientLightPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("light_kind") {
        None => {
            return Err(SpecError::InvalidAmbientLightPayload {
                reason: "missing required field light_kind".to_owned(),
            });
        }
        Some(JsonValue::String(s)) if is_v1_light_kind(s) => {}
        Some(JsonValue::String(s)) => {
            return Err(SpecError::InvalidAmbientLightPayload {
                reason: format!("light_kind `{s}` is not a v1 ambient light kind"),
            });
        }
        Some(_) => {
            return Err(SpecError::InvalidAmbientLightPayload {
                reason: "light_kind must be a string".to_owned(),
            });
        }
    }

    if let Some(level_val) = obj.get("level") {
        let level = match level_val {
            JsonValue::Number(n) => n
                .as_u64()
                .or_else(|| {
                    n.as_i64().and_then(|i| u64::try_from(i).ok()).or_else(|| {
                        n.as_f64()
                            .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                            .map(|f| f as u64)
                    })
                })
                .ok_or_else(|| SpecError::InvalidAmbientLightPayload {
                    reason: "level must be an integer 0–100".to_owned(),
                })?,
            _ => {
                return Err(SpecError::InvalidAmbientLightPayload {
                    reason: "level must be an integer 0–100".to_owned(),
                });
            }
        };
        if level > LEVEL_MAX {
            return Err(SpecError::InvalidAmbientLightPayload {
                reason: "level must be ≤ 100".to_owned(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_dim_with_level() {
        validate_ambient_light_payload(&json!({
            "light_kind": "dim",
            "level": 25
        }))
        .expect("valid");
    }

    #[test]
    fn accepts_kind_only() {
        validate_ambient_light_payload(&json!({
            "light_kind": "bright"
        }))
        .expect("valid");
    }

    #[test]
    fn accepts_unknown() {
        validate_ambient_light_payload(&json!({
            "light_kind": "unknown"
        }))
        .expect("valid");
    }

    #[test]
    fn rejects_unknown_kind() {
        let err = validate_ambient_light_payload(&json!({
            "light_kind": "glaring",
            "level": 10
        }))
        .expect_err("not in v1 set");
        assert!(matches!(err, SpecError::InvalidAmbientLightPayload { .. }));
    }

    #[test]
    fn rejects_level_out_of_range() {
        let err = validate_ambient_light_payload(&json!({
            "light_kind": "dim",
            "level": 101
        }))
        .expect_err("level max");
        assert!(matches!(err, SpecError::InvalidAmbientLightPayload { .. }));
    }

    #[test]
    fn rejects_missing_kind() {
        assert!(matches!(
            validate_ambient_light_payload(&json!({ "level": 10 })),
            Err(SpecError::InvalidAmbientLightPayload { .. })
        ));
    }
}
