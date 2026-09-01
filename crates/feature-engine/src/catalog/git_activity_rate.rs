//! `GitActivityRate` v1 — calm personal VCS cadence from Git activity
//! (`docs/06-feature-catalog.md` / ADR-013).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR /
//!   DistractionScore / AmbientMediaShare).
//! - **Inputs (required):** `git_activity` Observations with at least one
//!   **countable** kind in the window: `commit` / `checkout` / `sync` / `other`.
//! - **Value (≥ 0):** sum of `event_count` (default **1** when absent) for
//!   countable kinds → **events per 15-minute window**.
//! - **Omit policy:** empty window, no `git_activity`, or **only-`idle` /
//!   only-`unknown`** (no countable kinds) → **omit** Feature (no busy-loop).
//! - **Confidence (ADR-007):** single family (`git_activity`); when emitted
//!   `confidence = mean(evidence Observation.confidence)` over countable rows.
//! - **Explanation factors:** when ≥1 countable event — per-kind shares of
//!   summed counts (`commit` / `checkout` / `sync` / `other`); shares sum to 1.0.
//! - Calm framing only: “version-control cadence in this window” — **not**
//!   “you commit too little” / workplace policing / clinical diagnosis.
//! - **Distinct from** `DistractionScore` — does not merge or redefine Browser math.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, ACTIVITY_KIND_CHECKOUT,
    ACTIVITY_KIND_COMMIT, ACTIVITY_KIND_OTHER, ACTIVITY_KIND_SYNC, DATA_TYPE_GIT_ACTIVITY,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "GitActivityRate";

const FACTOR_COMMIT: &str = "commit";
const FACTOR_CHECKOUT: &str = "checkout";
const FACTOR_SYNC: &str = "sync";
const FACTOR_OTHER: &str = "other";
const LABEL_COMMIT: &str = "Commits";
const LABEL_CHECKOUT: &str = "Checkouts";
const LABEL_SYNC: &str = "Sync events";
const LABEL_OTHER: &str = "Other VCS events";

/// DAG node computing [`FEATURE_ID`] (independent; no upstream Feature deps).
#[derive(Debug, Clone, Default)]
pub struct GitActivityRateNode;

impl GitActivityRateNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for GitActivityRateNode {
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
        .filter(|o| o.data_type == DATA_TYPE_GIT_ACTIVITY && in_window(o, window))
        .collect();
    samples.sort_by_key(|o| o.timestamp.as_secs());

    if samples.is_empty() {
        return None;
    }

    let countable: Vec<&Observation> = samples
        .iter()
        .copied()
        .filter(|o| is_countable_kind(o))
        .collect();

    // Only-idle / only-unknown / no usable countable kinds → omit.
    if countable.is_empty() {
        return None;
    }

    let value = countable
        .iter()
        .map(|o| event_count(o) as f64)
        .sum::<f64>()
        .max(0.0);

    let factors = kind_count_factors(&countable);
    let provenance: Vec<_> = countable.iter().map(|o| o.id).collect();
    let confidence = single_family_confidence(&countable);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value),
        provenance,
        confidence,
        factors,
    })
}

fn activity_kind(obs: &Observation) -> Option<&str> {
    obs.payload.get("activity_kind").and_then(|v| v.as_str())
}

fn is_countable_kind(obs: &Observation) -> bool {
    matches!(
        activity_kind(obs),
        Some(
            ACTIVITY_KIND_COMMIT
                | ACTIVITY_KIND_CHECKOUT
                | ACTIVITY_KIND_SYNC
                | ACTIVITY_KIND_OTHER
        )
    )
}

fn event_count(obs: &Observation) -> u64 {
    match obs.payload.get("event_count") {
        Some(v) => v
            .as_u64()
            .or_else(|| {
                v.as_i64()
                    .and_then(|i| u64::try_from(i).ok())
                    .or_else(|| {
                        v.as_f64()
                            .filter(|f| f.is_finite() && *f >= 1.0 && f.fract() == 0.0)
                            .map(|f| f as u64)
                    })
            })
            .filter(|n| *n >= 1)
            .unwrap_or(1),
        None => 1,
    }
}

