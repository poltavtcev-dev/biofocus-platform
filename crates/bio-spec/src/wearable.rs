//! Wearables v2 payloads (ADR-030). Schema v1 is unchanged: new kinds live in
//! `payload`. Old payloads without `src` or `method` stay valid.

use std::collections::{HashMap, HashSet};

use serde_json::Value as JsonValue;

use crate::observation::Observation;
use crate::{SpecError, SpecResult};

/// Companion provider. `source_deletion` is accepted only from this id.
pub const PROVIDER_APPLE_HEALTH: &str = "com.biofocus.applehealth";

pub const DATA_TYPE_HEART_RATE: &str = "heart_rate";
pub const DATA_TYPE_RESTING_HEART_RATE: &str = "resting_heart_rate";
pub const DATA_TYPE_WALKING_HEART_RATE_AVERAGE: &str = "walking_heart_rate_average";
pub const DATA_TYPE_HRV: &str = "hrv";
pub const DATA_TYPE_RESPIRATORY_RATE: &str = "respiratory_rate";
pub const DATA_TYPE_SLEEPING_WRIST_TEMPERATURE: &str = "sleeping_wrist_temperature";
pub const DATA_TYPE_VO2_MAX: &str = "vo2_max";
pub const DATA_TYPE_WORKOUT: &str = "workout";
pub const DATA_TYPE_EXERCISE_TIME: &str = "exercise_time";
pub const DATA_TYPE_STAND_TIME: &str = "stand_time";
pub const DATA_TYPE_STAND_HOUR: &str = "stand_hour";
pub const DATA_TYPE_MINDFUL_SESSION: &str = "mindful_session";
pub const DATA_TYPE_DISTANCE_WALKING_RUNNING: &str = "distance_walking_running";
pub const DATA_TYPE_BASAL_ENERGY: &str = "basal_energy";
pub const DATA_TYPE_SOURCE_DELETION: &str = "source_deletion";

pub const SRC_KIND_APPLE_WATCH: &str = "apple_watch";
pub const SRC_KIND_IPHONE: &str = "iphone";
pub const SRC_KIND_XIAOMI_MI_FITNESS: &str = "xiaomi_mi_fitness";
pub const SRC_KIND_ZEPP_LIFE: &str = "zepp_life";
pub const SRC_KIND_OTHER_APP: &str = "other_app";
pub const SRC_KIND_MANUAL: &str = "manual";

const SRC_KINDS: &[&str] = &[
    SRC_KIND_APPLE_WATCH,
    SRC_KIND_IPHONE,
    SRC_KIND_XIAOMI_MI_FITNESS,
    SRC_KIND_ZEPP_LIFE,
    SRC_KIND_OTHER_APP,
    SRC_KIND_MANUAL,
];

/// Health types the phone may POST, plus `life_event` (not listed here).
const COMPANION_HEALTH_TYPES: &[&str] = &[
    DATA_TYPE_HEART_RATE,
    DATA_TYPE_RESTING_HEART_RATE,
    DATA_TYPE_WALKING_HEART_RATE_AVERAGE,
    DATA_TYPE_HRV,
    DATA_TYPE_RESPIRATORY_RATE,
    DATA_TYPE_SLEEPING_WRIST_TEMPERATURE,
    DATA_TYPE_VO2_MAX,
    crate::oxygen_saturation::DATA_TYPE_OXYGEN_SATURATION,
    crate::sleep_interval::DATA_TYPE_SLEEP_INTERVAL,
    DATA_TYPE_WORKOUT,
    DATA_TYPE_EXERCISE_TIME,
    DATA_TYPE_STAND_TIME,
    DATA_TYPE_STAND_HOUR,
    DATA_TYPE_MINDFUL_SESSION,
    DATA_TYPE_DISTANCE_WALKING_RUNNING,
    DATA_TYPE_BASAL_ENERGY,
    crate::step_count::DATA_TYPE_STEP_COUNT,
    crate::active_energy::DATA_TYPE_ACTIVE_ENERGY,
    crate::life_event::DATA_TYPE_LIFE_EVENT,
    DATA_TYPE_SOURCE_DELETION,
];

