//! `DistractionScore` v1 — calm context-fragmentation proxy from browser
//! categories (`docs/06-feature-catalog.md` / ADR-010).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR).
//! - **Inputs (required):** `browser_category` Observations with at least one
//!   **non-`unknown`** coarse label in the window. Optional upstream
//!   [`super::ContextSwitchRateNode`] when present for the same window.
//! - **Category mix (0–100):** equal-weighted mean of label distraction
//!   weights — `work` 10, `reference` 20, `communication` 45, `shopping` 75,
//!   `entertainment` 90. `unknown` samples are **excluded** from the mix mean.
//! - **Category churn (0–100):** consecutive category-label changes in the
//!   window (including `unknown`) → `min(100, switches * 25)`.
//! - **CSR (optional, 0–100):** `min(100, ContextSwitchRate * 50)` — more
//!   app switching raises the score alongside browser fragmentation.
//! - **Weights:** mix 0.55, churn 0.25, CSR 0.20 — **renormalized** when CSR
//!   absent (mix/churn only).
//! - **Omit policy:** empty window, no `browser_category`, or **only-`unknown`**
//!   thin windows → **omit** Feature (no busy-loop).
//! - **Confidence (ADR-007):** expected slots = 2 (browser / CSR);
//!   `coverage × mean(evidence Observation.confidence)`. Browser-only → 0.5
//!   coverage when obs confidence is 1.0.
//! - **Explanation factors:** present components — `browser_mix`,
//!   `category_churn`, optional `app_switches`; shares sum to 1.0.
//! - Calm framing only: personal context fragmentation — **not** clinical
//!   ADHD / “you are distracted” diagnosis.

use bio_spec::{ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::context_switch_rate;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "DistractionScore";

const DATA_TYPE_BROWSER_CATEGORY: &str = "browser_category";

const WEIGHT_MIX: f64 = 0.55;
const WEIGHT_CHURN: f64 = 0.25;
const WEIGHT_CSR: f64 = 0.20;
/// Catalog input families for ADR-007 coverage (browser / CSR).
const EXPECTED_INPUT_SLOTS: usize = 2;
/// Maps one category switch onto the churn component scale.
const CHURN_PER_SWITCH: f64 = 25.0;
/// Maps CSR (switches/min) onto the optional CSR component scale.

const FACTOR_MIX: &str = "browser_mix";
const FACTOR_CHURN: &str = "category_churn";
const FACTOR_CSR: &str = "app_switches";
const LABEL_MIX: &str = "Browser category mix";
const LABEL_CHURN: &str = "Category changes";
const LABEL_CSR: &str = "App switching";

/// DAG node computing [`FEATURE_ID`]; depends on `ContextSwitchRate` (optional use).
#[derive(Debug, Clone)]
pub struct DistractionScoreNode {
    deps: Vec<NodeId>,
}

impl DistractionScoreNode {
    /// Constructs the catalog node with a dependency on `ContextSwitchRate`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![context_switch_rate::FEATURE_ID.to_owned()],
        }
    }
}

impl Default for DistractionScoreNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for DistractionScoreNode {
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

