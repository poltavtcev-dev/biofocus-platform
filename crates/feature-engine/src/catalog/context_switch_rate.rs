//! `ContextSwitchRate` v1 — switches/min from `context_window` Observations.
//!
//! # v1 formula (documented simplification)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with `FocusScore`).
//! - **Input:** normalized `context_window` payloads (`bundle_id`).
//! - **Switch:** consecutive (time-sorted) `bundle_id` changes inside the window.
//! - **Value:** `switch_count / 15.0` (switches per nominal window minute).
//! - **Provenance:** IDs of `context_window` Observations inside the window.
//! - **Confidence (ADR-007):** single family; when emitted,
//!   `confidence = mean(context Observation.confidence)`.
//! - Empty window (no `context_window`) → no Feature for that step.

use bio_spec::{Feature, FeatureValue, Observation};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at, WINDOW_SECS,
};
use crate::{
    ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput,
};

/// Stable Feature / node id (`docs/02-domain-model.md`).
pub const FEATURE_ID: &str = "ContextSwitchRate";

const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

/// DAG node computing [`FEATURE_ID`] over sliding windows.
#[derive(Debug, Default, Clone)]
pub struct ContextSwitchRateNode;

impl ContextSwitchRateNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for ContextSwitchRateNode {
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

        let mut context: Vec<&Observation> = ctx
            .observations()
            .iter()
            .filter(|o| o.data_type == DATA_TYPE_CONTEXT_WINDOW)
            .collect();
        context.sort_by_key(|o| o.timestamp.as_secs());

        if context.is_empty() {
            return Ok(NodeOutput::empty());
        }

        let mut features = Vec::new();
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
            let window = window_ending_at(end);
            let in_win: Vec<&Observation> = context
                .iter()
                .copied()
                .filter(|o| in_window(o, &window))
                .collect();
            if in_win.is_empty() {
                continue;
            }

            let switches = count_bundle_switches(&in_win);
            let rate = switches as f64 / (WINDOW_SECS as f64 / 60.0);
            let provenance = in_win.iter().map(|o| o.id).collect();
            let confidence = single_family_confidence(&in_win);

            features.push(Feature {
                feature_id: FEATURE_ID.to_owned(),
                time_window: window,
                value: FeatureValue::Scalar(rate),
                provenance,
                confidence,
                factors: Vec::new(),
            });
        }

        Ok(NodeOutput::features(features))
    }
}

fn count_bundle_switches(sorted: &[&Observation]) -> usize {
    let mut switches = 0usize;
    let mut prev_bundle: Option<String> = None;
    for obs in sorted {
        let Some(bundle) = bundle_id(obs) else {
            continue;
        };
        if let Some(ref prev) = prev_bundle {
            if prev != &bundle {
                switches += 1;
            }
        }
        prev_bundle = Some(bundle);
    }
    switches
}

fn bundle_id(obs: &Observation) -> Option<String> {
    obs.payload
        .get("bundle_id")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::FeatureEngine;

    fn ctx_obs(id: u128, ts: i64, bundle: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.context",
            DATA_TYPE_CONTEXT_WINDOW,
            json!({ "bundle_id": bundle, "app_name": bundle }),
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn counts_switches_per_minute_over_15m() {
        // t=900..1800: A → B → A = 2 switches → rate = 2/15
        let obs = vec![
            ctx_obs(1, 900, "app.a"),
            ctx_obs(2, 1200, "app.b"),
            ctx_obs(3, 1500, "app.a"),
            ctx_obs(4, 1800, "app.a"),
        ];
        let mut engine = FeatureEngine::new();
        engine
            .register(ContextSwitchRateNode::new())
            .expect("register");
        let out = engine.run(&obs).expect("run");

        assert!(!out.features.is_empty());
        let last = out.features.last().expect("last");
        assert_eq!(last.feature_id, FEATURE_ID);
        assert_eq!(last.time_window.end.as_secs(), 1800);
        assert_eq!(last.time_window.start.as_secs(), 1800 - WINDOW_SECS);
        match last.value {
            FeatureValue::Scalar(v) => assert!((v - (2.0 / 15.0)).abs() < 1e-9, "got {v}"),
            _ => panic!("expected scalar"),
        }
        assert_eq!(last.provenance.len(), 4);
    }

    #[test]
    fn no_context_window_yields_empty() {
        let hr = Observation::try_new(
            Uuid::from_u128(9),
            UnixTimestamp::from_secs(100),
            "p",
            "heart_rate",
            json!({ "bpm": 70.0 }),
            1.0,
        )
        .expect("obs");
        let mut engine = FeatureEngine::new();
        engine
            .register(ContextSwitchRateNode::new())
            .expect("register");
        let out = engine.run(&[hr]).expect("run");
        assert!(out.features.is_empty());
    }
}
