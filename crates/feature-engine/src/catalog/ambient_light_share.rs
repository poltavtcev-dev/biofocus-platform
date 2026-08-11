//! `AmbientLightShare` v1 — calm ambient light-band share from ambient light
//! Observations (`docs/06-feature-catalog.md` / ADR-015 / P16-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR /
//!   AmbientMediaShare / GitActivityRate).
//! - **Inputs (required):** `ambient_light` Observations with at least one
//!   **closed-set** kind in the window: `dark` / `dim` / `moderate` / `bright`.
//! - **Value (0–100):** share of window samples with closed-set kinds —
//!   `100 * closed_set_count / total_ambient_light_in_window`.
//! - **Omit policy:** empty window, no `ambient_light`, or **only-`unknown`**
//!   (no closed-set kinds) → **omit** Feature.
//! - **Optional `level`:** accepted on Observations (0–100) but **not** used in
//!   v1 value / factors (stays coarse band-share only — no lux / camera).
//! - **Confidence (ADR-007):** single family (`ambient_light`); when emitted
//!   `confidence = mean(evidence Observation.confidence)` over all in-window
//!   `ambient_light` samples (including `unknown` when mixed with closed-set).
//! - **Explanation factors:** when ≥1 closed-set sample — per-kind shares among
//!   closed-set samples (`dark` / `dim` / `moderate` / `bright`); shares sum to 1.0.
//! - Calm framing only: “light context during this window” — **not** clinical
//!   lighting advice / “bad lighting harms you”.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_AMBIENT_LIGHT,
    LIGHT_KIND_BRIGHT, LIGHT_KIND_DARK, LIGHT_KIND_DIM, LIGHT_KIND_MODERATE,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "AmbientLightShare";

const FACTOR_DARK: &str = "dark";
const FACTOR_DIM: &str = "dim";
const FACTOR_MODERATE: &str = "moderate";
const FACTOR_BRIGHT: &str = "bright";
const LABEL_DARK: &str = "Dark band";
const LABEL_DIM: &str = "Dim band";
const LABEL_MODERATE: &str = "Moderate band";
const LABEL_BRIGHT: &str = "Bright band";

/// DAG node computing [`FEATURE_ID`] (independent; no upstream Feature deps).
#[derive(Debug, Clone, Default)]
pub struct AmbientLightShareNode;

impl AmbientLightShareNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for AmbientLightShareNode {
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
        for end in sliding_window_ends(min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let mut samples: Vec<&Observation> = ctx
        .observations()
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_AMBIENT_LIGHT && in_window(o, window))
        .collect();
    samples.sort_by_key(|o| o.timestamp.as_secs());

    if samples.is_empty() {
        return None;
    }

    let closed_set: Vec<&Observation> = samples
        .iter()
        .copied()
        .filter(|o| is_closed_set_kind(o))
        .collect();

    // Only-unknown / no usable closed-set labels → omit.
    if closed_set.is_empty() {
        return None;
    }

    let total = samples.len() as f64;
    let closed = closed_set.len() as f64;
    let value = (100.0 * closed / total).clamp(0.0, 100.0);

    let factors = closed_set_kind_factors(&closed_set);
    let provenance: Vec<_> = samples.iter().map(|o| o.id).collect();
    let confidence = single_family_confidence(&samples);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value),
        provenance,
        confidence,
        factors,
    })
}

fn light_kind(obs: &Observation) -> Option<&str> {
    obs.payload.get("light_kind").and_then(|v| v.as_str())
}

fn is_closed_set_kind(obs: &Observation) -> bool {
    matches!(
        light_kind(obs),
        Some(LIGHT_KIND_DARK | LIGHT_KIND_DIM | LIGHT_KIND_MODERATE | LIGHT_KIND_BRIGHT)
    )
}

