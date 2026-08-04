//! Normalization & calibration stage — canonical payloads / units for Feature Engine.
//!
//! # Contract (P3-E1-T3)
//!
//! Sync, in-memory only. Does **not** rewrite SQLite rows. Output remains
//! [`Observation`] values with rewritten `payload` JSON (still Observations, not
//! Features).
//!
//! ## Known `data_type` rules
//!
//! | `data_type` | Canonical payload | Calibration / aliases |
//! | :--- | :--- | :--- |
//! | `heart_rate` | `{ "bpm": f64, "source"?: string }` | bpm from `bpm` / `heart_rate` / `hr` / `beats_per_minute`; if `unit` is `hz` → ×60; optional `source` kept |
//! | `hrv` | `{ "rmssd_ms": f64, "sdnn_ms"?: f64, "pnn50"?: f64 }` | RMSSD from `rmssd_ms` / `rmssd` / `hrv_ms` / `hrv`; if `unit` is `s` → ×1000; optional SDNN / pNN50 |
//! | `context_window` | `{ "bundle_id": string, "app_name": string }` | aliases `bundleId` / `appName` / `name`; other keys stripped |
//! | `keystrokes` | `{ "count": u64, "window_secs": u64, "rate_per_min": f64 }` | `window_seconds`→`window_secs`; `rate_per_min` recomputed; content keys stripped |
//!
//! Known type with missing / non-finite required fields → **skipped** (dropped from
//! the batch; counted in [`NormalizedBatch::skipped_count`]).
//!
//! ## Unknown `data_type`
//!
//! **Pass-through** — Observation kept unchanged (forward-compatible). Not counted
//! as skipped.

use bio_spec::Observation;
use serde_json::{json, Map, Value as JsonValue};

use crate::dedupe::DedupedBatch;
use crate::{PipelineResult, PipelineStage};

/// Canonical storage / schema `data_type` values handled by this stage.
pub const DATA_TYPE_HEART_RATE: &str = "heart_rate";
/// HRV aggregates (RMSSD / SDNN / pNN50), milliseconds where applicable.
pub const DATA_TYPE_HRV: &str = "hrv";
/// Active-window context (app identity only).
pub const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";
/// Keystroke input aggregates (counts / rates only).
pub const DATA_TYPE_KEYSTROKES: &str = "keystrokes";

/// Batch after normalization (`PipelineStage::Normalized`).
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedBatch {
    observations: Vec<Observation>,
    skipped_count: usize,
    stage: PipelineStage,
}

impl NormalizedBatch {
    /// Pipeline stage marker after a successful normalize call.
    #[must_use]
    pub const fn stage(&self) -> PipelineStage {
        self.stage
    }

    /// Kept Observations (normalized known types + pass-through unknowns).
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Number of kept Observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// `true` when no Observations remain (valid idle / all-skipped outcome).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Known-type Observations dropped because required fields were missing / invalid.
    #[must_use]
    pub const fn skipped_count(&self) -> usize {
        self.skipped_count
    }

    /// Consume the batch and return owned Observations.
    #[must_use]
    pub fn into_observations(self) -> Vec<Observation> {
        self.observations
    }
}

/// Normalize a slice of Observations (payload / units → canon).
///
/// # Contract
/// - **Empty batch** → [`Ok`] with empty [`NormalizedBatch`] (idle-friendly).
/// - **Non-empty** → apply per-`data_type` rules; stage [`PipelineStage::Normalized`].
/// - Pure / synchronous: no I/O, no threads, no busy-loop.
/// - Does **not** touch SQLite.
///
/// # Errors
/// Reserved for future policy failures ([`crate::PipelineError::StageFailed`]).
/// The default T3 path always returns [`Ok`] (unparseable known types are skipped).
pub fn normalize_observations(batch: &[Observation]) -> PipelineResult<NormalizedBatch> {
    normalize_owned(batch.to_vec())
}

/// Normalize an owned Observation list without an extra slice clone.
pub fn normalize_owned(observations: Vec<Observation>) -> PipelineResult<NormalizedBatch> {
    let mut kept = Vec::with_capacity(observations.len());
    let mut skipped_count = 0usize;

    for obs in observations {
        match normalize_one(obs) {
            NormalizeOutcome::Keep(normalized) => kept.push(normalized),
            NormalizeOutcome::Skip => {
                skipped_count = skipped_count.saturating_add(1);
            }
        }
    }

    Ok(NormalizedBatch {
        observations: kept,
        skipped_count,
        stage: PipelineStage::Normalized,
    })
}

