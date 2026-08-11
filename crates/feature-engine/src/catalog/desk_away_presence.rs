//! `DeskAwayPresence` v1 — calm away-from-desk / break-or-walk likelihood
//! (`docs/06-feature-catalog.md` / ADR-024 / P23-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with catalog; series may
//!   coarsen).
//! - **Inputs (Observation-level, existing families only):**
//!   - Quiet / absent `keystrokes` (optional boost when emitted)
//!   - Quiet / absent `context_window` (optional boost when emitted)
//!   - Optional `step_count` cadence (walk-like movement)
//!   - Optional `life_event` with `kind == "walk"`
//! - **Emit policy (LOCKED):** require **positive away evidence** —
//!   `walk` Life Event **or** step sum ≥ [`MIN_STEPS_AWAY`]. Quiet input /
//!   context alone is **insufficient** (idle-at-desk vs away indistinguishable)
//!   → **omit**. Substantial typing without a walk (and weak steps) → **omit**.
//! - **No** precise GPS / continuous geo; **no** new Observation `data_type`.
//! - **Value (0–100):** weighted renormalize of present components:
//!   - `walk_event` = 100 when walk Life Event present (weight 0.40)
//!   - `steps` = `clamp(100 * step_sum / 250, 0, 100)` when steps contribute
//!     (weight 0.35)
//!   - `input_quiet` = 100 when typing is not active (weight 0.15)
//!   - context quiet folds into the `input_quiet` factor share when both calm
//!     (documented as input quietness); context evidence still in provenance
//! - **Confidence (ADR-007):** expected slots = 3 (`input_quiet` / `steps` /
//!   `walk_event`); `coverage × mean(evidence Observation.confidence)`.
//! - **Explanation factors:** present components — calm ids only.
//! - Calm framing: “away from desk in this window” — **not** workplace
//!   surveillance / GPS tracking / clinical claims.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_LIFE_EVENT,
    DATA_TYPE_STEP_COUNT, LIFE_EVENT_KIND_WALK,
};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "DeskAwayPresence";

const DATA_TYPE_KEYSTROKES: &str = "keystrokes";
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

/// Minimum summed `step_count` in the window to count as walk-like away evidence.
pub const MIN_STEPS_AWAY: u64 = 40;
/// Steps that map the steps component to 100.
const STEPS_SATURATION: f64 = 250.0;
/// Typing rate (keys/min) at or above which the person looks actively at-desk.
const TYPING_ACTIVE_RATE: f64 = 30.0;

const EXPECTED_INPUT_SLOTS: usize = 3;
const WEIGHT_WALK: f64 = 0.40;
const WEIGHT_STEPS: f64 = 0.35;
const WEIGHT_INPUT_QUIET: f64 = 0.15;
/// Context quiet shares the input_quiet weight budget (ADR factors list three ids).
const WEIGHT_CONTEXT_QUIET: f64 = 0.10;

const FACTOR_INPUT_QUIET: &str = "input_quiet";
const FACTOR_STEPS: &str = "steps";
const FACTOR_WALK: &str = "walk_event";
const LABEL_INPUT_QUIET: &str = "Input quiet";
const LABEL_STEPS: &str = "Steps";
const LABEL_WALK: &str = "Walk event";

/// DAG node computing [`FEATURE_ID`] (independent; Observation-level only).
#[derive(Debug, Clone, Default)]
pub struct DeskAwayPresenceNode;

impl DeskAwayPresenceNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for DeskAwayPresenceNode {
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
    if in_win.is_empty() {
        return None;
    }