/// `true` when `kind` is a documented `src.kind`.
#[must_use]
pub fn is_src_kind(kind: &str) -> bool {
    SRC_KINDS.contains(&kind)
}

/// `true` when `POST /v1/ingest` may accept this `data_type`.
///
/// Mac-only types (`life_event_retraction`, collectors) are rejected so a
/// paired phone cannot cancel events or invent desktop Observations.
#[must_use]
pub fn companion_ingest_allowed(data_type: &str) -> bool {
    COMPANION_HEALTH_TYPES.contains(&data_type)
}

/// Validates a wearables v2 Observation. Unknown types are not this function's job.
pub fn validate_wearable_observation(obs: &Observation) -> SpecResult<()> {
    match obs.data_type.as_str() {
        DATA_TYPE_HEART_RATE | DATA_TYPE_WALKING_HEART_RATE_AVERAGE => validate_bpm(
            &obs.payload,
            &["bpm", "heart_rate", "hr", "beats_per_minute"],
        ),
        DATA_TYPE_RESTING_HEART_RATE => validate_bpm(&obs.payload, &["bpm", "resting_heart_rate"]),
        DATA_TYPE_HRV => validate_hrv(&obs.payload),
        DATA_TYPE_RESPIRATORY_RATE => {
            validate_bpm(&obs.payload, &["breaths_per_min", "respiratory_rate"])
        }
        DATA_TYPE_SLEEPING_WRIST_TEMPERATURE => {
            require_finite(&obs.payload, &["celsius", "delta_celsius"])
        }
        DATA_TYPE_VO2_MAX => require_finite(&obs.payload, &["ml_kg_min", "vo2_max"]),
        DATA_TYPE_DISTANCE_WALKING_RUNNING => {
            require_finite(&obs.payload, &["meters", "distance_m"])
        }
        DATA_TYPE_BASAL_ENERGY => require_finite(&obs.payload, &["kcal", "basal_energy_kcal"]),
        DATA_TYPE_EXERCISE_TIME | DATA_TYPE_STAND_TIME => {
            require_finite(&obs.payload, &["minutes"])
        }
        DATA_TYPE_STAND_HOUR | DATA_TYPE_MINDFUL_SESSION => validate_interval(&obs.payload),
        DATA_TYPE_WORKOUT => validate_workout(&obs.payload),
        DATA_TYPE_SOURCE_DELETION => validate_source_deletion(obs),
        other => Err(invalid(format!("not a wearable data_type `{other}`"))),
    }
}

/// Drops `source_deletion` markers and any Observation they target **with the
/// same `provider_id`**. A marker from another provider does not hide a row.
#[must_use]
pub fn apply_source_deletions(observations: Vec<Observation>) -> Vec<Observation> {
    let mut by_target: HashMap<uuid::Uuid, HashSet<String>> = HashMap::new();
    for obs in &observations {
        if obs.data_type != DATA_TYPE_SOURCE_DELETION {
            continue;
        }
        if let Some(id) = target_id(&obs.payload) {
            by_target
                .entry(id)
                .or_default()
                .insert(obs.provider_id.clone());
        }
    }
    observations
        .into_iter()
        .filter(|obs| {
            if obs.data_type == DATA_TYPE_SOURCE_DELETION {
                return false;
            }
            match by_target.get(&obs.id) {
                Some(providers) => !providers.contains(obs.provider_id.as_str()),
                None => true,
            }
        })
        .collect()
}

fn validate_source_deletion(obs: &Observation) -> SpecResult<()> {
    if obs.provider_id != PROVIDER_APPLE_HEALTH {
        return Err(invalid(
            "source_deletion is only valid from com.biofocus.applehealth".to_owned(),
        ));
    }
    if target_id(&obs.payload).is_none() {
        return Err(invalid(
            "source_deletion needs target_id (UUID string)".to_owned(),
        ));
    }
    Ok(())
}

fn validate_bpm(payload: &JsonValue, keys: &[&str]) -> SpecResult<()> {
    let obj = object(payload)?;
    if finite_from(obj, keys).is_none() {
        return Err(invalid(format!(
            "missing finite {}",
            keys.first().copied().unwrap_or("value")
        )));
    }
    validate_src_if_present(obj)
}

