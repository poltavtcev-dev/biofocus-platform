//! `DeepWorkScore` v1 — calm sustained-focus intensity from Feature-level
//! inputs (`docs/06-feature-catalog.md` / ADR-022 / P21-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR;
//!   series may coarsen).
//! - **Inputs (Feature-level):** upstream [`FocusScore`](super::FocusScoreNode)
//!   (**required**) + [`ContextSwitchRate`](super::ContextSwitchRateNode)
//!   (**optional** stability term) for the **same** window. Idle dropped for
//!   v1. Not a raw Observation mix; not a parallel FocusScore.
//! - **Compose high Focus + low CSR:**
//!   - `focus = FocusScore` (already 0–100)
//!   - `stability = clamp(100 - ContextSwitchRate × 50, 0, 100)`
//! - **Weights:** Focus 0.60 / stability 0.40 — **renormalized** when CSR
//!   absent (Focus-only).
//! - **Omit policy:** if FocusScore is absent for the step → **omit**.
//!   CSR-only must **not** emit (windows are driven from FocusScore ends).
//! - **Confidence (ADR-007):** expected slots = 2;
//!   `coverage × mean(upstream Feature.confidence)`.
//! - **Explanation factors:** present components — `focus` / `stability`
//!   with calm labels; shares sum to 1.0.
//! - Calm framing only: “sustained focus in this window” — **not** clinical
//!   flow state / ADHD / burnout / “you are in flow.”

use bio_spec::{ExplanationFactor, Feature, FeatureValue, TimeWindow};

use crate::catalog::confidence::compute_from_values;
use crate::catalog::context_switch_rate;
use crate::catalog::focus_score;
use crate::catalog::window::window_ending_at;
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "DeepWorkScore";

const WEIGHT_FOCUS: f64 = 0.60;
const WEIGHT_STABILITY: f64 = 0.40;
/// Catalog input families for ADR-007 coverage (Focus / CSR).
const EXPECTED_INPUT_SLOTS: usize = 2;
/// Maps CSR (switches/min) onto the stability component (ADR-022).
const CSR_SCORE_SCALE: f64 = 50.0;

const FACTOR_FOCUS: &str = "focus";
const FACTOR_STABILITY: &str = "stability";
const LABEL_FOCUS: &str = "Focus depth";
const LABEL_STABILITY: &str = "App stability";

/// DAG node computing [`FEATURE_ID`]; depends on FocusScore + ContextSwitchRate.
#[derive(Debug, Clone)]
pub struct DeepWorkScoreNode {
    deps: Vec<NodeId>,
}

impl DeepWorkScoreNode {
    /// Constructs the catalog node with Feature-level DAG dependencies.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![
                focus_score::FEATURE_ID.to_owned(),
                context_switch_rate::FEATURE_ID.to_owned(),
            ],
        }
    }
}

impl Default for DeepWorkScoreNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for DeepWorkScoreNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        // Drive windows from FocusScore only — omit-without-Focus is structural
        // (CSR-only windows never create a DeepWorkScore step).
        let mut ends: Vec<i64> = ctx
            .features()
            .iter()
            .filter(|f| f.feature_id == focus_score::FEATURE_ID)
            .map(|f| f.time_window.end.as_secs())
            .collect();
        ends.sort_unstable();
        ends.dedup();

        let mut features = Vec::new();
        for end in ends {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let focus_feat = upstream_feature(ctx, focus_score::FEATURE_ID, window)?;
    let focus = scalar_value(focus_feat)?.clamp(0.0, 100.0);

    // (factor_id, label, catalog_weight, component_score, upstream Feature)
    let mut weighted: Vec<(&str, &str, f64, f64, &Feature)> = Vec::new();
    weighted.push((FACTOR_FOCUS, LABEL_FOCUS, WEIGHT_FOCUS, focus, focus_feat));

    if let Some(csr_feat) = upstream_feature(ctx, context_switch_rate::FEATURE_ID, window) {
        let rate = scalar_value(csr_feat)?;
        let stability = (100.0 - rate * CSR_SCORE_SCALE).clamp(0.0, 100.0);
        weighted.push((
            FACTOR_STABILITY,
            LABEL_STABILITY,
            WEIGHT_STABILITY,
            stability,
            csr_feat,
        ));
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

    let mut provenance = Vec::new();
    let mut conf_values = Vec::new();
    for (_, _, _, _, feat) in &weighted {
        provenance.extend(feat.provenance.iter().copied());
        conf_values.push(feat.confidence.get());
    }

    let present_slots = weighted.len();
    let confidence = compute_from_values(EXPECTED_INPUT_SLOTS, present_slots, &conf_values);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors,
    })
}

fn upstream_feature<'a>(
    ctx: &'a ComputeContext<'_>,
    feature_id: &str,
    window: &TimeWindow,
) -> Option<&'a Feature> {
    ctx.features()
        .iter()
        .rev()
        .find(|f| f.feature_id == feature_id && f.time_window == *window)
}

