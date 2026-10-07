//! `AttentionStability` v1 — calm focus/switch stability from Feature-level
//! inputs (`docs/06-feature-catalog.md` / ADR-023 / P22-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR;
//!   series may coarsen).
//! - **Inputs (Feature-level):** upstream [`FocusScore`](super::FocusScoreNode)
//!   (**required**) + [`ContextSwitchRate`](super::ContextSwitchRateNode)
//!   (**optional** switch-stability term) for the **same** window. Not a raw
//!   Observation mix; not a parallel FocusScore; **not** DeepWorkScore intensity.
//! - **Compose LOW variance / HIGH stability (0–100):**
//!   - `focus_samples` = FocusScore scalar values whose Feature ends lie in
//!     `[window_start, window_end]`
//!   - if ≥2 samples: `focus_stability = clamp(100 - (max−min), 0, 100)`
//!   - if exactly one Focus sample: `focus_stability = 100` (no swing observed —
//!     **not** a DeepWorkScore Focus-level term)
//!   - `switch_stability = clamp(100 - ContextSwitchRate × 50, 0, 100)`
//! - **Weights:** focus_stability 0.50 / switch_stability 0.50 — **renormalized**
//!   when CSR absent (Focus-stability only).
//! - **Omit policy:** if FocusScore is absent for the step → **omit**.
//!   CSR-only must **not** emit (windows are driven from FocusScore ends).
//! - **Confidence (ADR-007):** expected slots = 2;
//!   `coverage × mean(upstream Feature.confidence)`.
//! - **Explanation factors:** present components — `focus_stability` /
//!   `switch_stability` with calm labels; shares sum to 1.0.
//! - Calm framing only: “focus stability in this window” — **not** ADHD /
//!   “you can’t focus” / burnout / attention-deficit diagnosis.

use bio_spec::{ExplanationFactor, Feature, FeatureValue, TimeWindow};

use crate::catalog::confidence::compute_from_values;
use crate::catalog::context_switch_rate;
use crate::catalog::focus_score;
use crate::catalog::window::window_ending_at;
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "AttentionStability";

const WEIGHT_FOCUS_STAB: f64 = 0.50;
const WEIGHT_SWITCH_STAB: f64 = 0.50;
/// Catalog input families for ADR-007 coverage (Focus / CSR).
const EXPECTED_INPUT_SLOTS: usize = 2;
/// Maps CSR (switches/min) onto the switch-stability component (ADR-023).

const FACTOR_FOCUS_STAB: &str = "focus_stability";
const FACTOR_SWITCH_STAB: &str = "switch_stability";
const LABEL_FOCUS_STAB: &str = "Focus consistency";
const LABEL_SWITCH_STAB: &str = "Switch steadiness";

/// DAG node computing [`FEATURE_ID`]; depends on FocusScore + ContextSwitchRate.
#[derive(Debug, Clone)]
pub struct AttentionStabilityNode {
    deps: Vec<NodeId>,
}

impl AttentionStabilityNode {
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

impl Default for AttentionStabilityNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for AttentionStabilityNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        // Drive windows from FocusScore only — omit-without-Focus is structural
        // (CSR-only windows never create an AttentionStability step).
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
    let focus_feats = focus_samples_in_window(ctx, window);
    if focus_feats.is_empty() {
        return None;
    }

    let focus_values: Vec<f64> = focus_feats
        .iter()
        .filter_map(|f| scalar_value(f).map(|v| v.clamp(0.0, 100.0)))
        .collect();
    if focus_values.is_empty() {
        return None;
    }

    let focus_stability = focus_stability_from_samples(&focus_values);

    // (factor_id, label, catalog_weight, component_score, upstream Features for provenance)
    let mut weighted: Vec<(&str, &str, f64, f64, Vec<&Feature>)> = Vec::new();
    weighted.push((
        FACTOR_FOCUS_STAB,
        LABEL_FOCUS_STAB,
        WEIGHT_FOCUS_STAB,
        focus_stability,
        focus_feats.clone(),
    ));

