//! `FatigueIndex` v1 (`docs/06-feature-catalog.md`).
//!
//! # v1 formula (documented simplifications)
//!
//! - **Window / step:** 15 minutes / 1 minute.
//! - **Inputs (catalog → v1 mapping):**
//!   - `FocusScore` history → `100 - FocusScore` for the same window (DAG dep).
//!   - Total active hours → cumulative distinct minutes (from snapshot min →
//!     window end) that contain `keystrokes` or `context_window`, scored vs
//!     an 8h reference day: `active_minutes / 480 * 100` (clamped 0–100).
//!   - Baseline HR shift → `(mean_bpm_in_window - baseline_bpm) / 20 * 100`
//!     where baseline is mean `heart_rate.bpm` in the earliest 15m of the
//!     snapshot (falls back to 60 bpm if no early HR).
//! - **Weights:** low-focus 0.50, active-load 0.30, hr-shift 0.20 —
//!   **renormalized** over components that have data.
//! - **Provenance:** Observation IDs of `keystrokes`, `heart_rate`, and
//!   `context_window` inside the Feature window.
//! - **Confidence (ADR-007):** expected slots = 3 (Focus / active / HR);
//!   `coverage × mean(evidence Observation.confidence)`. Empty → omit.
//! - Emits only when at least one component is available.

use bio_spec::{Feature, FeatureValue, Observation, TimeWindow};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::focus_score;
use crate::catalog::window::{
    in_window, sliding_window_ends, snapshot_time_span, window_ending_at, STEP_SECS, WINDOW_SECS,
};
use crate::{
    ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput,
};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "FatigueIndex";

const DATA_TYPE_KEYSTROKES: &str = "keystrokes";
const DATA_TYPE_HEART_RATE: &str = "heart_rate";
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

const WEIGHT_LOW_FOCUS: f64 = 0.50;
const WEIGHT_ACTIVE: f64 = 0.30;
const WEIGHT_HR_SHIFT: f64 = 0.20;
/// Catalog input families for ADR-007 coverage (Focus / active-load / HR).
const EXPECTED_INPUT_SLOTS: usize = 3;

/// Reference active minutes for a full workday (8h) → active score 100.
const ACTIVE_DAY_MINUTES: f64 = 8.0 * 60.0;
/// BPM rise above baseline that maps to hr-shift score 100.
const HR_SHIFT_REF_BPM: f64 = 20.0;
/// Fallback resting baseline when no early HR samples exist.
const DEFAULT_BASELINE_BPM: f64 = 60.0;

/// DAG node computing [`FEATURE_ID`]; depends on [`FocusScoreNode`](focus_score::FocusScoreNode).
#[derive(Debug, Clone)]
pub struct FatigueIndexNode {
    deps: Vec<NodeId>,
}

impl FatigueIndexNode {
    /// Constructs the catalog node with a dependency on `FocusScore`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![focus_score::FEATURE_ID.to_owned()],
        }
    }
}

impl Default for FatigueIndexNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for FatigueIndexNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let Some((min_ts, max_ts)) = snapshot_time_span(ctx.observations()) else {
            return Ok(NodeOutput::empty());
        };

        let baseline_bpm = baseline_heart_rate(ctx.observations(), min_ts);

        let mut features = Vec::new();
        for end in sliding_window_ends(min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window, min_ts, baseline_bpm) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(
    ctx: &ComputeContext<'_>,
    window: &TimeWindow,
    snapshot_min: i64,
    baseline_bpm: f64,
) -> Option<Feature> {
    let in_win: Vec<&Observation> = ctx
        .observations()
        .iter()
        .filter(|o| in_window(o, window))
        .collect();

    let keystrokes: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_KEYSTROKES)
        .collect();
    let heart_rate: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_HEART_RATE)
        .collect();
    let context: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_CONTEXT_WINDOW)
        .collect();

    let mut weighted: Vec<(f64, f64)> = Vec::new();
    let mut present_slots = 0usize;

    if let Some(focus) = upstream_focus(ctx, window) {
        weighted.push((WEIGHT_LOW_FOCUS, (100.0 - focus).clamp(0.0, 100.0)));
        present_slots += 1;
    }

    let active_score = active_load_score(ctx.observations(), snapshot_min, window.end.as_secs());
    if active_score > 0.0 || !keystrokes.is_empty() || !context.is_empty() {
        weighted.push((WEIGHT_ACTIVE, active_score));
        present_slots += 1;
    }

    if let Some(mean_bpm) = mean_bpm(&heart_rate) {
        let drift = ((mean_bpm - baseline_bpm) / HR_SHIFT_REF_BPM * 100.0).clamp(0.0, 100.0);
        weighted.push((WEIGHT_HR_SHIFT, drift));
        present_slots += 1;
    }

    if weighted.is_empty() {
        return None;
    }

    let w_sum: f64 = weighted.iter().map(|(w, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted.iter().map(|(w, s)| w * s).sum::<f64>() / w_sum;

    let mut provenance = Vec::new();
    let mut evidence: Vec<&Observation> = Vec::new();
    for obs in keystrokes
        .iter()
        .chain(heart_rate.iter())
        .chain(context.iter())
    {
        provenance.push(obs.id);
        evidence.push(*obs);
    }

    // Focus slot may contribute without window Observations; still count coverage.
    // When Focus-only (no local obs), use upstream Feature.confidence as evidence mean.
    let confidence = if evidence.is_empty() {
        if let Some(focus_feat) = ctx.features().iter().rev().find(|f| {
            f.feature_id == focus_score::FEATURE_ID && f.time_window == *window
        }) {
            crate::catalog::confidence::compute_from_values(
                EXPECTED_INPUT_SLOTS,
                present_slots,
                &[focus_feat.confidence.get()],
            )
        } else {
            compute_feature_confidence(EXPECTED_INPUT_SLOTS, present_slots, &evidence)
        }
    } else {
        compute_feature_confidence(EXPECTED_INPUT_SLOTS, present_slots, &evidence)
    };

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors: Vec::new(),
    })
}

