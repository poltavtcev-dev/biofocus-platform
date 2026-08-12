//! `CircadianOffset` v1 — calm schedule alignment from Observation-level
//! sleep timing vs work/activity timing (`docs/06-feature-catalog.md` /
//! ADR-025 / P24-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (catalog DAG; series may coarsen).
//! - **Lookback for timing math:** 24 hours ending at the Feature window end
//!   (sleep + work schedule needs a day — mirrors SleepDebt rest lookback).
//! - **Inputs (Observation-level timing only):**
//!   - Sleep timing: qualifying `sleep_interval` (same rest stages as SleepDebt:
//!     `asleep` / `in_bed` / missing stage; `awake` / `unknown` do not contribute)
//!     → sleep midpoint of the union of lookback overlaps.
//!   - Work/activity timing: preferred `keystrokes` / `context_window` timestamps;
//!     optional reinforcement when desk signals are thin — `step_count` /
//!     `active_energy` / workout `life_event` → work/activity centroid.
//! - **Compose HIGH alignment (0–100):**
//!   `expected_wake_mid = sleep_mid + 12h` (circular day);
//!   `offset_hours = circular_hours(|work_mid − expected_wake_mid|)` (0..12);
//!   `value = clamp(100 × (1 − offset_hours / 6), 0, 100)`.
//! - **Omit (LOCKED):** unless **both** sleep timing and work/activity timing
//!   present — do **not** renormalize a single slot; do **not** emit signed
//!   chronotype hours; do **not** use SleepDebt / EnergyScore / ActivityBalance /
//!   FocusScore / DeskAwayPresence magnitudes as timing proxies.
//! - **Confidence (ADR-007):** expected slots = 2 (`sleep_timing` /
//!   `work_activity_timing`); emit only when both present → coverage = 1.0;
//!   `coverage × mean(evidence Observation.confidence)`.
//! - **Explanation factors:** `sleep_timing` / `work_timing` (+ optional
//!   `activity_timing` when reinforcement used); shares sum to 1.0.
//! - Calm framing: “schedule alignment in this window” — **not** chronotype /
//!   circadian-disorder / “night owl so you fail”.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_ACTIVE_ENERGY,
    DATA_TYPE_LIFE_EVENT, DATA_TYPE_SLEEP_INTERVAL, DATA_TYPE_STEP_COUNT, LIFE_EVENT_KIND_WORKOUT,
    SLEEP_STAGE_ASLEEP, SLEEP_STAGE_AWAKE, SLEEP_STAGE_IN_BED, SLEEP_STAGE_UNKNOWN,
};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::window::{sliding_window_ends_for, snapshot_time_span, window_ending_at};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "CircadianOffset";

const DATA_TYPE_KEYSTROKES: &str = "keystrokes";
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

const LOOKBACK_SECS: i64 = 24 * 60 * 60;
const DAY_SECS: i64 = 24 * 60 * 60;
const HALF_DAY_SECS: i64 = 12 * 60 * 60;
/// Offset hours at which alignment saturates to 0.
const OFFSET_SATURATION_HOURS: f64 = 6.0;

const EXPECTED_INPUT_SLOTS: usize = 2;

const FACTOR_SLEEP: &str = "sleep_timing";
const FACTOR_WORK: &str = "work_timing";
const FACTOR_ACTIVITY: &str = "activity_timing";
const LABEL_SLEEP: &str = "Sleep timing";
const LABEL_WORK: &str = "Work timing";
const LABEL_ACTIVITY: &str = "Activity timing";

/// DAG node computing [`FEATURE_ID`] (independent; Observation-level only).
#[derive(Debug, Clone, Default)]
pub struct CircadianOffsetNode;

