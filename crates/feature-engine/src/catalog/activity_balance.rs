//! `ActivityBalance` v1 — movement vs sedentary proxy from step counts
//! (+ optional Life Event workout) (`docs/06-feature-catalog.md` / ADR-018).
//!
//! # v1 formula
//!
//! - **Window / step:** 15 minutes / context step (default 1 minute; series may coarsen).
//! - **Inputs:** `step_count` (required to emit); optional `life_event` with
//!   `kind == "workout"`.
//! - **Steps component:** sum of `payload.count` in the window, mapped
//!   linearly — **0** at 0 steps, **100** at ≥ 750 steps (≈ brisk 15m).
//! - **Workout boost:** if ≥1 workout Life Event in the window, value =
//!   `max(steps_component, 70)` then blend 85% steps / 15% workout-present (100).
//! - **Omit:** no `step_count` in the window (workout-only does not invent a score).
//! - **Confidence (ADR-007):** expected slots = 2 (steps / workout);
//!   coverage × mean evidence Observation confidence.
//! - **Explanation factors:** present inputs — `step_count` / `workout`
//!   with renormalized shares. Calm composition only — not fitness advice.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_LIFE_EVENT,
    DATA_TYPE_STEP_COUNT, LIFE_EVENT_KIND_WORKOUT,
};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "ActivityBalance";

const EXPECTED_INPUT_SLOTS: usize = 2;
/// Steps in a 15m window that saturate the steps component at 100.
const STEPS_SATURATION: f64 = 750.0;
const WEIGHT_STEPS: f64 = 0.85;
const WEIGHT_WORKOUT: f64 = 0.15;
const WORKOUT_FLOOR: f64 = 70.0;

const FACTOR_STEPS: &str = "step_count";
const FACTOR_WORKOUT: &str = "workout";
const LABEL_STEPS: &str = "Steps";
const LABEL_WORKOUT: &str = "Workout";

/// DAG node computing [`FEATURE_ID`] (independent; no Feature deps).
#[derive(Debug, Default, Clone)]
pub struct ActivityBalanceNode;

impl ActivityBalanceNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for ActivityBalanceNode {
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
    let in_win: Vec<&Observation> = observations
        .iter()
        .filter(|o| in_window(o, window))
        .collect();

    let steps: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_STEP_COUNT)
        .collect();
    if steps.is_empty() {
        return None;
    }

    let step_total = steps
        .iter()
        .filter_map(|o| o.payload.get("count").and_then(json_u64))
        .sum::<u64>() as f64;
    let steps_component = (100.0 * step_total / STEPS_SATURATION).clamp(0.0, 100.0);

    let workouts: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| {
            o.data_type == DATA_TYPE_LIFE_EVENT
                && o.payload
                    .get("kind")
                    .and_then(|v| v.as_str())
                    == Some(LIFE_EVENT_KIND_WORKOUT)
        })
        .collect();

    let mut weighted: Vec<(&str, &str, f64, f64)> =
        vec![(FACTOR_STEPS, LABEL_STEPS, WEIGHT_STEPS, steps_component)];
    let mut present_slots = 1usize;
    let mut evidence: Vec<&Observation> = steps.clone();

    if !workouts.is_empty() {
        let boosted = steps_component.max(WORKOUT_FLOOR);
        weighted[0].3 = boosted;
        weighted.push((FACTOR_WORKOUT, LABEL_WORKOUT, WEIGHT_WORKOUT, 100.0));
        present_slots += 1;
        evidence.extend(workouts.iter().copied());
    }

    let w_sum: f64 = weighted.iter().map(|(_, _, w, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted
        .iter()
        .map(|(_, _, w, s)| w * s)
        .sum::<f64>()
        / w_sum;

    let factors: Vec<ExplanationFactor> = weighted
        .iter()
        .map(|(id, label, w, _)| ExplanationFactor {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            share: w / w_sum,
        })
        .collect();

    let provenance: Vec<_> = evidence.iter().map(|o| o.id).collect();
    let confidence = compute_feature_confidence(EXPECTED_INPUT_SLOTS, present_slots, &evidence);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors,
    })
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

    fn step_obs(id: u128, ts: i64, count: u64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_STEP_COUNT,
            json!({ "count": count }),
            1.0,
        )
        .expect("obs")
    }

    fn workout_obs(id: u128, ts: i64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_LIFE_EVENT,
            json!({ "kind": "workout" }),
            1.0,
        )
        .expect("obs")
    }

    fn last_balance(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        engine.register(ActivityBalanceNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .next_back()
            .expect("ActivityBalance")
    }

    #[test]
    fn omits_without_steps() {
        let mut engine = FeatureEngine::new();
        engine.register(ActivityBalanceNode::new()).expect("reg");
        let out = engine
            .run(&[workout_obs(1, 1000)])
            .expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn emits_from_steps() {
        let feat = last_balance(&[step_obs(1, 1500, 375)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 50.0).abs() < 0.01, "got {v}");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_STEPS);
        assert!((feat.confidence.get() - 0.5).abs() < 0.01);
    }

    #[test]
    fn workout_boosts_and_adds_factor() {
        let feat = last_balance(&[step_obs(1, 1500, 100), workout_obs(2, 1560)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v >= WORKOUT_FLOOR * WEIGHT_STEPS, "got {v}");
        assert_eq!(feat.factors.len(), 2);
        assert!((feat.confidence.get() - 1.0).abs() < 0.01);
    }

    #[test]
    fn register_catalog_v1_includes_activity_balance() {
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("reg");
        assert!(engine
            .run(&[step_obs(1, 2000, 800)])
            .expect("run")
            .features
            .iter()
            .any(|f| f.feature_id == FEATURE_ID));
    }
}