fn kind_count_factors(samples: &[&Observation]) -> Vec<ExplanationFactor> {
    let mut commit = 0u64;
    let mut checkout = 0u64;
    let mut sync = 0u64;
    let mut other = 0u64;
    for o in samples {
        let n = event_count(o);
        match activity_kind(o) {
            Some(ACTIVITY_KIND_COMMIT) => commit += n,
            Some(ACTIVITY_KIND_CHECKOUT) => checkout += n,
            Some(ACTIVITY_KIND_SYNC) => sync += n,
            Some(ACTIVITY_KIND_OTHER) => other += n,
            _ => {}
        }
    }
    let total = commit + checkout + sync + other;
    if total == 0 {
        return Vec::new();
    }
    let mut factors = Vec::new();
    let push = |factors: &mut Vec<ExplanationFactor>, id: &str, label: &str, n: u64| {
        if n > 0 {
            factors.push(ExplanationFactor {
                id: id.to_owned(),
                label: label.to_owned(),
                share: n as f64 / total as f64,
            });
        }
    };
    push(&mut factors, FACTOR_COMMIT, LABEL_COMMIT, commit);
    push(&mut factors, FACTOR_CHECKOUT, LABEL_CHECKOUT, checkout);
    push(&mut factors, FACTOR_SYNC, LABEL_SYNC, sync);
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

    fn git_obs(id: u128, ts: i64, kind: &str, count: Option<u64>) -> Observation {
        let payload = match count {
            Some(n) => json!({ "activity_kind": kind, "event_count": n }),
            None => json!({ "activity_kind": kind }),
        };
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.git",
            DATA_TYPE_GIT_ACTIVITY,
            payload,
            1.0,
        )
        .expect("obs")
    }

    fn git_obs_conf(id: u128, ts: i64, kind: &str, count: Option<u64>, confidence: f64) -> Observation {
        let payload = match count {
            Some(n) => json!({ "activity_kind": kind, "event_count": n }),
            None => json!({ "activity_kind": kind }),
        };
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.git",
            DATA_TYPE_GIT_ACTIVITY,
            payload,
            confidence,
        )
        .expect("obs")
    }

    fn last_rate(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        engine.register(GitActivityRateNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("GitActivityRate")
    }

    #[test]
    fn empty_snapshot_idle_safe() {
        let mut engine = FeatureEngine::new();
        engine.register(GitActivityRateNode::new()).expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn idle_only_omits_feature() {
        let batch = vec![
            git_obs(1, 1500, "idle", None),
            git_obs(2, 1800, "idle", None),
        ];
        let mut engine = FeatureEngine::new();
        engine.register(GitActivityRateNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "idle-only must omit GitActivityRate"
        );
    }

    #[test]
    fn unknown_only_omits_feature() {
        let batch = vec![
            git_obs(1, 1500, "unknown", Some(1)),
            git_obs(2, 1800, "unknown", Some(2)),
        ];
        let mut engine = FeatureEngine::new();
        engine.register(GitActivityRateNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "unknown-only must omit GitActivityRate"
        );
    }

    #[test]
    fn rich_commit_and_sync_sums_event_counts() {
        let feat = last_rate(&[
            git_obs(1, 1500, "commit", Some(2)),
            git_obs(2, 1800, "sync", Some(3)),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 5.0).abs() < 1e-9, "expected 5 events/window, got {v}");
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn missing_event_count_defaults_to_one() {
        let feat = last_rate(&[
            git_obs(1, 1500, "commit", None),
            git_obs(2, 1800, "other", None),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 2.0).abs() < 1e-9, "default counts → 2, got {v}");
    }

    #[test]
    fn idle_mixed_with_countable_does_not_inflate_rate() {
        let feat = last_rate(&[
            git_obs(1, 1200, "idle", None),
            git_obs(2, 1500, "checkout", Some(1)),
            git_obs(3, 1800, "unknown", Some(9)),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 1.0).abs() < 1e-9, "only checkout counts, got {v}");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_CHECKOUT);
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let feat = last_rate(&[
            git_obs_conf(1, 1500, "commit", Some(1), 0.4),
            git_obs_conf(2, 1800, "commit", Some(1), 0.4),
        ]);
        assert!((feat.confidence.get() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_git_activity_rate() {
        let batch = vec![
            git_obs(1, 1500, "commit", Some(1)),
            git_obs(2, 1800, "sync", Some(1)),
        ];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "GitActivityRate must be registered via register_catalog_v1"
        );
    }

    #[test]
    fn does_not_emit_distraction_score_from_git_alone() {
        let batch = vec![git_obs(1, 1500, "commit", Some(4))];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(out.features.iter().any(|f| f.feature_id == FEATURE_ID));
        assert!(
            out.features
                .iter()
                .all(|f| f.feature_id != crate::DISTRACTION_SCORE_ID),
            "git_activity must not drive DistractionScore"
        );
    }
}