/// Normalize a [`DedupedBatch`] from the previous stage.
pub fn normalize_deduped(deduped: DedupedBatch) -> PipelineResult<NormalizedBatch> {
    normalize_owned(deduped.into_observations())
}

enum NormalizeOutcome {
    Keep(Observation),
    Skip,
}

fn normalize_one(mut obs: Observation) -> NormalizeOutcome {
    match obs.data_type.as_str() {
        DATA_TYPE_HEART_RATE => match normalize_heart_rate(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_HRV => match normalize_hrv(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_CONTEXT_WINDOW => match normalize_context_window(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_KEYSTROKES => match normalize_keystrokes(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        // Unknown type: pass-through unchanged.
        _ => NormalizeOutcome::Keep(obs),
    }
}

fn normalize_heart_rate(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let mut bpm = first_f64(obj, &["bpm", "heart_rate", "hr", "beats_per_minute"])?;
    if !bpm.is_finite() || bpm <= 0.0 {
        return None;
    }
    if unit_is(obj, "hz") {
        bpm *= 60.0;
    }
    let mut out = Map::new();
    out.insert("bpm".to_string(), json!(bpm));
    if let Some(source) = obj.get("source").and_then(JsonValue::as_str) {
        let trimmed = source.trim();
        if !trimmed.is_empty() {
            out.insert("source".to_string(), json!(trimmed));
        }
    }
    Some(JsonValue::Object(out))
}

fn normalize_hrv(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let mut rmssd_ms = first_f64(obj, &["rmssd_ms", "rmssd", "hrv_ms", "hrv"])?;
    if !rmssd_ms.is_finite() || rmssd_ms < 0.0 {
        return None;
    }
    if unit_is(obj, "s") {
        rmssd_ms *= 1000.0;
    }

    let mut out = Map::new();
    out.insert("rmssd_ms".to_string(), json!(rmssd_ms));

    if let Some(mut sdnn) = first_f64(obj, &["sdnn_ms", "sdnn"]) {
        if sdnn.is_finite() && sdnn >= 0.0 {
            if unit_is(obj, "s") {
                sdnn *= 1000.0;
            }
            out.insert("sdnn_ms".to_string(), json!(sdnn));
        }
    }
    if let Some(pnn50) = first_f64(obj, &["pnn50", "pNN50"]) {
        if pnn50.is_finite() && (0.0..=100.0).contains(&pnn50) {
            out.insert("pnn50".to_string(), json!(pnn50));
        }
    }

    Some(JsonValue::Object(out))
}

fn normalize_context_window(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let bundle_id = first_nonempty_str(obj, &["bundle_id", "bundleId"])?;
    let app_name = first_nonempty_str(obj, &["app_name", "appName", "name"])?;
    Some(json!({
        "bundle_id": bundle_id,
        "app_name": app_name,
    }))
}

fn normalize_keystrokes(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let count = first_u64(obj, &["count"])?;
    let window_secs = first_u64(obj, &["window_secs", "window_seconds", "window"])
        .unwrap_or(60)
        .max(1);
    let rate_per_min = (count as f64) * 60.0 / (window_secs as f64);
    if !rate_per_min.is_finite() {
        return None;
    }
    Some(json!({
        "count": count,
        "window_secs": window_secs,
        "rate_per_min": rate_per_min,
    }))
}

fn first_f64(obj: &Map<String, JsonValue>, keys: &[&str]) -> Option<f64> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(n) = json_as_f64(v) {
                return Some(n);
            }
        }
    }
    None
}

fn first_u64(obj: &Map<String, JsonValue>, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(n) = json_as_u64(v) {
                return Some(n);
            }
        }
    }
    None
}

fn first_nonempty_str<'a>(obj: &'a Map<String, JsonValue>, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(JsonValue::String(s)) = obj.get(*key) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