fn closed_set_kind_factors(closed_set: &[&Observation]) -> Vec<ExplanationFactor> {
    let mut dark = 0usize;
    let mut dim = 0usize;
    let mut moderate = 0usize;
    let mut bright = 0usize;
    for o in closed_set {
        match light_kind(o) {
            Some(LIGHT_KIND_DARK) => dark += 1,
            Some(LIGHT_KIND_DIM) => dim += 1,
            Some(LIGHT_KIND_MODERATE) => moderate += 1,
            Some(LIGHT_KIND_BRIGHT) => bright += 1,
            _ => {}
        }
    }
    let total = dark + dim + moderate + bright;
    if total == 0 {
        return Vec::new();
    }
    let mut factors = Vec::new();
    let push = |factors: &mut Vec<ExplanationFactor>, id: &str, label: &str, n: usize| {
        if n > 0 {
            factors.push(ExplanationFactor {
                id: id.to_owned(),
                label: label.to_owned(),
                share: n as f64 / total as f64,
            });
        }
    };
    push(&mut factors, FACTOR_DARK, LABEL_DARK, dark);
    push(&mut factors, FACTOR_DIM, LABEL_DIM, dim);
    push(&mut factors, FACTOR_MODERATE, LABEL_MODERATE, moderate);
    push(&mut factors, FACTOR_BRIGHT, LABEL_BRIGHT, bright);
    factors
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::register_catalog_v1;
    use crate::FeatureEngine;

    fn light_obs(id: u128, ts: i64, kind: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.ambient_light",
            DATA_TYPE_AMBIENT_LIGHT,
            json!({ "light_kind": kind }),
            1.0,
        )
        .expect("obs")
    }

    fn light_obs_level(id: u128, ts: i64, kind: &str, level: u8) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.ambient_light",
            DATA_TYPE_AMBIENT_LIGHT,
            json!({ "light_kind": kind, "level": level }),
            1.0,
        )
        .expect("obs")
    }

    fn light_obs_conf(id: u128, ts: i64, kind: &str, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.ambient_light",
            DATA_TYPE_AMBIENT_LIGHT,
            json!({ "light_kind": kind }),
            confidence,
        )
        .expect("obs")
    }

    fn last_share(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientLightShareNode::new())
            .expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AmbientLightShare")
    }

    #[test]
    fn empty_snapshot_idle_safe() {
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientLightShareNode::new())
            .expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn unknown_only_omits_feature() {
        let batch = vec![
            light_obs(1, 1500, "unknown"),
            light_obs(2, 1800, "unknown"),
        ];
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientLightShareNode::new())
            .expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "unknown-only must omit AmbientLightShare"
        );
    }

    #[test]
    fn rich_closed_set_emits_full_share() {
        let feat = last_share(&[
            light_obs(1, 1500, "dim"),
            light_obs(2, 1800, "moderate"),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9, "expected 100 share, got {v}");
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn half_unknown_half_dim_yields_fifty() {
        let feat = last_share(&[
            light_obs(1, 1500, "dim"),
            light_obs(2, 1800, "unknown"),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 50.0).abs() < 1e-9, "expected 50 share, got {v}");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_DIM);
        assert!((feat.factors[0].share - 1.0).abs() < 1e-12);
    }

    #[test]
    fn optional_level_does_not_change_v1_share() {
        let with_level = last_share(&[
            light_obs_level(1, 1500, "bright", 90),
            light_obs_level(2, 1800, "dark", 5),
        ]);
        let without = last_share(&[
            light_obs(1, 1500, "bright"),
            light_obs(2, 1800, "dark"),
        ]);
        let FeatureValue::Scalar(a) = with_level.value else {
            panic!("scalar");
        };
        let FeatureValue::Scalar(b) = without.value else {
            panic!("scalar");
        };
        assert!((a - b).abs() < 1e-12);
        assert!((a - 100.0).abs() < 1e-9);
    }

    #[test]
    fn mixed_bands_factor_shares() {
        let feat = last_share(&[
            light_obs(1, 1200, "dark"),
            light_obs(2, 1500, "dim"),
            light_obs(3, 1800, "bright"),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9);
        assert_eq!(feat.factors.len(), 3);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let feat = last_share(&[
            light_obs_conf(1, 1500, "dim", 0.4),
            light_obs_conf(2, 1800, "dim", 0.4),
        ]);
        assert!((feat.confidence.get() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_ambient_light_share() {
        let batch = vec![
            light_obs(1, 1500, "dim"),
            light_obs(2, 1800, "dim"),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "AmbientLightShare must be registered via register_catalog_v1"
        );
    }
}
