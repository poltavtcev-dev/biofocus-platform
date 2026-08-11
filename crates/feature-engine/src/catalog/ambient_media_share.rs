//! `AmbientMediaShare` v1 — calm ambient media-presence share from Now Playing
//! (`docs/06-feature-catalog.md` / ADR-012).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR /
//!   DistractionScore).
//! - **Inputs (required):** `now_playing` Observations with at least one
//!   **closed-set** kind in the window: `music` / `podcast` / `other`
//!   (playing or paused).
//! - **Value (0–100):** share of window samples where
//!   `is_playing && media_kind ∈ {music, podcast, other}` —
//!   `100 * active / total_now_playing_in_window`.
//! - **Omit policy:** empty window, no `now_playing`, or **only-`none` /
//!   only-`unknown`** (no closed-set kinds) → **omit** Feature (no busy-loop).
//! - **Confidence (ADR-007):** single family (`now_playing`); when emitted
//!   `confidence = mean(evidence Observation.confidence)`.
//! - **Explanation factors:** when ≥1 active-playing sample — per-kind shares
//!   among playing samples (`music` / `podcast` / `other`); shares sum to 1.0.
//!   Paused-only closed-set windows emit with value 0 and empty factors.
//! - Calm framing only: “media present during this window” — **not**
//!   “you listen too much” / clinical diagnosis.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_NOW_PLAYING,
    MEDIA_KIND_MUSIC, MEDIA_KIND_OTHER, MEDIA_KIND_PODCAST,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "AmbientMediaShare";

const FACTOR_MUSIC: &str = "music";
const FACTOR_PODCAST: &str = "podcast";
const FACTOR_OTHER: &str = "other";
const LABEL_MUSIC: &str = "Music playing";
const LABEL_PODCAST: &str = "Podcast playing";
const LABEL_OTHER: &str = "Other media playing";

/// DAG node computing [`FEATURE_ID`] (independent; no upstream Feature deps).
#[derive(Debug, Clone, Default)]
pub struct AmbientMediaShareNode;

impl AmbientMediaShareNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for AmbientMediaShareNode {
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
        .filter(|o| o.data_type == DATA_TYPE_NOW_PLAYING && in_window(o, window))
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

    // Only-none / only-unknown / no usable closed-set labels → omit.
    if closed_set.is_empty() {
        return None;
    }

    let total = samples.len() as f64;
    let active = samples.iter().filter(|o| is_active_playing(o)).count();
    let value = (100.0 * active as f64 / total).clamp(0.0, 100.0);

    let factors = playing_kind_factors(&samples);
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

fn media_kind(obs: &Observation) -> Option<&str> {
    obs.payload.get("media_kind").and_then(|v| v.as_str())
}

fn is_playing(obs: &Observation) -> bool {
    obs.payload
        .get("is_playing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn is_closed_set_kind(obs: &Observation) -> bool {
    matches!(
        media_kind(obs),
        Some(MEDIA_KIND_MUSIC | MEDIA_KIND_PODCAST | MEDIA_KIND_OTHER)
    )
}

fn is_active_playing(obs: &Observation) -> bool {
    is_playing(obs) && is_closed_set_kind(obs)
}

fn playing_kind_factors(samples: &[&Observation]) -> Vec<ExplanationFactor> {
    let mut music = 0usize;
    let mut podcast = 0usize;
    let mut other = 0usize;
    for o in samples {
        if !is_active_playing(o) {
            continue;
        }
        match media_kind(o) {
            Some(MEDIA_KIND_MUSIC) => music += 1,
            Some(MEDIA_KIND_PODCAST) => podcast += 1,
            Some(MEDIA_KIND_OTHER) => other += 1,
            _ => {}
        }
    }
    let total = music + podcast + other;
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
    push(&mut factors, FACTOR_MUSIC, LABEL_MUSIC, music);
    push(&mut factors, FACTOR_PODCAST, LABEL_PODCAST, podcast);
    push(&mut factors, FACTOR_OTHER, LABEL_OTHER, other);
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

    fn np_obs(id: u128, ts: i64, kind: &str, playing: bool) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.now_playing",
            DATA_TYPE_NOW_PLAYING,
            json!({
                "media_kind": kind,
                "is_playing": playing
            }),
            1.0,
        )
        .expect("obs")
    }

    fn np_obs_conf(id: u128, ts: i64, kind: &str, playing: bool, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.now_playing",
            DATA_TYPE_NOW_PLAYING,
            json!({
                "media_kind": kind,
                "is_playing": playing
            }),
            confidence,
        )
        .expect("obs")
    }

    fn last_share(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientMediaShareNode::new())
            .expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("AmbientMediaShare")
    }

    #[test]
    fn empty_snapshot_idle_safe() {
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientMediaShareNode::new())
            .expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn none_only_omits_feature() {
        let batch = vec![
            np_obs(1, 1500, "none", false),
            np_obs(2, 1800, "none", false),
        ];
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientMediaShareNode::new())
            .expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "none-only must omit AmbientMediaShare"
        );
    }

    #[test]
    fn unknown_only_omits_feature() {
        let batch = vec![
            np_obs(1, 1500, "unknown", false),
            np_obs(2, 1800, "unknown", true),
        ];
        let mut engine = FeatureEngine::new();
        engine
            .register(AmbientMediaShareNode::new())
            .expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "unknown-only must omit AmbientMediaShare"
        );
    }

    #[test]
    fn rich_playing_music_emits_full_share() {
        let feat = last_share(&[
            np_obs(1, 1500, "music", true),
            np_obs(2, 1800, "music", true),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9, "expected 100 share, got {v}");
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_MUSIC);
        assert!((feat.factors[0].share - 1.0).abs() < 1e-12);
    }

    #[test]
    fn half_playing_half_idle_yields_fifty() {
        let feat = last_share(&[
            np_obs(1, 1500, "music", true),
            np_obs(2, 1800, "none", false),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 50.0).abs() < 1e-9, "expected 50 share, got {v}");
    }

    #[test]
    fn paused_closed_set_emits_zero() {
        let feat = last_share(&[
            np_obs(1, 1500, "music", false),
            np_obs(2, 1800, "podcast", false),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 0.0).abs() < 1e-9, "paused closed-set → 0, got {v}");
        assert!(feat.factors.is_empty(), "no playing → empty factors");
    }

    #[test]
    fn mixed_playing_kinds_factor_shares() {
        let feat = last_share(&[
            np_obs(1, 1200, "music", true),
            np_obs(2, 1500, "podcast", true),
            np_obs(3, 1800, "other", true),
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
            np_obs_conf(1, 1500, "music", true, 0.4),
            np_obs_conf(2, 1800, "music", true, 0.4),
        ]);
        assert!((feat.confidence.get() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_ambient_media_share() {
        let batch = vec![
            np_obs(1, 1500, "music", true),
            np_obs(2, 1800, "music", true),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "AmbientMediaShare must be registered via register_catalog_v1"
        );
    }
}
