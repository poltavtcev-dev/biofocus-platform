//! Bounded FocusScore baseline series (ADR-008 Pattern Discovery v1).
//!
//! Recompute-on-read: given local Observations, produce up to
//! [`BASELINE_MAX_WINDOWS`] prior **UTC afternoon** FocusScore means.
//! No SQLite Feature history; no always-on worker. Empty / thin days are omitted.

use std::collections::HashSet;

use bio_spec::{
    Confidence, Feature, FeatureValue, Observation, TimeWindow, UnixTimestamp,
};

use crate::catalog::{register_focus_v1, FOCUS_SCORE_ID};
use crate::{FeatureEngine, FeatureEngineResult};

/// Max prior comparable windows for Pattern Discovery v1 (ADR-008).
pub const BASELINE_MAX_WINDOWS: usize = 7;

/// Minimum prior windows before a baseline mean is considered usable by callers.
pub const BASELINE_MIN_WINDOWS: usize = 2;

/// ADR-007 confidence floor for current / series windows used in baseline compare.
pub const BASELINE_CONFIDENCE_GATE: f64 = 0.4;

/// UTC afternoon bucket start hour (inclusive), 13:00.
pub const AFTERNOON_START_HOUR_UTC: i64 = 13;

/// UTC afternoon bucket end hour (inclusive bound via seconds), 17:00.
pub const AFTERNOON_END_HOUR_UTC: i64 = 17;

const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_HOUR: i64 = 3_600;

/// Recompute FocusScore afternoon means for up to `max_days` prior UTC calendar
/// days before the day containing `reference_ts`.
///
/// - Caps `max_days` at [`BASELINE_MAX_WINDOWS`].
/// - Days with no qualifying FocusScore (empty obs / confidence below gate) are skipped.
/// - Returns chronological order (oldest → newest). Idle / thin history → `Ok([])`.
pub fn recompute_focus_afternoon_baseline(
    observations: &[Observation],
    reference_ts: i64,
    max_days: usize,
) -> FeatureEngineResult<Vec<Feature>> {
    let max_days = max_days.min(BASELINE_MAX_WINDOWS);
    if max_days == 0 || observations.is_empty() {
        return Ok(Vec::new());
    }

    let mut engine = FeatureEngine::new();
    register_focus_v1(&mut engine)?;

    let today_start = utc_day_start(reference_ts);
    let mut series = Vec::new();

    for offset in 1..=max_days {
        let day_start = today_start - (offset as i64) * SECS_PER_DAY;
        let Some(mean) = afternoon_focus_mean(&engine, observations, day_start)? else {
            continue;
        };
        series.push(mean);
    }

    // Offsets walk newest-prior first; reverse to oldest → newest.
    series.reverse();
    Ok(series)
}

fn afternoon_focus_mean(
    engine: &FeatureEngine,
    observations: &[Observation],
    day_start: i64,
) -> FeatureEngineResult<Option<Feature>> {
    let aft_start = day_start + AFTERNOON_START_HOUR_UTC * SECS_PER_HOUR;
    let aft_end = day_start + AFTERNOON_END_HOUR_UTC * SECS_PER_HOUR;
    let day_obs: Vec<Observation> = observations
        .iter()
        .filter(|o| {
            let t = o.timestamp.as_secs();
            t >= aft_start && t <= aft_end
        })
        .cloned()
        .collect();
    if day_obs.is_empty() {
        return Ok(None);
    }

    let out = engine.run(&day_obs)?;
    let mut values = Vec::new();
    let mut confidences = Vec::new();
    let mut provenance = Vec::new();
    let mut seen = HashSet::new();

    for feature in out.features {
        if feature.feature_id != FOCUS_SCORE_ID {
            continue;
        }
        if feature.confidence.get() < BASELINE_CONFIDENCE_GATE {
            continue;
        }
        let FeatureValue::Scalar(v) = feature.value else {
            continue;
        };
        if !v.is_finite() {
            continue;
        }
        values.push(v);
        confidences.push(feature.confidence.get());
        for id in feature.provenance {
            if seen.insert(id) {
                provenance.push(id);
            }
        }
    }

    if values.is_empty() {
        return Ok(None);
    }

    let mean_value = values.iter().sum::<f64>() / values.len() as f64;
    let mean_conf = confidences.iter().sum::<f64>() / confidences.len() as f64;
    let window = TimeWindow {
        start: UnixTimestamp::from_secs(aft_start),
        end: UnixTimestamp::from_secs(aft_end),
    };

    Ok(Some(Feature {
        feature_id: FOCUS_SCORE_ID.to_owned(),
        time_window: window,
        value: FeatureValue::Scalar(mean_value.clamp(0.0, 100.0)),
        provenance,
        confidence: Confidence::saturating_from(mean_conf),
        factors: Vec::new(),
    }))
}