    if let Some(csr_feat) = upstream_feature(ctx, context_switch_rate::FEATURE_ID, window) {
        let rate = scalar_value(csr_feat)?;
        let switch_stability = crate::catalog::switch_curve::switch_stability(rate);
        weighted.push((
            FACTOR_SWITCH_STAB,
            LABEL_SWITCH_STAB,
            WEIGHT_SWITCH_STAB,
            switch_stability,
            vec![csr_feat],
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
    for (_, _, _, _, feats) in &weighted {
        for feat in feats {
            provenance.extend(feat.provenance.iter().copied());
        }
        // One confidence contribution per catalog input slot (Focus / CSR).
        let slot_conf = feats.iter().map(|f| f.confidence.get()).sum::<f64>() / feats.len() as f64;
        conf_values.push(slot_conf);
    }
    provenance.sort_unstable();
    provenance.dedup();

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

/// FocusScore Features whose window **ends** lie inside `[start, end]` (ADR-023).
fn focus_samples_in_window<'a>(
    ctx: &'a ComputeContext<'_>,
    window: &TimeWindow,
) -> Vec<&'a Feature> {
    let start = window.start.as_secs();
    let end = window.end.as_secs();
    ctx.features()
        .iter()
        .filter(|f| {
            if f.feature_id != focus_score::FEATURE_ID {
                return false;
            }
            let tip = f.time_window.end.as_secs();
            tip >= start && tip <= end
        })
        .collect()
}

fn focus_stability_from_samples(samples: &[f64]) -> f64 {
    if samples.len() >= 2 {
        let mut min_v = f64::INFINITY;
        let mut max_v = f64::NEG_INFINITY;
        for v in samples {
            min_v = min_v.min(*v);
            max_v = max_v.max(*v);
        }
        (100.0 - (max_v - min_v)).clamp(0.0, 100.0)
    } else {
        // Exactly one Focus sample — no swing observed yet.
        100.0
    }
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
        DEEP_WORK_SCORE_ID, FOCUS_SCORE_ID,
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

    fn register_attention(engine: &mut FeatureEngine) {
        register_focus_v1(engine).expect("focus");
        engine
            .register(AttentionStabilityNode::new())
            .expect("attention");
    }

    fn last_attention(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        register_attention(&mut engine);
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability")
    }

    fn expected_focus_stability(focus_feats: &[&Feature], window: &TimeWindow) -> f64 {
        let start = window.start.as_secs();
        let end = window.end.as_secs();
        let samples: Vec<f64> = focus_feats
            .iter()
            .filter(|f| {
                let tip = f.time_window.end.as_secs();
                tip >= start && tip <= end
            })
            .filter_map(|f| match f.value {
                FeatureValue::Scalar(v) => Some(v.clamp(0.0, 100.0)),
                _ => None,
            })
            .collect();
        focus_stability_from_samples(&samples)
    }

    #[test]
    fn empty_snapshot_omits_feature() {
        let mut engine = FeatureEngine::new();
        register_attention(&mut engine);
        let out = engine.run(&[]).expect("run");
        assert!(!out.features.iter().any(|f| f.feature_id == FEATURE_ID));
    }

    #[test]
    fn omit_without_focus_even_with_other_features() {
        // NotificationPressure-only — no FocusScore → no AttentionStability.
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        register_notification_v1(&mut engine).expect("notif");
        engine
            .register(AttentionStabilityNode::new())
            .expect("attention");
        let batch = vec![obs(
            1,
            1500,
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": 4 }),
        )];
        let out = engine.run(&batch).expect("run");
        assert!(
            !out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "AttentionStability must omit without FocusScore"
        );
        assert!(out
            .features
            .iter()
            .any(|f| f.feature_id == "NotificationPressure"));
    }

    #[test]
    fn rich_focus_and_csr_emits_with_factors() {
        // Same-bundle context → CSR ~0 → switch_stability 100.
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
        register_attention(&mut engine);
        let out = engine.run(&batch).expect("run");

        let focus_feats: Vec<&Feature> = out
            .features
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .collect();
        assert!(!focus_feats.is_empty());
        let csr = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == CONTEXT_SWITCH_RATE_ID)
            .expect("CSR");
        let attn = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");

        let FeatureValue::Scalar(rate) = csr.value else {
            panic!("csr scalar");
        };
        let FeatureValue::Scalar(v) = attn.value else {
            panic!("attn scalar");
        };
        let focus_stab = expected_focus_stability(&focus_feats, &attn.time_window);
        let switch_stab = crate::catalog::switch_curve::switch_stability(rate);
        let expected = WEIGHT_FOCUS_STAB * focus_stab + WEIGHT_SWITCH_STAB * switch_stab;
        assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}");
        assert_eq!(attn.factors.len(), 2);
        let share_sum: f64 = attn.factors.iter().map(|x| x.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        assert!(
            (attn
                .factors
                .iter()
                .find(|x| x.id == FACTOR_FOCUS_STAB)
                .unwrap()
                .share
                - 0.50)
                .abs()
                < 1e-12
        );
        assert!(
            (attn
                .factors
                .iter()
                .find(|x| x.id == FACTOR_SWITCH_STAB)
                .unwrap()
                .share
                - 0.50)
                .abs()
                < 1e-12
        );
        assert_eq!(
            attn.factors
                .iter()
                .find(|x| x.id == FACTOR_FOCUS_STAB)
                .unwrap()
                .label,
            LABEL_FOCUS_STAB
        );
        assert_eq!(
            attn.factors
                .iter()
                .find(|x| x.id == FACTOR_SWITCH_STAB)
                .unwrap()
                .label,
            LABEL_SWITCH_STAB
        );
        assert!(attn.factors.iter().all(|x| {
            let lower = x.label.to_lowercase();
            !lower.contains("adhd")
                && !lower.contains("burnout")
                && !lower.contains("can't focus")
                && !lower.contains("cannot focus")
                && !lower.contains("you can't")
        }));
        let start = attn.time_window.start.as_secs();
        let end = attn.time_window.end.as_secs();
        let focus_slot_conf = {
            let in_win: Vec<f64> = focus_feats
                .iter()
                .filter(|f| {
                    let tip = f.time_window.end.as_secs();
                    tip >= start && tip <= end
                })
                .map(|f| f.confidence.get())
                .collect();
            in_win.iter().sum::<f64>() / in_win.len() as f64
        };
        let expected_conf = (focus_slot_conf + csr.confidence.get()) / 2.0;
        assert!((attn.confidence.get() - expected_conf).abs() < 1e-12);
        assert!(!attn.provenance.is_empty());
    }

    #[test]
    fn focus_only_renormalizes_without_csr() {
        // Keystrokes + HRV, no context → FocusScore emits; CSR omits.
        let batch = [
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
        ];
        let mut engine = FeatureEngine::new();
        register_attention(&mut engine);
        let out = engine.run(&batch).expect("run");
        assert!(!out
            .features
            .iter()
            .any(|f| f.feature_id == CONTEXT_SWITCH_RATE_ID));

        let focus_feats: Vec<&Feature> = out
            .features
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .collect();
        let attn = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");
        let FeatureValue::Scalar(v) = attn.value else {
            panic!("attn scalar");
        };
        let focus_stab = expected_focus_stability(&focus_feats, &attn.time_window);
        assert!(
            (v - focus_stab).abs() < 1e-9,
            "Focus-only should equal focus_stability ({focus_stab}), got {v}"
        );
        assert_eq!(attn.factors.len(), 1);
        assert_eq!(attn.factors[0].id, FACTOR_FOCUS_STAB);
        assert!((attn.factors[0].share - 1.0).abs() < 1e-12);

        let tip_focus = focus_feats
            .iter()
            .rev()
            .find(|f| f.time_window == attn.time_window)
            .expect("tip Focus");
        // coverage 1/2 × mean upstream Focus-slot conf (mean of in-window samples)
        let start = attn.time_window.start.as_secs();
        let end = attn.time_window.end.as_secs();
        let in_win: Vec<f64> = focus_feats
            .iter()
            .filter(|f| {
                let tip = f.time_window.end.as_secs();
                tip >= start && tip <= end
            })
            .map(|f| f.confidence.get())
            .collect();
        let mean_focus = in_win.iter().sum::<f64>() / in_win.len() as f64;
        assert!((attn.confidence.get() - 0.5 * mean_focus).abs() < 1e-12);
        // Distinct from DeepWorkScore Focus-level: single-or-stable range ≠ Focus value
        // unless Focus happens to equal focus_stability by coincidence.
        let _ = tip_focus;
    }

    #[test]
    fn single_focus_sample_yields_full_focus_stability() {
        // All evidence at one tip → one FocusScore end → focus_stability = 100
        // (not Focus level — DeepWorkScore would use Focus intensity here).
        let batch = [
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 40, "window_secs": 60, "rate_per_min": 40.0 }),
            ),
            obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
        ];
        let mut engine = FeatureEngine::new();
        register_attention(&mut engine);
        let out = engine.run(&batch).expect("run");
        let focus_count = out
            .features
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .count();
        assert_eq!(focus_count, 1, "expected a single FocusScore sample");
        let focus = out
            .features
            .iter()
            .find(|f| f.feature_id == FOCUS_SCORE_ID)
            .expect("FocusScore");
        let attn = out
            .features
            .iter()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");
        let FeatureValue::Scalar(f) = focus.value else {
            panic!("focus scalar");
        };
        let FeatureValue::Scalar(v) = attn.value else {
            panic!("attn scalar");
        };
        assert!(
            (v - 100.0).abs() < 1e-9,
            "single Focus sample → focus_stability 100, got {v}"
        );
        assert!(
            (f - 100.0).abs() > 1.0,
            "fixture Focus level should be well below 100 so this is not DeepWork math"
        );
    }

    #[test]
    fn multi_focus_range_lowers_focus_stability() {
        // Low typing early, high typing late → Focus ends differ → range > 0.
        let batch = vec![
            obs(
                1,
                900,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 10, "window_secs": 60, "rate_per_min": 10.0 }),
            ),
            obs(2, 900, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
            obs(
                3,
                1800,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 200, "window_secs": 60, "rate_per_min": 200.0 }),
            ),
            obs(4, 1800, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
        ];
        let mut engine = FeatureEngine::new();
        register_attention(&mut engine);
        let out = engine.run(&batch).expect("run");

        let focus_feats: Vec<&Feature> = out
            .features
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .collect();
        assert!(
            focus_feats.len() >= 2,
            "need ≥2 FocusScore samples, got {}",
            focus_feats.len()
        );
        let attn = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");
        let focus_stab = expected_focus_stability(&focus_feats, &attn.time_window);
        assert!(
            focus_stab < 100.0,
            "multi-sample range should lower focus_stability, got {focus_stab}"
        );
        let FeatureValue::Scalar(v) = attn.value else {
            panic!("attn scalar");
        };
        // Focus-only → value equals focus_stability
        assert!((v - focus_stab).abs() < 1e-9);
    }

    #[test]
    fn distinct_from_deep_work_focus_level_intensity() {
        // Same Focus-only single-sample fixture: DeepWork == Focus level;
        // AttentionStability == 100 (consistency), not Focus intensity.
        let batch = [
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 40, "window_secs": 60, "rate_per_min": 40.0 }),
            ),
            obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 45.0 })),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        let focus = out
            .features
            .iter()
            .find(|f| f.feature_id == FOCUS_SCORE_ID)
            .expect("FocusScore");
        let deep = out
            .features
            .iter()
            .find(|f| f.feature_id == DEEP_WORK_SCORE_ID)
            .expect("DeepWorkScore");
        let attn = out
            .features
            .iter()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");
        let FeatureValue::Scalar(f) = focus.value else {
            panic!("focus");
        };
        let FeatureValue::Scalar(d) = deep.value else {
            panic!("deep");
        };
        let FeatureValue::Scalar(a) = attn.value else {
            panic!("attn");
        };
        assert!((d - f).abs() < 1e-9, "DeepWork Focus-only == Focus level");
        assert!((a - 100.0).abs() < 1e-9, "Attention single-sample == 100");
        assert!((a - d).abs() > 1.0, "siblings must differ on this fixture");
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
        register_attention(&mut engine);
        let out = engine.run(&batch).expect("run");
        let attn = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AttentionStability");
        let focus_feats: Vec<&Feature> = out
            .features
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .collect();
        let start = attn.time_window.start.as_secs();
        let end = attn.time_window.end.as_secs();
        let in_win: Vec<f64> = focus_feats
            .iter()
            .filter(|f| {
                let tip = f.time_window.end.as_secs();
                tip >= start && tip <= end
            })
            .map(|f| f.confidence.get())
            .collect();
        let mean_focus = in_win.iter().sum::<f64>() / in_win.len() as f64;
        let expected = 0.5 * mean_focus;
        assert!((attn.confidence.get() - expected).abs() < 1e-12);
        assert!(attn.confidence.get() < 0.5);
    }

    #[test]
    fn register_catalog_v1_includes_attention_stability() {
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
            "AttentionStability must be registered via register_catalog_v1"
        );
        // Sibling DeepWork still present — leaf/sibling formulas untouched.
        assert!(out
            .features
            .iter()
            .any(|f| f.feature_id == DEEP_WORK_SCORE_ID));
    }

    #[test]
    fn last_attention_helper_smoke() {
        let feat = last_attention(&[
            obs(
                1,
                1500,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 100, "window_secs": 60, "rate_per_min": 100.0 }),
            ),
            obs(2, 1500, DATA_TYPE_HRV, json!({ "rmssd_ms": 40.0 })),
        ]);
        assert_eq!(feat.feature_id, FEATURE_ID);
    }
}