        let mut features = Vec::new();
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let mut browser: Vec<&Observation> = ctx
        .observations()
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_BROWSER_CATEGORY && in_window(o, window))
        .collect();
    browser.sort_by_key(|o| o.timestamp.as_secs());

    if browser.is_empty() {
        return None;
    }

    let known: Vec<&Observation> = browser
        .iter()
        .copied()
        .filter(|o| {
            o.payload
                .get("category")
                .and_then(|v| v.as_str())
                .is_some_and(|c| c != "unknown" && category_weight(c).is_some())
        })
        .collect();

    // Only-unknown / no usable closed-set labels → omit (thin OS dogfood).
    if known.is_empty() {
        return None;
    }

    let mix = mean_category_weight(&known)?;
    let churn = category_churn_score(&browser);

    // (factor_id, label, catalog_weight, component_score)
    let mut weighted: Vec<(&str, &str, f64, f64)> = Vec::new();
    weighted.push((FACTOR_MIX, LABEL_MIX, WEIGHT_MIX, mix));
    weighted.push((FACTOR_CHURN, LABEL_CHURN, WEIGHT_CHURN, churn));

    let mut present_slots = 1usize;
    if let Some(csr) = upstream_csr(ctx, window) {
        let csr_comp = crate::catalog::switch_curve::switch_load(csr);
        weighted.push((FACTOR_CSR, LABEL_CSR, WEIGHT_CSR, csr_comp));
        present_slots += 1;
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

    let provenance: Vec<_> = browser.iter().map(|o| o.id).collect();
    let evidence: Vec<&Observation> = browser.clone();
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

fn category_weight(category: &str) -> Option<f64> {
    match category {
        "work" => Some(10.0),
        "reference" => Some(20.0),
        "communication" => Some(45.0),
        "shopping" => Some(75.0),
        "entertainment" => Some(90.0),
        _ => None,
    }
}

fn mean_category_weight(known: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in known {
        let Some(cat) = o.payload.get("category").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(w) = category_weight(cat) else {
            continue;
        };
        sum += w;
        n += 1;
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

fn category_churn_score(sorted: &[&Observation]) -> f64 {
    let mut switches = 0usize;
    let mut prev: Option<&str> = None;
    for o in sorted {
        let Some(cat) = o.payload.get("category").and_then(|v| v.as_str()) else {
            continue;
        };
        if let Some(p) = prev {
            if p != cat {
                switches += 1;
            }
        }
        prev = Some(cat);
    }
    (switches as f64 * CHURN_PER_SWITCH).clamp(0.0, 100.0)
}

fn upstream_csr(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<f64> {
    ctx.features()
        .iter()
        .rev()
        .find(|f| {
            f.feature_id == context_switch_rate::FEATURE_ID && f.time_window == *window
        })
        .and_then(|f| match f.value {
            FeatureValue::Scalar(v) => Some(v),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::{register_catalog_v1, register_focus_v1};
    use crate::FeatureEngine;

    fn browser_obs(id: u128, ts: i64, category: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.browser",
            DATA_TYPE_BROWSER_CATEGORY,
            json!({
                "category": category,
                "browser_bundle_id": "com.apple.Safari"
            }),
            1.0,
        )
        .expect("obs")
    }

    fn browser_obs_conf(id: u128, ts: i64, category: &str, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.browser",
            DATA_TYPE_BROWSER_CATEGORY,
            json!({ "category": category }),
            confidence,
        )
        .expect("obs")
    }

    fn context_obs(id: u128, ts: i64, bundle: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.context",
            "context_window",
            json!({ "bundle_id": bundle, "app_name": bundle }),
            1.0,
        )
        .expect("obs")
    }

    fn last_distraction(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        engine.register(DistractionScoreNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("DistractionScore")
    }

    #[test]
    fn empty_snapshot_idle_safe() {
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        engine.register(DistractionScoreNode::new()).expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn unknown_only_omits_feature() {
        let batch = vec![
            browser_obs(1, 1500, "unknown"),
            browser_obs(2, 1800, "unknown"),
        ];
        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("focus");
        engine.register(DistractionScoreNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "unknown-only must omit DistractionScore"
        );
    }

    #[test]
    fn rich_entertainment_yields_high_score() {
        let feat = last_distraction(&[
            browser_obs(1, 1500, "entertainment"),
            browser_obs(2, 1800, "entertainment"),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        // mix 90, churn 0 → (0.55*90 + 0.25*0) / 0.80 = 61.875
        assert!(
            (v - 61.875).abs() < 1e-9,
            "expected stable entertainment blend, got {v}"
        );
        assert!((feat.confidence.get() - 0.5).abs() < 1e-12, "browser-only coverage");
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn work_yields_lower_than_entertainment() {
        let work = last_distraction(&[browser_obs(1, 1500, "work"), browser_obs(2, 1800, "work")]);
        let fun = last_distraction(&[
            browser_obs(3, 1500, "entertainment"),
            browser_obs(4, 1800, "entertainment"),
        ]);
        let FeatureValue::Scalar(v_work) = work.value else {
            panic!("scalar");
        };
        let FeatureValue::Scalar(v_fun) = fun.value else {
            panic!("scalar");
        };
        assert!(
            v_work < v_fun,
            "work should score lower than entertainment: {v_work} vs {v_fun}"
        );
    }

    #[test]
    fn category_churn_raises_score_vs_stable_work() {
        let stable = last_distraction(&[
            browser_obs(1, 1200, "work"),
            browser_obs(2, 1500, "work"),
            browser_obs(3, 1800, "work"),
        ]);
        let churny = last_distraction(&[
            browser_obs(10, 1200, "work"),
            browser_obs(11, 1400, "entertainment"),
            browser_obs(12, 1600, "shopping"),
            browser_obs(13, 1800, "communication"),
        ]);
        let FeatureValue::Scalar(v_stable) = stable.value else {
            panic!("scalar");
        };
        let FeatureValue::Scalar(v_churn) = churny.value else {
            panic!("scalar");
        };
        assert!(
            v_churn > v_stable,
            "churn should raise score: {v_churn} vs {v_stable}"
        );
    }

    #[test]
    fn with_csr_full_confidence_and_three_factors() {
        let batch = vec![
            context_obs(1, 1200, "com.apple.Safari"),
            context_obs(2, 1500, "com.google.Chrome"),
            context_obs(3, 1800, "com.apple.Terminal"),
            browser_obs(4, 1500, "work"),
            browser_obs(5, 1800, "work"),
        ];
        let feat = last_distraction(&batch);
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 3);
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_CSR));
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let feat = last_distraction(&[
            browser_obs_conf(1, 1500, "work", 0.4),
            browser_obs_conf(2, 1800, "work", 0.4),
        ]);
        // coverage 0.5 × mean obs 0.4 = 0.2
        assert!((feat.confidence.get() - 0.2).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_distraction_score() {
        let batch = vec![
            browser_obs(1, 1500, "shopping"),
            browser_obs(2, 1800, "shopping"),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "DistractionScore must be registered via register_catalog_v1"
        );
    }
}