impl CircadianOffsetNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for CircadianOffsetNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &[]
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let Some((min_ts, max_ts)) = snapshot_time_span(ctx.observations()) else {
            return Ok(NodeOutput::empty());
        };

        let mut features = Vec::new();
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx.observations(), &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(observations: &[Observation], window: &TimeWindow) -> Option<Feature> {
    let lookback_start = window.end.as_secs().saturating_sub(LOOKBACK_SECS);
    let lookback_end = window.end.as_secs();

    let (sleep_mid, sleep_evidence) =
        sleep_timing_midpoint(observations, lookback_start, lookback_end)?;

    let work = work_activity_centroid(observations, lookback_start, lookback_end)?;
    let work_mid = work.centroid_secs;

    let expected_wake_mid = sleep_mid.saturating_add(HALF_DAY_SECS);
    let offset_hours = circular_hours_diff(work_mid, expected_wake_mid);
    let value = (100.0 * (1.0 - offset_hours / OFFSET_SATURATION_HOURS)).clamp(0.0, 100.0);

    let mut evidence: Vec<&Observation> = sleep_evidence.clone();
    evidence.extend(work.evidence.iter().copied());

    let confidence = compute_feature_confidence(EXPECTED_INPUT_SLOTS, 2, &evidence);

    let factors = explanation_factors(work.used_desk, work.used_activity);

    let mut provenance: Vec<_> = evidence.iter().map(|o| o.id).collect();
    provenance.sort_unstable();
    provenance.dedup();

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value),
        provenance,
        confidence,
        factors,
    })
}

/// Sleep midpoint from union of qualifying rest intervals overlapping lookback.
///
/// Returns `(midpoint_secs, evidence)` or `None` when no qualifying overlap.
fn sleep_timing_midpoint<'a>(
    observations: &'a [Observation],
    lookback_start: i64,
    lookback_end: i64,
) -> Option<(i64, Vec<&'a Observation>)> {
    let mut evidence: Vec<&Observation> = Vec::new();
    let mut span_start: Option<i64> = None;
    let mut span_end: Option<i64> = None;

    for obs in observations
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_SLEEP_INTERVAL)
    {
        if !qualifies_as_rest(obs) {
            continue;
        }
        let Some((start, end)) = interval_bounds(obs) else {
            continue;
        };
        let overlap_start = start.max(lookback_start);
        let overlap_end = end.min(lookback_end);
        if overlap_end <= overlap_start {
            continue;
        }
        evidence.push(obs);
        span_start = Some(match span_start {
            Some(s) => s.min(overlap_start),
            None => overlap_start,
        });
        span_end = Some(match span_end {
            Some(e) => e.max(overlap_end),
            None => overlap_end,
        });
    }

    let start = span_start?;
    let end = span_end?;
    if evidence.is_empty() || end <= start {
        return None;
    }
    // Midpoint of the union span (ADR-025: midpoint of union(sleep_intervals)).
    let mid = start + (end - start) / 2;
    Some((mid, evidence))
}

struct WorkActivityTiming<'a> {
    centroid_secs: i64,
    evidence: Vec<&'a Observation>,
    used_desk: bool,
    used_activity: bool,
}