    let keystrokes: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_KEYSTROKES)
        .collect();
    let context: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_CONTEXT_WINDOW)
        .collect();
    let steps: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_STEP_COUNT)
        .collect();
    let walks: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| {
            o.data_type == DATA_TYPE_LIFE_EVENT
                && o.payload.get("kind").and_then(|v| v.as_str()) == Some(LIFE_EVENT_KIND_WALK)
        })
        .collect();

    let step_sum = steps
        .iter()
        .filter_map(|o| o.payload.get("count").and_then(json_u64))
        .sum::<u64>();
    let has_walk = !walks.is_empty();
    let has_steps_away = step_sum >= MIN_STEPS_AWAY;
    let typing_active = is_typing_active(&keystrokes);
    let input_quiet = !typing_active;
    let context_quiet = is_context_quiet(&context);

    // Positive away evidence required — quiet alone cannot distinguish idle-at-desk.
    if !has_walk && !has_steps_away {
        return None;
    }
    // Active typing without a walk and only weak steps → ambiguous desk activity.
    if typing_active && !has_walk && step_sum < MIN_STEPS_AWAY.saturating_mul(2) {
        return None;
    }

    // (factor_id, label, weight, score, evidence)
    let mut weighted: Vec<(&str, &str, f64, f64, Vec<&Observation>)> = Vec::new();

    if has_walk {
        weighted.push((FACTOR_WALK, LABEL_WALK, WEIGHT_WALK, 100.0, walks.clone()));
    }

    if has_steps_away || (has_walk && !steps.is_empty()) {
        let steps_score = (100.0 * step_sum as f64 / STEPS_SATURATION).clamp(0.0, 100.0);
        if has_steps_away || step_sum > 0 {
            weighted.push((
                FACTOR_STEPS,
                LABEL_STEPS,
                WEIGHT_STEPS,
                steps_score,
                steps.clone(),
            ));
        }
    }

    // input_quiet factor: combine keyboard quiet + context quiet into one ADR factor id.
    if input_quiet || context_quiet {
        let mut quiet_evidence = Vec::new();
        let mut quiet_weight = 0.0;
        let mut quiet_score_acc = 0.0;
        if input_quiet {
            quiet_weight += WEIGHT_INPUT_QUIET;
            quiet_score_acc += WEIGHT_INPUT_QUIET * 100.0;
            quiet_evidence.extend(keystrokes.iter().copied());
        }
        if context_quiet {
            quiet_weight += WEIGHT_CONTEXT_QUIET;
            quiet_score_acc += WEIGHT_CONTEXT_QUIET * 100.0;
            quiet_evidence.extend(context.iter().copied());
        }
        if quiet_weight > 0.0 {
            let quiet_score = quiet_score_acc / quiet_weight;
            weighted.push((
                FACTOR_INPUT_QUIET,
                LABEL_INPUT_QUIET,
                quiet_weight,
                quiet_score,
                quiet_evidence,
            ));
        }
    }

    if weighted.is_empty() {
        return None;
    }

    let w_sum: f64 = weighted.iter().map(|(_, _, w, _, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted
        .iter()
        .map(|(_, _, w, s, _)| w * s)
        .sum::<f64>()
        / w_sum;

    let factors: Vec<ExplanationFactor> = weighted
        .iter()
        .map(|(id, label, w, _, _)| ExplanationFactor {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            share: w / w_sum,
        })
        .collect();

    let mut evidence: Vec<&Observation> = Vec::new();
    for (_, _, _, _, ev) in &weighted {
        evidence.extend(ev.iter().copied());
    }
    // Present slots = distinct ADR factor families in the weighted set.
    let present_slots = weighted.len().min(EXPECTED_INPUT_SLOTS);
    let confidence = compute_feature_confidence(EXPECTED_INPUT_SLOTS, present_slots, &evidence);

    let mut provenance: Vec<_> = evidence.iter().map(|o| o.id).collect();
    provenance.sort_unstable();
    provenance.dedup();

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors,
    })
}

fn is_typing_active(keystrokes: &[&Observation]) -> bool {
    if keystrokes.is_empty() {
        return false;
    }
    keystrokes.iter().any(|o| {
        o.payload
            .get("rate_per_min")
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite())
            .map(|r| r >= TYPING_ACTIVE_RATE)
            .unwrap_or(false)
    })
}