#[must_use]
pub fn utc_day_start(ts: i64) -> i64 {
    ts.div_euclid(SECS_PER_DAY) * SECS_PER_DAY
}

/// Lookback start for loading Observations covering `max_days` prior afternoons.
#[must_use]
pub fn baseline_lookback_start(reference_ts: i64, max_days: usize) -> i64 {
    let max_days = max_days.min(BASELINE_MAX_WINDOWS) as i64;
    utc_day_start(reference_ts) - max_days * SECS_PER_DAY + AFTERNOON_START_HOUR_UTC * SECS_PER_HOUR
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn obs(id: u128, ts: i64, data_type: &str, payload: serde_json::Value) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            data_type,
            payload,
            1.0,
        )
        .expect("obs")
    }

    /// Rich afternoon batch at `day_start` (UTC) with high typing / stability / HRV.
    fn rich_afternoon(day_start: i64, id_base: u128, rate: f64) -> Vec<Observation> {
        let t0 = day_start + AFTERNOON_START_HOUR_UTC * SECS_PER_HOUR + 30 * 60;
        vec![
            obs(
                id_base,
                t0,
                "context_window",
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
            obs(
                id_base + 1,
                t0 + 60,
                "context_window",
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
            obs(
                id_base + 2,
                t0 + 120,
                "keystrokes",
                json!({ "count": 180, "window_secs": 60, "rate_per_min": rate }),
            ),
            obs(
                id_base + 3,
                t0 + 180,
                "hrv",
                json!({ "rmssd_ms": 45.0 }),
            ),
            obs(
                id_base + 4,
                t0 + 240,
                "context_window",
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
        ]
    }

    #[test]
    fn thin_history_returns_empty() {
        let reference = 1_700_000_000; // arbitrary
        let out = recompute_focus_afternoon_baseline(&[], reference, 7).expect("ok");
        assert!(out.is_empty());

        // Only "today" afternoon — prior days empty.
        let today = utc_day_start(reference);
        let batch = rich_afternoon(today, 1, 180.0);
        let out = recompute_focus_afternoon_baseline(&batch, reference, 7).expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn rich_prior_days_emit_bounded_series() {
        let reference = 1_700_000_000;
        let today = utc_day_start(reference);
        let mut batch = Vec::new();
        for day in 1..=5 {
            let day_start = today - day * SECS_PER_DAY;
            batch.extend(rich_afternoon(day_start, (day as u128) * 100, 160.0));
        }
        let out = recompute_focus_afternoon_baseline(&batch, reference, 7).expect("ok");
        assert_eq!(out.len(), 5);
        assert!(out.len() <= BASELINE_MAX_WINDOWS);
        for f in &out {
            assert_eq!(f.feature_id, FOCUS_SCORE_ID);
            assert!(f.confidence.get() >= BASELINE_CONFIDENCE_GATE);
            let FeatureValue::Scalar(v) = f.value else {
                panic!("scalar");
            };
            assert!(v > 50.0, "expected solid focus mean, got {v}");
        }
        // Chronological: day-5 window before day-1.
        assert!(out[0].time_window.start.as_secs() < out[4].time_window.start.as_secs());
    }

    #[test]
    fn caps_at_seven_windows() {
        let reference = 1_700_000_000;
        let today = utc_day_start(reference);
        let mut batch = Vec::new();
        for day in 1..=10 {
            let day_start = today - day * SECS_PER_DAY;
            batch.extend(rich_afternoon(day_start, (day as u128) * 100, 150.0));
        }
        let out = recompute_focus_afternoon_baseline(&batch, reference, 99).expect("ok");
        assert_eq!(out.len(), BASELINE_MAX_WINDOWS);
    }

    #[test]
    fn lookback_start_covers_prior_afternoons() {
        let reference = 1_700_000_000;
        let start = baseline_lookback_start(reference, 7);
        let today = utc_day_start(reference);
        let oldest_aft = today - 7 * SECS_PER_DAY + AFTERNOON_START_HOUR_UTC * SECS_PER_HOUR;
        assert_eq!(start, oldest_aft);
    }
}