/// Work/activity centroid: prefer desk (`keystrokes` / `context_window`);
/// reinforce with steps / active energy / workout when desk is thin.
fn work_activity_centroid<'a>(
    observations: &'a [Observation],
    lookback_start: i64,
    lookback_end: i64,
) -> Option<WorkActivityTiming<'a>> {
    let mut desk_evidence: Vec<&Observation> = Vec::new();
    let mut desk_weight_sum = 0.0;
    let mut desk_weighted_ts = 0.0;

    let mut activity_evidence: Vec<&Observation> = Vec::new();
    let mut activity_weight_sum = 0.0;
    let mut activity_weighted_ts = 0.0;

    for obs in observations {
        let t = obs.timestamp.as_secs();
        if t < lookback_start || t > lookback_end {
            continue;
        }

        match obs.data_type.as_str() {
            DATA_TYPE_KEYSTROKES => {
                if !is_active_keystrokes(obs) {
                    continue;
                }
                let w = keystrokes_weight(obs);
                desk_weight_sum += w;
                desk_weighted_ts += w * t as f64;
                desk_evidence.push(obs);
            }
            DATA_TYPE_CONTEXT_WINDOW => {
                desk_weight_sum += 1.0;
                desk_weighted_ts += t as f64;
                desk_evidence.push(obs);
            }
            DATA_TYPE_STEP_COUNT => {
                let count = obs
                    .payload
                    .get("count")
                    .and_then(json_u64)
                    .unwrap_or(0);
                if count == 0 {
                    continue;
                }
                let w = count as f64;
                activity_weight_sum += w;
                activity_weighted_ts += w * t as f64;
                activity_evidence.push(obs);
            }
            DATA_TYPE_ACTIVE_ENERGY => {
                let kcal = obs
                    .payload
                    .get("kcal")
                    .and_then(|v| v.as_f64())
                    .filter(|k| k.is_finite() && *k > 0.0);
                let Some(kcal) = kcal else {
                    continue;
                };
                activity_weight_sum += kcal;
                activity_weighted_ts += kcal * t as f64;
                activity_evidence.push(obs);
            }
            DATA_TYPE_LIFE_EVENT => {
                if obs.payload.get("kind").and_then(|v| v.as_str()) != Some(LIFE_EVENT_KIND_WORKOUT)
                {
                    continue;
                }
                activity_weight_sum += 1.0;
                activity_weighted_ts += t as f64;
                activity_evidence.push(obs);
            }
            _ => {}
        }
    }

    // Prefer desk; reinforce with activity only when desk signals are thin/absent.
    if desk_weight_sum > 0.0 {
        let centroid = (desk_weighted_ts / desk_weight_sum).round() as i64;
        return Some(WorkActivityTiming {
            centroid_secs: centroid,
            evidence: desk_evidence,
            used_desk: true,
            used_activity: false,
        });
    }

    if activity_weight_sum > 0.0 {
        let centroid = (activity_weighted_ts / activity_weight_sum).round() as i64;
        return Some(WorkActivityTiming {
            centroid_secs: centroid,
            evidence: activity_evidence,
            used_desk: false,
            used_activity: true,
        });
    }

    None
}

fn explanation_factors(used_desk: bool, used_activity: bool) -> Vec<ExplanationFactor> {
    let mut factors = vec![ExplanationFactor {
        id: FACTOR_SLEEP.to_owned(),
        label: LABEL_SLEEP.to_owned(),
        share: 0.5,
    }];
    if used_desk {
        factors.push(ExplanationFactor {
            id: FACTOR_WORK.to_owned(),
            label: LABEL_WORK.to_owned(),
            share: 0.5,
        });
    } else if used_activity {
        factors.push(ExplanationFactor {
            id: FACTOR_ACTIVITY.to_owned(),
            label: LABEL_ACTIVITY.to_owned(),
            share: 0.5,
        });
    }
    factors
}

/// Absolute circular hour difference folded into `[0, 12]`.
fn circular_hours_diff(a_secs: i64, b_secs: i64) -> f64 {
    let mut diff = (a_secs - b_secs).rem_euclid(DAY_SECS);
    if diff > HALF_DAY_SECS {
        diff = DAY_SECS - diff;
    }
    diff as f64 / 3600.0
}

fn qualifies_as_rest(obs: &Observation) -> bool {
    match obs.payload.get("stage").and_then(|v| v.as_str()) {
        None => true,
        Some(SLEEP_STAGE_ASLEEP | SLEEP_STAGE_IN_BED) => true,
        Some(SLEEP_STAGE_AWAKE | SLEEP_STAGE_UNKNOWN) => false,
        Some(_) => false,
    }
}

fn interval_bounds(obs: &Observation) -> Option<(i64, i64)> {
    let start = obs.payload.get("start").and_then(json_i64)?;
    let end = obs.payload.get("end").and_then(json_i64)?;
    if end < start {
        return None;
    }
    Some((start, end))
}

fn is_active_keystrokes(obs: &Observation) -> bool {
    let count = obs
        .payload
        .get("count")
        .and_then(json_u64)
        .unwrap_or(0);
    if count > 0 {
        return true;
    }
    obs.payload
        .get("rate_per_min")
        .and_then(|v| v.as_f64())
        .filter(|r| r.is_finite() && *r > 0.0)
        .is_some()
}

