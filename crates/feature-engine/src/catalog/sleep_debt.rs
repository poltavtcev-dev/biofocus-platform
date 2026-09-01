//! `SleepDebt` v1 — calm sleep shortfall proxy from `sleep_interval`
//! Observations (`docs/06-feature-catalog.md` / ADR-018).
//!
//! # v1 formula
//!
//! - **Window / step:** 15 minutes / context step. Debt is scored over the
//!   **24h lookback ending at the window end** (not only the 15m Feature window).
//! - **Inputs:** `sleep_interval` with `stage` ∈ {`asleep`, `in_bed`} (or
//!   missing stage — treated as rest). `awake` / `unknown` do not add rest time.
//! - **Rest seconds:** overlap of each qualifying interval with
//!   `[window.end − 86400, window.end]`.
//! - **Target:** 8h (28_800s) personal default — not a clinical prescription.
//! - **Value (0–100):** shortfall share —
//!   `100 × clamp((target − rest) / target, 0, 1)`. 0 = met/exceeded target;
//!   100 = no qualifying rest in the lookback.
//! - **Omit:** no overlapping qualifying `sleep_interval` in the lookback.
//! - **Confidence (ADR-007):** single family (`sleep_interval`).
//! - **Explanation factors:** `rest` share = rest/target (clamped), `shortfall`
//!   share = 1 − rest_share when both present conceptually — emitted as
//!   `rest` / `shortfall` renormalized to sum 1.0.
//! - Not a sleep diagnosis; SpO2 unused.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_SLEEP_INTERVAL,
    SLEEP_STAGE_ASLEEP, SLEEP_STAGE_AWAKE, SLEEP_STAGE_IN_BED, SLEEP_STAGE_UNKNOWN,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{sliding_window_ends_for, snapshot_time_span, window_ending_at};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "SleepDebt";

const LOOKBACK_SECS: i64 = 24 * 60 * 60;
const TARGET_REST_SECS: f64 = 8.0 * 60.0 * 60.0;

const FACTOR_REST: &str = "rest";
const FACTOR_SHORTFALL: &str = "shortfall";
const LABEL_REST: &str = "Rest time";
const LABEL_SHORTFALL: &str = "Shortfall";

/// DAG node computing [`FEATURE_ID`] (independent; no Feature deps).
#[derive(Debug, Default, Clone)]
pub struct SleepDebtNode;

impl SleepDebtNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for SleepDebtNode {
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

    let mut evidence: Vec<&Observation> = Vec::new();
    let mut rest_secs: f64 = 0.0;

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
        let overlap = overlap_secs(start, end, lookback_start, lookback_end);
        if overlap <= 0.0 {
            continue;
        }
        rest_secs += overlap;
        evidence.push(obs);
    }

    if evidence.is_empty() {
        return None;
    }

    let shortfall_ratio = ((TARGET_REST_SECS - rest_secs) / TARGET_REST_SECS).clamp(0.0, 1.0);
    let value = (100.0 * shortfall_ratio).clamp(0.0, 100.0);
    let rest_share = (rest_secs / TARGET_REST_SECS).clamp(0.0, 1.0);
    let shortfall_share = (1.0 - rest_share).clamp(0.0, 1.0);
    let share_sum = rest_share + shortfall_share;
    let factors = if share_sum > 0.0 {
        let mut f = Vec::new();
        if rest_share > 0.0 {
            f.push(ExplanationFactor {
                id: FACTOR_REST.to_owned(),
                label: LABEL_REST.to_owned(),
                share: rest_share / share_sum,
            });
        }
        if shortfall_share > 0.0 {
            f.push(ExplanationFactor {
                id: FACTOR_SHORTFALL.to_owned(),
                label: LABEL_SHORTFALL.to_owned(),
                share: shortfall_share / share_sum,
            });
        }
        f
    } else {
        Vec::new()
    };

    let provenance: Vec<_> = evidence.iter().map(|o| o.id).collect();
    let confidence = single_family_confidence(&evidence);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value),
        provenance,
        confidence,
        factors,
    })
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

fn overlap_secs(a0: i64, a1: i64, b0: i64, b1: i64) -> f64 {
    let start = a0.max(b0);
    let end = a1.min(b1);
    (end - start).max(0) as f64
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

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
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

    fn last_debt(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        engine.register(SleepDebtNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .next_back()
            .expect("SleepDebt")
    }

    #[test]
    fn omits_without_sleep() {
        let mut engine = FeatureEngine::new();
        engine.register(SleepDebtNode::new()).expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn omits_awake_only() {
        let end = 100_000i64;
        let batch = vec![sleep_obs(1, end, end - 3600, end, "awake")];
        let mut engine = FeatureEngine::new();
        engine.register(SleepDebtNode::new()).expect("reg");
        assert!(engine.run(&batch).expect("run").features.is_empty());
    }

    #[test]
    fn full_target_is_zero_debt() {
        let end = 200_000i64;
        let start = end - 28_800;
        let feat = last_debt(&[sleep_obs(1, end, start, end, "asleep")]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v < 0.01, "got {v}");
    }

    #[test]
    fn half_night_is_about_fifty() {
        let end = 300_000i64;
        let start = end - 14_400;
        let feat = last_debt(&[sleep_obs(1, end, start, end, "asleep")]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 50.0).abs() < 0.5, "got {v}");
        assert!(!feat.factors.is_empty());
    }
}
