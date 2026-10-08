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
//! | `hrv` | `{ "rmssd_ms"?: f64, "sdnn_ms"?: f64, "pnn50"?: f64 }` | at least one of RMSSD (`rmssd_ms` / `rmssd` / `hrv_ms` / `hrv`) or SDNN (`sdnn_ms` / `sdnn`); if `unit` is `s` → ×1000; optional pNN50 |
//! | `context_window` | `{ "bundle_id": string, "app_name": string }` | aliases `bundleId` / `appName` / `name`; other keys stripped |
//! | `keystrokes` | `{ "count": u64, "window_secs": u64, "rate_per_min": f64 }` | `window_seconds`→`window_secs`; `rate_per_min` recomputed; content keys stripped |
//! | `calendar_event` | `{ "uid", "start", "end", "all_day"?, "busy"? }` | titles/bodies/attendees stripped; required uid/start/end |
//! | `browser_category` | `{ "category": string, "browser_bundle_id"?: string }` | closed-set category; `url` / `title` / `href` / content extras stripped (ADR-010) |
//! | `now_playing` | `{ "media_kind": string, "is_playing": bool }` | closed-set kind; `title` / `artist` / `album` / `lyrics` / playlist ids stripped (ADR-012) |
//! | `git_activity` | `{ "activity_kind": string, "event_count"?: u64 }` | closed-set kind; paths / remotes / branch / SHA / message / diff / author stripped (ADR-013) |
//! | `ambient_light` | `{ "light_kind": string, "level"?: u64 }` | closed-set kind; optional level 0–100; camera / screen / geo / mic extras stripped (ADR-015) |
//! | `notification_event` | `{ "count": u64, "category"?, "interruption_level"?, "app_kind"? }` | count ≥ 1; optional closed-sets; body/title/message/screenshot extras stripped (ADR-019) |
//! | `step_count` | `{ "count": u64, "window_secs"?: u64 }` | non-neg count; optional window ≥ 1 (ADR-018) |
//! | `active_energy` | `{ "kcal": f64 }` | kcal ≥ 0; aliases `active_energy_kcal` / `calories` (ADR-018) |
//! | `sleep_interval` | `{ "start", "end", "stage"? }` | end ≥ start; stage closed-set (ADR-018) |
//! | `oxygen_saturation` | `{ "spo2_percent": f64 }` | 0–100; fraction ≤1 → ×100 (ADR-018) |
//!
//! Known type with missing / non-finite required fields → **skipped** (dropped from
//! the batch; counted in [`NormalizedBatch::skipped_count`]).
//!
//! ## Unknown `data_type`
//!
//! **Pass-through** — Observation kept unchanged (forward-compatible). Not counted
//! as skipped.

use bio_spec::{
    Observation, is_v1_activity_kind, is_v1_interruption_level, is_v1_light_kind, is_v1_media_kind,
    is_v1_notification_app_kind, is_v1_notification_category, is_v1_sleep_stage,
};
use serde_json::{Map, Value as JsonValue, json};

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
/// Calendar / meeting events (schedule metadata only).
pub const DATA_TYPE_CALENDAR_EVENT: &str = "calendar_event";
/// Browser coarse category (ADR-010 / P10-E2).
pub const DATA_TYPE_BROWSER_CATEGORY: &str = "browser_category";
/// Now Playing ambient media (ADR-012 / P12-E2).
pub const DATA_TYPE_NOW_PLAYING: &str = "now_playing";
/// Git activity aggregates (ADR-013 / P13-E2).
pub const DATA_TYPE_GIT_ACTIVITY: &str = "git_activity";
/// Ambient light (ADR-015 / P15-E2).
pub const DATA_TYPE_AMBIENT_LIGHT: &str = "ambient_light";
/// Notification event (ADR-019 / P18-E2).
pub const DATA_TYPE_NOTIFICATION_EVENT: &str = "notification_event";
/// Step count (ADR-018 / P17-E2).
pub const DATA_TYPE_STEP_COUNT: &str = "step_count";
/// Active energy kcal (ADR-018 / P17-E2).
pub const DATA_TYPE_ACTIVE_ENERGY: &str = "active_energy";
/// Sleep interval (ADR-018 / P17-E2).
pub const DATA_TYPE_SLEEP_INTERVAL: &str = "sleep_interval";
/// Soft-optional SpO2 percent (ADR-018 / P17-E2).
pub const DATA_TYPE_OXYGEN_SATURATION: &str = "oxygen_saturation";