fn validate_hrv(payload: &JsonValue) -> SpecResult<()> {
    let obj = object(payload)?;
    let sdnn = finite_from(obj, &["sdnn_ms", "sdnn"]);
    let rmssd = finite_from(obj, &["rmssd_ms", "rmssd", "hrv_ms", "hrv"]);
    if sdnn.is_none() && rmssd.is_none() {
        return Err(invalid("hrv needs sdnn_ms and/or rmssd_ms".to_owned()));
    }
    if let Some(method) = obj.get("method") {
        let method = method
            .as_str()
            .ok_or_else(|| invalid("hrv.method must be a string".to_owned()))?;
        match method {
            "sdnn" if sdnn.is_some() && rmssd.is_none() => {}
            "rmssd" if rmssd.is_some() && sdnn.is_none() => {}
            "sdnn" | "rmssd" => {
                return Err(invalid(
                    "hrv.method must match the single metric present; sdnn and rmssd are not mixed"
                        .to_owned(),
                ));
            }
            other => {
                return Err(invalid(format!("unknown hrv.method `{other}`")));
            }
        }
    }
    validate_src_if_present(obj)
}

fn validate_interval(payload: &JsonValue) -> SpecResult<()> {
    let obj = object(payload)?;
    let start = i64_from(obj, "start").ok_or_else(|| invalid("missing start".to_owned()))?;
    let end = i64_from(obj, "end").ok_or_else(|| invalid("missing end".to_owned()))?;
    if end < start {
        return Err(invalid("end must be ≥ start".to_owned()));
    }
    validate_src_if_present(obj)
}

fn validate_workout(payload: &JsonValue) -> SpecResult<()> {
    let obj = object(payload)?;
    match obj.get("activity_type") {
        Some(JsonValue::String(s)) if !s.trim().is_empty() => {}
        _ => return Err(invalid("workout needs activity_type".to_owned())),
    }
    validate_interval(payload)
}

fn require_finite(payload: &JsonValue, keys: &[&str]) -> SpecResult<()> {
    let obj = object(payload)?;
    if finite_from(obj, keys).is_none() {
        return Err(invalid(format!(
            "missing finite {}",
            keys.first().copied().unwrap_or("value")
        )));
    }
    validate_src_if_present(obj)
}

pub(crate) fn validate_src_if_present(obj: &serde_json::Map<String, JsonValue>) -> SpecResult<()> {
    let Some(src) = obj.get("src") else {
        return Ok(());
    };
    let src = src
        .as_object()
        .ok_or_else(|| invalid("src must be an object".to_owned()))?;
    for banned in ["name", "device_name", "source_name"] {
        if src.contains_key(banned) {
            return Err(invalid(format!(
                "src.{banned} is not stored (it can contain a personal name)"
            )));
        }
    }
    match src.get("kind").and_then(JsonValue::as_str) {
        Some(kind) if is_src_kind(kind) => Ok(()),
        Some(kind) => Err(invalid(format!("unknown src.kind `{kind}`"))),
        None => Err(invalid(
            "src.kind is required when src is present".to_owned(),
        )),
    }
}

fn object(payload: &JsonValue) -> SpecResult<&serde_json::Map<String, JsonValue>> {
    payload
        .as_object()
        .ok_or_else(|| invalid("payload must be a JSON object".to_owned()))
}

fn finite_from(obj: &serde_json::Map<String, JsonValue>, keys: &[&str]) -> Option<f64> {
    for key in keys {
        if let Some(n) = obj.get(*key).and_then(JsonValue::as_f64) {
            if n.is_finite() {
                return Some(n);
            }
        }
    }
    None
}

fn i64_from(obj: &serde_json::Map<String, JsonValue>, key: &str) -> Option<i64> {
    obj.get(key).and_then(|v| match v {
        JsonValue::Number(n) => n.as_i64(),
        _ => None,
    })
}

fn target_id(payload: &JsonValue) -> Option<uuid::Uuid> {
    payload
        .get("target_id")
        .and_then(JsonValue::as_str)
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
}