fn is_context_quiet(context: &[&Observation]) -> bool {
    if context.is_empty() {
        return true;
    }
    let mut bundles = Vec::new();
    for o in context {
        let Some(b) = o.payload.get("bundle_id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !bundles.iter().any(|x| *x == b) {
            bundles.push(b);
        }
    }
    // One (or zero named) bundle → quiet / stable desktop focus; many switches → not quiet.
    bundles.len() <= 1
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

    fn register_desk(engine: &mut FeatureEngine) {
        engine
            .register(DeskAwayPresenceNode::new())
            .expect("desk away");
    }

    fn last_desk(batch: &[Observation]) -> Option<Feature> {
        let mut engine = FeatureEngine::new();
        register_desk(&mut engine);
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
    }

    #[test]
    fn empty_snapshot_omits() {
        let mut engine = FeatureEngine::new();
        register_desk(&mut engine);
        let out = engine.run(&[]).expect("run");
        assert!(!out.features.iter().any(|f| f.feature_id == FEATURE_ID));
    }

    #[test]
    fn quiet_alone_omits_insufficient_evidence() {
        // Quiet keystrokes + quiet context — idle-at-desk vs away indistinguishable.
        let batch = [
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 2, "window_secs": 60, "rate_per_min": 2.0 }),
            ),
            obs(
                2,
                1500,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
        ];
        assert!(
            last_desk(&batch).is_none(),
            "quiet alone must omit (no false presence)"
        );
    }

    #[test]
    fn walk_event_emits_with_factors() {
        let batch = [
            obs(
                1,
                1400,
                DATA_TYPE_LIFE_EVENT,
                json!({ "kind": "walk" }),
            ),
            obs(
                2,
                1500,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
        ];
        let feat = last_desk(&batch).expect("DeskAwayPresence");
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v > 0.0 && v <= 100.0);
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_WALK));
        assert!(feat.factors.iter().all(|f| {
            let lower = f.label.to_lowercase();
            !lower.contains("gps")
                && !lower.contains("surveillance")
                && !lower.contains("employer")
                && !lower.contains("track")
        }));
        assert!(!feat.provenance.is_empty());
    }

    #[test]
    fn steps_away_emits() {
        let batch = [obs(
            1,
            1500,
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 120 }),
        )];
        let feat = last_desk(&batch).expect("DeskAwayPresence");
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        let expected = (100.0 * 120.0 / STEPS_SATURATION).clamp(0.0, 100.0);
        // Steps-only → renormalize to steps component (+ maybe empty quiet with no keystrokes)
        // No keystrokes → input_quiet true with weight; no context → context_quiet true.
        assert!(v >= expected * 0.5, "got {v}, steps component {expected}");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_STEPS));
    }

    #[test]
    fn active_typing_without_walk_omits_weak_steps() {
        let batch = [
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(
                2,
                1500,
                DATA_TYPE_STEP_COUNT,
                json!({ "count": 45 }),
            ),
        ];
        assert!(
            last_desk(&batch).is_none(),
            "active typing + weak steps without walk must omit"
        );
    }

    #[test]
    fn walk_overrides_typing_ambiguity() {
        let batch = [
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(
                2,
                1510,
                DATA_TYPE_LIFE_EVENT,
                json!({ "kind": "walk" }),
            ),
        ];
        let feat = last_desk(&batch).expect("walk wins");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_WALK));
    }

    #[test]
    fn no_geo_payload_keys_in_scoring() {
        // Scoring helpers must not read geo fields (ADR-024).
        let src = include_str!("desk_away_presence.rs");
        let prod: String = src
            .lines()
            .take_while(|l| !l.contains("mod tests"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!prod.contains("latitude"));
        assert!(!prod.contains("longitude"));
        assert!(!prod.contains("geolocation"));
        assert!(!prod.contains("\"gps\""));
        assert!(!prod.contains("data_type_gps"));
    }

    #[test]
    fn register_catalog_v1_includes_desk_away() {
        let batch = [obs(
            1,
            1500,
            DATA_TYPE_LIFE_EVENT,
            json!({ "kind": "walk" }),
        )];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "DeskAwayPresence must be registered via register_catalog_v1"
        );
    }

    #[test]
    fn low_steps_below_threshold_omits() {
        let batch = [obs(
            1,
            1500,
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 10 }),
        )];
        assert!(last_desk(&batch).is_none());
    }
}