const V1_BROWSER_CATEGORIES: &[&str] = &[
    "work",
    "communication",
    "entertainment",
    "reference",
    "shopping",
    "unknown",
];

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
        DATA_TYPE_CALENDAR_EVENT => match normalize_calendar_event(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_BROWSER_CATEGORY => match normalize_browser_category(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_NOW_PLAYING => match normalize_now_playing(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_GIT_ACTIVITY => match normalize_git_activity(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_AMBIENT_LIGHT => match normalize_ambient_light(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_NOTIFICATION_EVENT => match normalize_notification_event(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_STEP_COUNT => match normalize_step_count(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_ACTIVE_ENERGY => match normalize_active_energy(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_SLEEP_INTERVAL => match normalize_sleep_interval(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        DATA_TYPE_OXYGEN_SATURATION => match normalize_oxygen_saturation(&obs.payload) {
            Some(payload) => {
                obs.payload = payload;
                NormalizeOutcome::Keep(obs)
            }
            None => NormalizeOutcome::Skip,
        },
        bio_spec::DATA_TYPE_RESTING_HEART_RATE => {
            keep_number(obs, &["bpm", "resting_heart_rate"], "bpm", 25.0, 150.0)
        }
        bio_spec::DATA_TYPE_WALKING_HEART_RATE_AVERAGE => keep_number(
            obs,
            &["bpm", "walking_heart_rate_average"],
            "bpm",
            25.0,
            250.0,
        ),
        bio_spec::DATA_TYPE_RESPIRATORY_RATE => keep_number(
            obs,
            &["breaths_per_min", "respiratory_rate"],
            "breaths_per_min",
            4.0,
            40.0,
        ),
        bio_spec::DATA_TYPE_SLEEPING_WRIST_TEMPERATURE => {
            keep_number(obs, &["celsius", "delta_celsius"], "celsius", -5.0, 5.0)
        }
        bio_spec::DATA_TYPE_VO2_MAX => {
            keep_number(obs, &["ml_kg_min", "vo2_max"], "ml_kg_min", 10.0, 90.0)
        }
        bio_spec::DATA_TYPE_SOURCE_DELETION => NormalizeOutcome::Keep(obs),
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
    if !(25.0..=250.0).contains(&bpm) {
        return None;
    }
    let mut out = Map::new();
    out.insert("bpm".to_string(), json!(bpm));
    if let Some(source) = obj.get("source").and_then(JsonValue::as_str) {
        let trimmed = source.trim();
        if !trimmed.is_empty() {
            out.insert("source".to_string(), json!(trimmed));
        }
    }
    if !attach_src(obj, &mut out) {
        return None;
    }
    Some(JsonValue::Object(out))
}

fn normalize_hrv(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let unit_s = unit_is(obj, "s");

    let mut rmssd_ms = first_f64(obj, &["rmssd_ms", "rmssd", "hrv_ms", "hrv"]).and_then(|v| {
        if !v.is_finite() || v < 0.0 {
            return None;
        }
        Some(if unit_s { v * 1000.0 } else { v })
    });

    let mut sdnn_ms = first_f64(obj, &["sdnn_ms", "sdnn"]).and_then(|v| {
        if !v.is_finite() || v < 0.0 {
            return None;
        }
        Some(if unit_s { v * 1000.0 } else { v })
    });

    rmssd_ms = rmssd_ms.filter(|v| (1.0..=300.0).contains(v));
    sdnn_ms = sdnn_ms.filter(|v| (1.0..=300.0).contains(v));

    let method = obj.get("method").and_then(JsonValue::as_str);
    match method {
        Some("sdnn") => rmssd_ms = None,
        Some("rmssd") => sdnn_ms = None,
        Some(_) => return None,
        None => {}
    }

    // ADR-016: accept SDNN-only (Apple HealthKit HRV) or RMSSD-only / both.
    // ADR-030: a stated method keeps only that metric. Out of 1–300 ms is dropped.
    if rmssd_ms.is_none() && sdnn_ms.is_none() {
        return None;
    }

    let mut out = Map::new();
    if let Some(v) = rmssd_ms.take() {
        out.insert("rmssd_ms".to_string(), json!(v));
    }
    if let Some(v) = sdnn_ms.take() {
        out.insert("sdnn_ms".to_string(), json!(v));
    }
    if method == Some("sdnn")
        || (method.is_none() && out.contains_key("sdnn_ms") && !out.contains_key("rmssd_ms"))
    {
        out.insert("method".to_string(), json!("sdnn"));
    } else if method == Some("rmssd")
        || (method.is_none() && out.contains_key("rmssd_ms") && !out.contains_key("sdnn_ms"))
    {
        out.insert("method".to_string(), json!("rmssd"));
    }
    if let Some(pnn50) = first_f64(obj, &["pnn50", "pNN50"]) {
        if pnn50.is_finite() && (0.0..=100.0).contains(&pnn50) {
            out.insert("pnn50".to_string(), json!(pnn50));
        }
    }
    if !attach_src(obj, &mut out) {
        return None;
    }

    Some(JsonValue::Object(out))
}

fn attach_src(from: &Map<String, JsonValue>, out: &mut Map<String, JsonValue>) -> bool {
    let Some(src_val) = from.get("src") else {
        return true;
    };
    let Some(src) = src_val.as_object() else {
        return false;
    };
    if src.contains_key("name")
        || src.contains_key("device_name")
        || src.contains_key("source_name")
    {
        return false;
    }
    let Some(kind) = src.get("kind").and_then(JsonValue::as_str) else {
        return false;
    };
    if !bio_spec::is_src_kind(kind) {
        return false;
    }
    out.insert("src".to_string(), src_val.clone());
    true
}

fn in_range(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && (min..=max).contains(&value)
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

fn normalize_calendar_event(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let uid = first_nonempty_str(obj, &["uid"])?;
    let start = first_i64(obj, &["start"])?;
    let end = first_i64(obj, &["end"])?;
    if end < start {
        return None;
    }
    let mut out = Map::new();
    out.insert("uid".to_string(), json!(uid));
    out.insert("start".to_string(), json!(start));
    out.insert("end".to_string(), json!(end));
    if let Some(JsonValue::Bool(all_day)) = obj.get("all_day") {
        out.insert("all_day".to_string(), json!(all_day));
    }
    if let Some(JsonValue::Bool(busy)) = obj.get("busy") {
        out.insert("busy".to_string(), json!(busy));
    }
    // Explicitly drop title / body / attendees / location if present.
    Some(JsonValue::Object(out))
}

fn normalize_browser_category(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let category = first_nonempty_str(obj, &["category"])?;
    if !V1_BROWSER_CATEGORIES.contains(&category) {
        return None;
    }
    let mut out = Map::new();
    out.insert("category".to_string(), json!(category));
    if let Some(bundle) = first_nonempty_str(obj, &["browser_bundle_id", "browserBundleId"]) {
        out.insert("browser_bundle_id".to_string(), json!(bundle));
    }
    // Explicitly drop url / title / href / content if present.
    Some(JsonValue::Object(out))
}

fn normalize_now_playing(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let media_kind = first_nonempty_str(obj, &["media_kind", "mediaKind"])?;
    if !is_v1_media_kind(media_kind) {
        return None;
    }
    let is_playing = match obj.get("is_playing").or_else(|| obj.get("isPlaying")) {
        Some(JsonValue::Bool(b)) => *b,
        _ => return None,
    };
    let mut out = Map::new();
    out.insert("media_kind".to_string(), json!(media_kind));
    out.insert("is_playing".to_string(), json!(is_playing));
    // Explicitly drop title / artist / album / lyrics / playlist ids / etc.
    Some(JsonValue::Object(out))
}

fn normalize_git_activity(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let activity_kind = first_nonempty_str(obj, &["activity_kind", "activityKind"])?;
    if !is_v1_activity_kind(activity_kind) {
        return None;
    }
    let mut out = Map::new();
    out.insert("activity_kind".to_string(), json!(activity_kind));
    if let Some(count) = first_u64(obj, &["event_count", "eventCount"]) {
        if count >= 1 {
            out.insert("event_count".to_string(), json!(count));
        } else {
            return None;
        }
    }
    // Explicitly drop path / remote / branch / sha / message / diff / author / etc.
    Some(JsonValue::Object(out))
}

fn normalize_ambient_light(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let light_kind = first_nonempty_str(obj, &["light_kind", "lightKind"])?;
    if !is_v1_light_kind(light_kind) {
        return None;
    }
    let mut out = Map::new();
    out.insert("light_kind".to_string(), json!(light_kind));
    if let Some(level) = first_u64(obj, &["level"]) {
        if level <= 100 {
            out.insert("level".to_string(), json!(level));
        } else {
            return None;
        }
    }
    // Explicitly drop camera / screen / geo / mic extras if present.
    Some(JsonValue::Object(out))
}

fn normalize_notification_event(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let count = first_u64(obj, &["count"])?;
    if count < 1 {
        return None;
    }
    let mut out = Map::new();
    out.insert("count".to_string(), json!(count));

    if let Some(category) = first_nonempty_str(obj, &["category"]) {
        if !is_v1_notification_category(category) {
            return None;
        }
        out.insert("category".to_string(), json!(category));
    }
    if let Some(level) = first_nonempty_str(obj, &["interruption_level", "interruptionLevel"]) {
        if !is_v1_interruption_level(level) {
            return None;
        }
        out.insert("interruption_level".to_string(), json!(level));
    }
    if let Some(app_kind) = first_nonempty_str(obj, &["app_kind", "appKind"]) {
        if !is_v1_notification_app_kind(app_kind) {
            return None;
        }
        out.insert("app_kind".to_string(), json!(app_kind));
    }
    // Explicitly drop body / title / message / screenshot / userInfo extras.
    Some(JsonValue::Object(out))
}

fn normalize_step_count(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let count = first_u64(obj, &["count", "steps"])?;
    let mut out = Map::new();
    out.insert("count".to_string(), json!(count));
    if let Some(window) = first_u64(obj, &["window_secs", "window_seconds"]) {
        if window < 1 {
            return None;
        }
        out.insert("window_secs".to_string(), json!(window));
    }
    if !attach_src(obj, &mut out) {
        return None;
    }
    Some(JsonValue::Object(out))
}

fn normalize_active_energy(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let kcal = first_f64(obj, &["kcal", "active_energy_kcal", "calories"])?;
    if !kcal.is_finite() || kcal < 0.0 {
        return None;
    }
    let mut out = Map::new();
    out.insert("kcal".to_string(), json!(kcal));
    if !attach_src(obj, &mut out) {
        return None;
    }
    Some(JsonValue::Object(out))
}

fn normalize_sleep_interval(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let start = first_i64(obj, &["start"])?;
    let end = first_i64(obj, &["end"])?;
    if end < start {
        return None;
    }
    let mut out = Map::new();
    out.insert("start".to_string(), json!(start));
    out.insert("end".to_string(), json!(end));
    if let Some(stage) = first_nonempty_str(obj, &["stage", "sleep_stage"]) {
        if !is_v1_sleep_stage(stage) {
            return None;
        }
        out.insert("stage".to_string(), json!(stage));
    }
    if !attach_src(obj, &mut out) {
        return None;
    }
    Some(JsonValue::Object(out))
}

fn normalize_oxygen_saturation(payload: &JsonValue) -> Option<JsonValue> {
    let obj = payload.as_object()?;
    let mut pct = first_f64(obj, &["spo2_percent", "spo2", "oxygen_saturation"])?;
    if !pct.is_finite() || pct < 0.0 {
        return None;
    }
    // HealthKit often stores SpO2 as a 0–1 fraction.
    if pct <= 1.0 {
        pct *= 100.0;
    }
    if !in_range(pct, 50.0, 100.0) {
        return None;
    }
    let mut out = Map::new();
    out.insert("spo2_percent".to_string(), json!(pct));
    if !attach_src(obj, &mut out) {
        return None;
    }
    Some(JsonValue::Object(out))
}

fn keep_number(
    mut obs: Observation,
    keys: &[&str],
    out_key: &str,
    min: f64,
    max: f64,
) -> NormalizeOutcome {
    let Some(obj) = obs.payload.as_object() else {
        return NormalizeOutcome::Skip;
    };
    let Some(value) = first_f64(obj, keys) else {
        return NormalizeOutcome::Skip;
    };
    if !in_range(value, min, max) {
        return NormalizeOutcome::Skip;
    }
    let mut out = Map::new();
    out.insert(out_key.to_string(), json!(value));
    if !attach_src(obj, &mut out) {
        return NormalizeOutcome::Skip;
    }
    obs.payload = JsonValue::Object(out);
    NormalizeOutcome::Keep(obs)
}

fn first_i64(obj: &Map<String, JsonValue>, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(v) = obj.get(*key) {
            if let Some(n) = json_as_i64(v) {
                return Some(n);
            }
        }
    }
    None
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
        JsonValue::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
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
        DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_AMBIENT_LIGHT, DATA_TYPE_BROWSER_CATEGORY,
        DATA_TYPE_CALENDAR_EVENT, DATA_TYPE_CONTEXT_WINDOW, DATA_TYPE_GIT_ACTIVITY,
        DATA_TYPE_HEART_RATE, DATA_TYPE_HRV, DATA_TYPE_KEYSTROKES, DATA_TYPE_NOTIFICATION_EVENT,
        DATA_TYPE_NOW_PLAYING, DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_SLEEP_INTERVAL,
        DATA_TYPE_STEP_COUNT, normalize_deduped, normalize_observations,
    };
    use crate::PipelineStage;
    use crate::dedupe::DedupeState;
    use crate::intake::accept_observations;

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
    fn hrv_sdnn_only_is_kept() {
        let input = obs(
            DATA_TYPE_HRV,
            json!({
                "sdnn_ms": 42.0,
                "source": "HealthKit"
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(p["sdnn_ms"], json!(42.0));
        assert!(p.get("rmssd_ms").is_none());
        assert!(p.get("source").is_none());
    }

    #[test]
    fn hrv_missing_both_ms_fields_is_skipped() {
        let input = obs(DATA_TYPE_HRV, json!({ "pnn50": 10.0 }));
        let out = normalize_observations(&[input]).expect("ok");
        assert!(out.is_empty());
        assert_eq!(out.skipped_count(), 1);
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
    fn calendar_event_strips_titles_and_keeps_schedule() {
        let input = obs(
            DATA_TYPE_CALENDAR_EVENT,
            json!({
                "uid": "meet-1",
                "start": 100,
                "end": 200,
                "all_day": false,
                "busy": true,
                "title": "Secret",
                "description": "leak",
                "attendees": ["a@b.c"]
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "uid": "meet-1",
                "start": 100,
                "end": 200,
                "all_day": false,
                "busy": true
            })
        );
        assert!(p.get("title").is_none());
        assert!(p.get("description").is_none());
        assert!(p.get("attendees").is_none());
    }

    #[test]
    fn browser_category_strips_url_title_and_keeps_coarse_label() {
        let input = obs(
            DATA_TYPE_BROWSER_CATEGORY,
            json!({
                "category": "work",
                "browser_bundle_id": "com.apple.Safari",
                "url": "https://example.com/secret",
                "title": "Should Not Persist",
                "href": "/path"
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(p["category"], json!("work"));
        assert_eq!(p["browser_bundle_id"], json!("com.apple.Safari"));
        assert!(p.get("url").is_none());
        assert!(p.get("title").is_none());
        assert!(p.get("href").is_none());
    }

    #[test]
    fn browser_category_rejects_unknown_label() {
        let input = obs(DATA_TYPE_BROWSER_CATEGORY, json!({ "category": "social" }));
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn now_playing_strips_title_artist_and_keeps_coarse_fields() {
        let input = obs(
            DATA_TYPE_NOW_PLAYING,
            json!({
                "media_kind": "music",
                "is_playing": true,
                "title": "Secret Song",
                "artist": "Leak Band",
                "album": "Private",
                "lyrics": "do not persist",
                "playlist_id": "pl-123",
                "track_id": "tr-9"
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "media_kind": "music",
                "is_playing": true
            })
        );
        assert!(p.get("title").is_none());
        assert!(p.get("artist").is_none());
        assert!(p.get("album").is_none());
        assert!(p.get("lyrics").is_none());
        assert!(p.get("playlist_id").is_none());
        assert!(p.get("track_id").is_none());
    }

    #[test]
    fn now_playing_rejects_invalid_kind() {
        let input = obs(
            DATA_TYPE_NOW_PLAYING,
            json!({ "media_kind": "audiobook", "is_playing": true }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn git_activity_strips_paths_remotes_and_keeps_coarse_fields() {
        let input = obs(
            DATA_TYPE_GIT_ACTIVITY,
            json!({
                "activity_kind": "commit",
                "event_count": 2,
                "repo_path": "/Users/me/secret-repo",
                "remote": "git@github.com:org/secret.git",
                "branch": "feature/leak",
                "sha": "deadbeef",
                "message": "do not persist",
                "diff": "--- a/file",
                "author": "leak@example.com",
                "files": ["src/a.rs"]
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "activity_kind": "commit",
                "event_count": 2
            })
        );
        assert!(p.get("repo_path").is_none());
        assert!(p.get("remote").is_none());
        assert!(p.get("branch").is_none());
        assert!(p.get("sha").is_none());
        assert!(p.get("message").is_none());
        assert!(p.get("diff").is_none());
        assert!(p.get("author").is_none());
        assert!(p.get("files").is_none());
    }

    #[test]
    fn git_activity_rejects_invalid_kind() {
        let input = obs(
            DATA_TYPE_GIT_ACTIVITY,
            json!({ "activity_kind": "rebase", "event_count": 1 }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn ambient_light_strips_camera_geo_and_keeps_coarse_fields() {
        let input = obs(
            DATA_TYPE_AMBIENT_LIGHT,
            json!({
                "light_kind": "dim",
                "level": 25,
                "camera_frame": "base64...",
                "screenshot": "...",
                "latitude": 52.1,
                "longitude": 21.0,
                "mic_waveform": [0.1, 0.2]
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "light_kind": "dim",
                "level": 25
            })
        );
        assert!(p.get("camera_frame").is_none());
        assert!(p.get("screenshot").is_none());
        assert!(p.get("latitude").is_none());
        assert!(p.get("longitude").is_none());
        assert!(p.get("mic_waveform").is_none());
    }

    #[test]
    fn notification_event_strips_content_and_keeps_coarse_fields() {
        let input = obs(
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({
                "count": 2,
                "category": "communication",
                "interruption_level": "active",
                "app_kind": "messaging",
                "title": "Secret subject",
                "body": "Message body",
                "subtitle": "Preview",
                "message": "chat text",
                "screenshot": "...",
                "userInfo": { "thread": "abc" }
            }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(
            p,
            &json!({
                "count": 2,
                "category": "communication",
                "interruption_level": "active",
                "app_kind": "messaging"
            })
        );
        assert!(p.get("title").is_none());
        assert!(p.get("body").is_none());
        assert!(p.get("subtitle").is_none());
        assert!(p.get("message").is_none());
        assert!(p.get("screenshot").is_none());
        assert!(p.get("userInfo").is_none());
    }

    #[test]
    fn notification_event_rejects_zero_count() {
        let input = obs(
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": 0, "category": "system" }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn notification_event_rejects_bad_category() {
        let input = obs(
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": 1, "category": "urgent_work" }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn ambient_light_rejects_invalid_kind() {
        let input = obs(
            DATA_TYPE_AMBIENT_LIGHT,
            json!({ "light_kind": "glaring", "level": 10 }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn ambient_light_rejects_level_above_100() {
        let input = obs(
            DATA_TYPE_AMBIENT_LIGHT,
            json!({ "light_kind": "dim", "level": 101 }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn step_count_canonicalizes() {
        let input = obs(
            DATA_TYPE_STEP_COUNT,
            json!({ "steps": 100, "window_seconds": 60 }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(
            out.observations()[0].payload,
            json!({ "count": 100, "window_secs": 60 })
        );
    }

    #[test]
    fn active_energy_rejects_negative() {
        let input = obs(DATA_TYPE_ACTIVE_ENERGY, json!({ "kcal": -1.0 }));
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 0);
        assert_eq!(out.skipped_count(), 1);
    }

    #[test]
    fn sleep_interval_keeps_stage() {
        let input = obs(
            DATA_TYPE_SLEEP_INTERVAL,
            json!({ "start": 100, "end": 200, "stage": "asleep" }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out.observations()[0].payload["stage"], json!("asleep"));
    }

    #[test]
    fn oxygen_saturation_scales_fraction() {
        let input = obs(DATA_TYPE_OXYGEN_SATURATION, json!({ "spo2": 0.97 }));
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let pct = out.observations()[0].payload["spo2_percent"]
            .as_f64()
            .expect("pct");
        assert!((pct - 97.0).abs() < 1e-9);
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
        let deduped = crate::dedupe_accepted(&mut state, accepted).expect("dedupe");
        assert_eq!(deduped.len(), 1);
        let normalized = normalize_deduped(deduped).expect("normalize");
        assert_eq!(normalized.stage(), PipelineStage::Normalized);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized.observations()[0].payload["bpm"], json!(70.0));
    }

    #[test]
    fn heart_rate_outside_range_is_skipped() {
        let low = obs(DATA_TYPE_HEART_RATE, json!({ "bpm": 10 }));
        let high = obs(DATA_TYPE_HEART_RATE, json!({ "bpm": 400 }));
        let out = normalize_observations(&[low, high]).expect("ok");
        assert!(out.is_empty());
        assert_eq!(out.skipped_count(), 2);
    }

    #[test]
    fn hrv_method_sdnn_drops_rmssd() {
        let input = obs(
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": 48, "rmssd_ms": 30 }),
        );
        let out = normalize_observations(&[input]).expect("ok");
        assert_eq!(out.len(), 1);
        let p = &out.observations()[0].payload;
        assert_eq!(p["method"], json!("sdnn"));
        assert_eq!(p["sdnn_ms"], json!(48.0));
        assert!(p.get("rmssd_ms").is_none());
    }

    #[test]
    fn hrv_outside_range_is_skipped() {
        let input = obs(DATA_TYPE_HRV, json!({ "sdnn_ms": 0 }));
        let out = normalize_observations(&[input]).expect("ok");
        assert!(out.is_empty());
        assert_eq!(out.skipped_count(), 1);
    }
}