fn upstream_focus(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<f64> {
    ctx.features()
        .iter()
        .rev()
        .find(|f| f.feature_id == focus_score::FEATURE_ID && f.time_window == *window)
        .and_then(|f| match f.value {
            FeatureValue::Scalar(v) => Some(v),
            _ => None,
        })
}

fn active_load_score(observations: &[Observation], snapshot_min: i64, until_end: i64) -> f64 {
    let mut minutes = std::collections::BTreeSet::new();
    for o in observations {
        if o.data_type != DATA_TYPE_KEYSTROKES && o.data_type != DATA_TYPE_CONTEXT_WINDOW {
            continue;
        }
        let t = o.timestamp.as_secs();
        if t < snapshot_min || t > until_end {
            continue;
        }
        minutes.insert((t / STEP_SECS) * STEP_SECS);
    }
    let active_minutes = minutes.len() as f64;
    (active_minutes / ACTIVE_DAY_MINUTES * 100.0).clamp(0.0, 100.0)
}

fn baseline_heart_rate(observations: &[Observation], snapshot_min: i64) -> f64 {
    // Earliest 15m of the snapshot: [snapshot_min, snapshot_min + WINDOW_SECS].
    let start = snapshot_min;
    let end = snapshot_min.saturating_add(WINDOW_SECS);
    let early_hr: Vec<&Observation> = observations
        .iter()
        .filter(|o| {
            o.data_type == DATA_TYPE_HEART_RATE
                && o.timestamp.as_secs() >= start
                && o.timestamp.as_secs() <= end
        })
        .collect();
    mean_bpm(&early_hr).unwrap_or(DEFAULT_BASELINE_BPM)
}

fn mean_bpm(obs: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(v) = o
            .payload
            .get("bpm")
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite() && *r > 0.0)
        {
            sum += v;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::{register_focus_v1, register_stress_v1};
    use crate::FeatureEngine;

    fn obs(
        id: u128,
        ts: i64,
        data_type: &str,
        payload: serde_json::Value,
    ) -> Observation {
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

    #[test]
    fn elevated_hr_and_low_focus_raise_fatigue_vs_rested() {
        let rested = vec![
            obs(
                1,
                900,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(2, 1000, DATA_TYPE_HEART_RATE, json!({ "bpm": 60.0 })),
            obs(
                3,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(4, 1600, DATA_TYPE_HEART_RATE, json!({ "bpm": 60.0 })),
            obs(
                5,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
        ];
        // Low typing → weaker FocusScore; HR rises vs early baseline.
        let loaded = vec![
            obs(
                11,
                900,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(12, 1000, DATA_TYPE_HEART_RATE, json!({ "bpm": 60.0 })),
            obs(
                13,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 10, "window_secs": 60, "rate_per_min": 10.0 }),
            ),
            obs(14, 1600, DATA_TYPE_HEART_RATE, json!({ "bpm": 85.0 })),
            obs(
                15,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
        ];

        let score_rested = last_fatigue(&run_catalog(&rested));
        let score_loaded = last_fatigue(&run_catalog(&loaded));
        assert!(
            score_loaded > score_rested,
            "loaded {score_loaded} should exceed rested {score_rested}"
        );
        let out = run_catalog(&loaded);
        let last = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("fatigue");
        assert!(!last.provenance.is_empty());
    }

    fn run_catalog(batch: &[Observation]) -> crate::EngineOutput {
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        register_stress_v1(&mut engine).expect("stress");
        engine.run(batch).expect("run")
    }

    fn last_fatigue(out: &crate::EngineOutput) -> f64 {
        out.features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .and_then(|f| match f.value {
                FeatureValue::Scalar(v) => Some(v),
                _ => None,
            })
            .expect("FatigueIndex")
    }

    #[test]
    fn fatigue_requires_focus_score_node() {
        let mut engine = FeatureEngine::new();
        engine
            .register(FatigueIndexNode::new())
            .expect("reg fatigue alone");
        let err = engine.run(&[]).expect_err("unknown FocusScore dep");
        let msg = err.to_string();
        assert!(
            msg.contains("FocusScore"),
            "unexpected err: {msg}"
        );
    }
}