fn keystrokes_weight(obs: &Observation) -> f64 {
    obs.payload
        .get("count")
        .and_then(json_u64)
        .map(|c| (c as f64).max(1.0))
        .or_else(|| {
            obs.payload
                .get("rate_per_min")
                .and_then(|v| v.as_f64())
                .filter(|r| r.is_finite() && *r > 0.0)
        })
        .unwrap_or(1.0)
}

fn json_i64(v: &serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64().or_else(|| {
            n.as_u64()
                .and_then(|u| i64::try_from(u).ok())
                .or_else(|| {
                    n.as_f64()
                        .filter(|f| f.is_finite() && f.fract() == 0.0)
                        .map(|f| f as i64)
                })
        }),
        _ => None,
    }
}

fn json_u64(v: &serde_json::Value) -> Option<u64> {
    v.as_u64().or_else(|| {
        v.as_i64()
            .and_then(|i| u64::try_from(i).ok())
            .or_else(|| {
                v.as_f64()
                    .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                    .map(|f| f as u64)
            })
    })
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::register_catalog_v1;
    use crate::FeatureEngine;

    fn sleep_obs(id: u128, ts: i64, start: i64, end: i64, stage: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_SLEEP_INTERVAL,
            json!({ "start": start, "end": end, "stage": stage }),
            1.0,
        )
        .expect("obs")
    }

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

    fn last_circadian(batch: &[Observation]) -> Option<Feature> {
        let mut engine = FeatureEngine::new();
        engine
            .register(CircadianOffsetNode::new())
            .expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
    }

    /// Sleep ~00:00–08:00 → mid 04:00; expected wake mid = 16:00.
    /// Work clustered at 16:00 → offset ≈ 0 → alignment ≈ 100.
    #[test]
    fn both_slots_aligned_emits_near_100() {
        let day = 1_700_000_000i64; // fixed epoch base
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        let work_ts = day + 16 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep"),
            obs(
                2,
                work_ts,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 60, "window_secs": 60, "rate_per_min": 60.0 }),
            ),
            obs(
                3,
                work_ts + 60,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
        ];
        let feat = last_circadian(&batch).expect("CircadianOffset");
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v >= 95.0, "aligned schedule expected ~100, got {v}");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_SLEEP));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_WORK));
        assert!((feat.confidence.get() - 1.0).abs() < 1e-9);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sleep_plus_activity_only_emits_with_activity_factor() {
        let day = 1_700_100_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        // Activity at expected mid-wake (~16:00).
        let activity_ts = day + 16 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep"),
            obs(
                2,
                activity_ts,
                DATA_TYPE_STEP_COUNT,
                json!({ "count": 200 }),
            ),
            obs(
                3,
                activity_ts + 120,
                DATA_TYPE_LIFE_EVENT,
                json!({ "kind": "workout" }),
            ),
        ];
        let feat = last_circadian(&batch).expect("activity reinforcement");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_ACTIVITY));
        assert!(!feat.factors.iter().any(|f| f.id == FACTOR_WORK));
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((0.0..=100.0).contains(&v));
    }

    #[test]
    fn omits_without_sleep() {
        let ts = 1_700_200_000i64;
        let batch = [obs(
            1,
            ts,
            DATA_TYPE_KEYSTROKES,
            json!({ "count": 40, "window_secs": 60, "rate_per_min": 40.0 }),
        )];
        assert!(last_circadian(&batch).is_none());
    }

    #[test]
    fn omits_without_work_or_activity() {
        let day = 1_700_300_000i64;
        let sleep_start = day;
        let sleep_end = day + 7 * 3600;
        let batch = [sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep")];
        assert!(last_circadian(&batch).is_none());
    }

    #[test]
    fn omits_awake_only_sleep() {
        let day = 1_700_400_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        let work_ts = day + 16 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "awake"),
            obs(
                2,
                work_ts,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 50, "window_secs": 60, "rate_per_min": 50.0 }),
            ),
        ];
        assert!(last_circadian(&batch).is_none());
    }

    #[test]
    fn large_offset_lowers_alignment() {
        let day = 1_700_500_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        // Sleep mid ≈ 04:00; expected wake mid ≈ 16:00.
        // Work at 04:00 same day → circular offset ≈ 12h → value ≈ 0.
        let work_ts = day + 4 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep"),
            obs(
                2,
                work_ts,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
        ];
        let feat = last_circadian(&batch).expect("emits");
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v <= 5.0, "12h offset should saturate near 0, got {v}");
    }

    #[test]
    fn confidence_uses_two_slots_when_emitted() {
        let day = 1_700_600_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        let work_ts = day + 16 * 3600;
        let mut sleep = sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep");
        sleep.confidence = bio_spec::Confidence::saturating_from(0.8);
        let mut keys = obs(
            2,
            work_ts,
            DATA_TYPE_KEYSTROKES,
            json!({ "count": 30, "window_secs": 60, "rate_per_min": 30.0 }),
        );
        keys.confidence = bio_spec::Confidence::saturating_from(0.6);
        let feat = last_circadian(&[sleep, keys]).expect("emits");
        // coverage = 1.0; mean evidence = (0.8+0.6)/2 = 0.7
        assert!((feat.confidence.get() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn factors_are_calm_not_chronotype() {
        let day = 1_700_700_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        let work_ts = day + 16 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "in_bed"),
            obs(
                2,
                work_ts,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 20, "window_secs": 60, "rate_per_min": 20.0 }),
            ),
        ];
        let feat = last_circadian(&batch).expect("emits");
        for f in &feat.factors {
            let lower = f.label.to_lowercase();
            assert!(!lower.contains("chronotype"));
            assert!(!lower.contains("night owl"));
            assert!(!lower.contains("disorder"));
            assert!(!lower.contains("fail"));
        }
    }

    #[test]
    fn no_feature_level_magnitude_proxy_path() {
        let src = include_str!("circadian_offset.rs");
        let prod: String = src
            .lines()
            .take_while(|l| !l.contains("mod tests"))
            .collect::<Vec<_>>()
            .join("\n");
        // Must not pull sibling Feature nodes / ids as timing proxies.
        assert!(!prod.contains("SleepDebtNode"));
        assert!(!prod.contains("EnergyScoreNode"));
        assert!(!prod.contains("ActivityBalanceNode"));
        assert!(!prod.contains("FocusScoreNode"));
        assert!(!prod.contains("DeskAwayPresenceNode"));
        assert!(!prod.contains("\"SleepDebt\""));
        assert!(!prod.contains("\"EnergyScore\""));
        assert!(!prod.contains("\"ActivityBalance\""));
        assert!(!prod.contains("\"FocusScore\""));
        assert!(!prod.contains("\"DeskAwayPresence\""));
        assert!(!prod.contains("ctx.features()"));
        // Independent Observation-level node — empty Feature deps.
        assert!(prod.contains("fn depends_on(&self) -> &[NodeId] {\n        &[]\n    }"));
    }

    #[test]
    fn register_catalog_v1_includes_circadian() {
        let day = 1_700_800_000i64;
        let sleep_start = day;
        let sleep_end = day + 8 * 3600;
        let work_ts = day + 16 * 3600;
        let batch = [
            sleep_obs(1, sleep_end, sleep_start, sleep_end, "asleep"),
            obs(
                2,
                work_ts,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 40, "window_secs": 60, "rate_per_min": 40.0 }),
            ),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "CircadianOffset must be registered via register_catalog_v1"
        );
    }

    #[test]
    fn circular_hours_folds_to_half_day() {
        assert!((circular_hours_diff(0, 0) - 0.0).abs() < 1e-9);
        assert!((circular_hours_diff(6 * 3600, 0) - 6.0).abs() < 1e-9);
        assert!((circular_hours_diff(18 * 3600, 0) - 6.0).abs() < 1e-9);
        assert!((circular_hours_diff(12 * 3600, 0) - 12.0).abs() < 1e-9);
    }
}