fn json_as_f64(v: &JsonValue) -> Option<f64> {
    match v {
        JsonValue::Number(n) => n.as_f64(),
        JsonValue::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn json_as_u64(v: &JsonValue) -> Option<u64> {
    match v {
        JsonValue::Number(n) => {
            if let Some(u) = n.as_u64() {
                Some(u)
            } else if let Some(i) = n.as_i64() {
                u64::try_from(i).ok()
            } else {
                n.as_f64()
                    .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                    .map(|f| f as u64)
            }
        }
        JsonValue::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn unit_is(obj: &Map<String, JsonValue>, expected: &str) -> bool {
    obj.get("unit")
        .and_then(JsonValue::as_str)
        .map(|u| u.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        normalize_deduped, normalize_observations, DATA_TYPE_CONTEXT_WINDOW, DATA_TYPE_HEART_RATE,
        DATA_TYPE_HRV, DATA_TYPE_KEYSTROKES,
    };
    use crate::dedupe::DedupeState;
    use crate::intake::accept_observations;
    use crate::PipelineStage;

    fn obs(data_type: &str, payload: serde_json::Value) -> Observation {
        Observation::try_new(
            Uuid::from_u128(1),
            UnixTimestamp::from_secs(100),
            "com.biofocus.test",
            data_type,
            payload,
            1.0,
        )
        .expect("valid test observation")
    }

    #[test]
    fn heart_rate_canonicalizes_bpm_and_source() {
        let input = obs(
            DATA_TYPE_HEART_RATE,
            json!({ "hr": 1.2, "unit": "hz", "source": "  Watch  ", "extra": true }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.stage(), PipelineStage::Normalized);
        assert_eq!(out.len(), 1);
        assert_eq!(out.skipped_count(), 0);
        let p = &out.observations()[0].payload;
        assert_eq!(p["bpm"], json!(72.0));
        assert_eq!(p["source"], json!("Watch"));
        assert!(p.get("extra").is_none());
        assert!(p.get("unit").is_none());
        assert!(p.get("hr").is_none());
    }

    #[test]
    fn heart_rate_missing_bpm_is_skipped() {
        let input = obs(DATA_TYPE_HEART_RATE, json!({ "source": "x" }));
        let out = normalize_observations(&[input]).expect("ok");
        assert!(out.is_empty());
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn hrv_canonicalizes_ms_and_optional_fields() {
        let input = obs(
            DATA_TYPE_HRV,
            json!({
                "hrv": 0.045,
                "unit": "s",
                "sdnn": 0.050,
                "pnn50": 12.5,
                "noise": 1
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(p["rmssd_ms"], json!(45.0));
        assert_eq!(p["sdnn_ms"], json!(50.0));
        assert_eq!(p["pnn50"], json!(12.5));
        assert!(p.get("noise").is_none());
    }

    #[test]
    fn context_window_strips_extras_and_aliases() {
        let input = obs(
            DATA_TYPE_CONTEXT_WINDOW,
            json!({
                "bundleId": "com.apple.Terminal",
                "name": "Terminal",
                "window_title": "secret",
                "keystrokes": 99
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "bundle_id": "com.apple.Terminal",
                "app_name": "Terminal"
            })
        );
    }

    #[test]
    fn keystrokes_recomputes_rate_and_defaults_window() {
        let input = obs(
            DATA_TYPE_KEYSTROKES,
            json!({ "count": 120, "typed_chars": "nope" }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(p["count"], json!(120));
        assert_eq!(p["window_secs"], json!(60));
        assert_eq!(p["rate_per_min"], json!(120.0));
        assert!(p.get("typed_chars").is_none());
    }

    #[test]
    fn unknown_data_type_is_pass_through() {
        let payload = json!({ "custom": 1, "nested": { "a": true } });
        let input = obs("sleep_stage", payload.clone());
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out.skipped_count(), 0);
        assert_eq!(out.observations()[0].data_type, "sleep_stage");
        assert_eq!(out.observations()[0].payload, payload);
    }

    #[test]
    fn empty_batch_is_ok_idle_friendly() {
        let out = normalize_observations(&[]).expect("empty ok");
        assert!(out.is_empty());
        assert_eq!(out.skipped_count(), 0);
        assert_eq!(out.stage(), PipelineStage::Normalized);
    }

    #[test]
    fn normalize_after_dedupe_chain() {
        let a = obs(DATA_TYPE_HEART_RATE, json!({ "bpm": 70 }));
        let dup = a.clone();
        let accepted = accept_observations(&[a, dup]).expect("intake");
        let mut state = DedupeState::new();
        let deduped =
            crate::dedupe_accepted(&mut state, accepted).expect("dedupe");
        assert_eq!(deduped.len(), 1);
        let normalized = normalize_deduped(deduped).expect("normalize");
        assert_eq!(normalized.stage(), PipelineStage::Normalized);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized.observations()[0].payload["bpm"], json!(70.0));
    }
}