fn invalid(reason: String) -> SpecError {
    SpecError::InvalidWearablePayload { reason }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UnixTimestamp;
    use serde_json::json;
    use uuid::Uuid;

    fn obs(provider: &str, data_type: &str, payload: JsonValue) -> Observation {
        Observation::try_new(
            Uuid::from_u128(7),
            UnixTimestamp::from_secs(1_700_000_000),
            provider,
            data_type,
            payload,
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn legacy_hr_and_hrv_stay_valid() {
        assert!(
            validate_wearable_observation(&obs(
                PROVIDER_APPLE_HEALTH,
                DATA_TYPE_HEART_RATE,
                json!({ "bpm": 70 })
            ))
            .is_ok()
        );
        assert!(
            validate_wearable_observation(&obs(
                PROVIDER_APPLE_HEALTH,
                DATA_TYPE_HRV,
                json!({ "rmssd_ms": 42 })
            ))
            .is_ok()
        );
    }

    #[test]
    fn hrv_method_rejects_mixed_metrics() {
        let mixed = obs(
            PROVIDER_APPLE_HEALTH,
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": 48, "rmssd_ms": 30 }),
        );
        assert!(validate_wearable_observation(&mixed).is_err());
        let sdnn = obs(
            PROVIDER_APPLE_HEALTH,
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": 48 }),
        );
        assert!(validate_wearable_observation(&sdnn).is_ok());
    }

    #[test]
    fn src_rejects_personal_name_keys() {
        let named = obs(
            PROVIDER_APPLE_HEALTH,
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 60, "src": { "kind": "apple_watch", "name": "Ada" } }),
        );
        assert!(validate_wearable_observation(&named).is_err());
    }

    #[test]
    fn source_deletion_requires_applehealth_provider() {
        let ok = obs(
            PROVIDER_APPLE_HEALTH,
            DATA_TYPE_SOURCE_DELETION,
            json!({ "target_id": Uuid::from_u128(1).to_string() }),
        );
        assert!(validate_wearable_observation(&ok).is_ok());
        let other = obs(
            "com.example.other",
            DATA_TYPE_SOURCE_DELETION,
            json!({ "target_id": Uuid::from_u128(1).to_string() }),
        );
        assert!(validate_wearable_observation(&other).is_err());
    }

    #[test]
    fn deletion_hides_only_the_same_provider() {
        let target = Uuid::from_u128(1);
        let rows = vec![
            obs_id(
                target,
                PROVIDER_APPLE_HEALTH,
                DATA_TYPE_HEART_RATE,
                json!({ "bpm": 60 }),
            ),
            obs_id(
                Uuid::from_u128(2),
                "com.example.other",
                DATA_TYPE_HEART_RATE,
                json!({ "bpm": 60 }),
            ),
            obs_id(
                Uuid::from_u128(8),
                "com.example.other",
                DATA_TYPE_SOURCE_DELETION,
                json!({ "target_id": target.to_string() }),
            ),
            obs_id(
                Uuid::from_u128(9),
                PROVIDER_APPLE_HEALTH,
                DATA_TYPE_SOURCE_DELETION,
                json!({ "target_id": target.to_string() }),
            ),
        ];
        let out = apply_source_deletions(rows);
        let ids: Vec<u128> = out.iter().map(|o| o.id.as_u128()).collect();
        assert_eq!(ids, vec![2]);
    }

    fn obs_id(id: Uuid, provider: &str, data_type: &str, payload: JsonValue) -> Observation {
        Observation::try_new(
            id,
            UnixTimestamp::from_secs(1_700_000_000),
            provider,
            data_type,
            payload,
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn companion_allowlist_blocks_mac_only_types() {
        assert!(companion_ingest_allowed(DATA_TYPE_HEART_RATE));
        assert!(companion_ingest_allowed(
            crate::life_event::DATA_TYPE_LIFE_EVENT
        ));
        assert!(!companion_ingest_allowed(
            crate::life_event::DATA_TYPE_LIFE_EVENT_RETRACTION
        ));
        assert!(!companion_ingest_allowed("context_window"));
        assert!(!companion_ingest_allowed("git_activity"));
    }
}