fn scalar_value(feat: &Feature) -> Option<f64> {
    match feat.value {
        FeatureValue::Scalar(v) => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        FeatureValue, Observation, UnixTimestamp, DATA_TYPE_NOTIFICATION_EVENT,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::{
        register_catalog_v1, register_focus_v1, register_notification_v1, CONTEXT_SWITCH_RATE_ID,
        FOCUS_SCORE_ID,
    };
    use crate::FeatureEngine;

    const DATA_TYPE_KEYSTROKES: &str = "keystrokes";
    const DATA_TYPE_HRV: &str = "hrv";
    const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

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

    fn obs_conf(
        id: u128,
        ts: i64,
        data_type: &str,
        payload: serde_json::Value,
        confidence: f64,
    ) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            data_type,
            payload,
            confidence,
        )
        .expect("obs")
    }

    fn register_deep_work(engine: &mut FeatureEngine) {
        register_focus_v1(engine).expect("focus");
        engine
            .register(DeepWorkScoreNode::new())
            .expect("deep work");
    }

    fn last_deep_work(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        register_deep_work(&mut engine);
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("DeepWorkScore")
    }

    #[test]
    fn empty_snapshot_omits_feature() {
        let mut engine = FeatureEngine::new();
        register_deep_work(&mut engine);
        let out = engine.run(&[]).expect("run");
        assert!(!out.features.iter().any(|f| f.feature_id == FEATURE_ID));
    }

    #[test]
    fn omit_without_focus_even_with_other_features() {
        // NotificationPressure-only — no FocusScore → no DeepWorkScore.
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        register_notification_v1(&mut engine).expect("notif");
        engine
            .register(DeepWorkScoreNode::new())
            .expect("deep work");
        let batch = vec![obs(
            1,
            1500,
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": 4 }),
        )];
        let out = engine.run(&batch).expect("run");
        assert!(
            !out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "DeepWorkScore must omit without FocusScore"
        );
        assert!(out
            .features
            .iter()
            .any(|f| f.feature_id == "NotificationPressure"));
    }

    #[test]
    fn rich_focus_and_csr_emits_with_factors() {
        // Same-bundle context → CSR ~0 → stability 100; high Focus.
        let batch = vec![
            obs(
                1,
                1000,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
            obs(
                2,
                1400,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
            obs(
                3,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(4, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
            obs(
                5,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide" }),
            ),
        ];
        let mut engine = FeatureEngine::new();
        register_deep_work(&mut engine);
        let out = engine.run(&batch).expect("run");

        let focus = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FOCUS_SCORE_ID)
            .expect("FocusScore");
        let csr = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == CONTEXT_SWITCH_RATE_ID)
            .expect("CSR");
        let deep = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("DeepWorkScore");

        let FeatureValue::Scalar(f) = focus.value else {
            panic!("focus scalar");
        };
        let FeatureValue::Scalar(rate) = csr.value else {
            panic!("csr scalar");
        };
        let FeatureValue::Scalar(v) = deep.value else {
            panic!("deep scalar");
        };
        let stability = (100.0 - rate * CSR_SCORE_SCALE).clamp(0.0, 100.0);
        let expected = WEIGHT_FOCUS * f + WEIGHT_STABILITY * stability;
        assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}");
        assert_eq!(deep.factors.len(), 2);
        let share_sum: f64 = deep.factors.iter().map(|x| x.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        assert!((deep.factors.iter().find(|x| x.id == FACTOR_FOCUS).unwrap().share - 0.60).abs()
            < 1e-12);
        assert!(
            (deep.factors.iter().find(|x| x.id == FACTOR_STABILITY).unwrap().share - 0.40).abs()
                < 1e-12
        );
        assert!(deep.factors.iter().all(|x| {
            let lower = x.label.to_lowercase();
            !lower.contains("flow")
                && !lower.contains("burnout")
                && !lower.contains("adhd")
                && !lower.contains("you are")
        }));
        assert!((deep.confidence.get() - 1.0).abs() < 1e-12);
        assert!(!deep.provenance.is_empty());
    }

    #[test]
    fn focus_only_renormalizes_without_csr() {
        // Keystrokes + HRV, no context → FocusScore emits; CSR omits.
        let feat = last_deep_work(&[
            obs(
                1,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
            obs(
                3,
                1800,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
        ]);
        let mut engine = FeatureEngine::new();
        register_deep_work(&mut engine);
        let out = engine
            .run(&[
                obs(
                    1,
                    1200,
                    DATA_TYPE_KEYSTROKES,
                    json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
                ),
                obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
                obs(
                    3,
                    1800,
                    DATA_TYPE_KEYSTROKES,
                    json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
                ),
            ])
            .expect("run");
        assert!(!out
            .features
            .iter()
            .any(|f| f.feature_id == CONTEXT_SWITCH_RATE_ID));
        let focus = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FOCUS_SCORE_ID)
            .expect("FocusScore");
        let FeatureValue::Scalar(f) = focus.value else {
            panic!("focus scalar");
        };
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("deep scalar");
        };
        assert!((v - f).abs() < 1e-9, "Focus-only should equal FocusScore");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_FOCUS);
        assert!((feat.factors[0].share - 1.0).abs() < 1e-12);
        // coverage 1/2 × mean upstream conf
        assert!((feat.confidence.get() - 0.5 * focus.confidence.get()).abs() < 1e-12);
    }

    #[test]
    fn low_upstream_confidence_lowers_feature() {
        let batch = [
            obs_conf(
                1,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
                0.6,
            ),
            obs_conf(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 }), 0.6),
            obs_conf(
                3,
                1800,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
                0.6,
            ),
        ];
        let mut engine = FeatureEngine::new();
        register_deep_work(&mut engine);
        let out = engine.run(&batch).expect("run");
        let focus = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FOCUS_SCORE_ID)
            .expect("FocusScore");
        let deep = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("DeepWorkScore");
        // Focus-only: coverage 1/2 × upstream Focus confidence
        let expected = 0.5 * focus.confidence.get();
        assert!((deep.confidence.get() - expected).abs() < 1e-12);
        assert!(deep.confidence.get() < 0.5);
    }

    #[test]
    fn register_catalog_v1_includes_deep_work_score() {
        let batch = vec![
            obs(
                1,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 100, "window_secs": 60, "rate_per_min": 100.0 }),
            ),
            obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 40.0 })),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "DeepWorkScore must be registered via register_catalog_v1"
        );
    }
}
