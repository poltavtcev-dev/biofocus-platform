//! `MeetingDensity` v1 — busy meeting load from `calendar_event` Observations.
//!
//! # v1 formula (documented simplification)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus/Stress).
//! - **Input:** `calendar_event` payloads (`uid` / `start` / `end`; optional
//!   `busy` / `all_day`). Titles not required.
//! - **Meeting:** timed event with `busy != false` (missing `busy` ⇒ busy);
//!   `all_day == true` skipped.
//! - **Value:** merged busy overlap seconds clipped to the window ÷
//!   [`WINDOW_SECS`](crate::catalog::WINDOW_SECS) → fraction in `[0.0, 1.0]`.
//! - **Provenance:** Observation IDs of busy meetings overlapping the window.
//! - **Confidence (ADR-007):** single family (calendar); when emitted,
//!   `confidence = mean(meeting Observation.confidence)`.
//! - No overlapping busy meetings → no Feature for that step.
//! - Tolerates partial calendars (malformed rows skipped).

use bio_spec::{Feature, FeatureValue};

use crate::catalog::calendar_meeting::{
    collect_busy_meetings, meeting_time_span, meetings_overlapping_window, merged_overlap_secs,
};
use crate::catalog::confidence::compute_from_values;
use crate::catalog::window::{sliding_window_ends_for, window_ending_at, WINDOW_SECS};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "MeetingDensity";

/// DAG node computing [`FEATURE_ID`] over sliding windows.
#[derive(Debug, Default, Clone)]
pub struct MeetingDensityNode;

impl MeetingDensityNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for MeetingDensityNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &[]
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let meetings = collect_busy_meetings(ctx.observations());
        let Some((min_ts, max_ts)) = meeting_time_span(&meetings) else {
            return Ok(NodeOutput::empty());
        };

        let mut features = Vec::new();
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
            let window = window_ending_at(end);
            let overlapping = meetings_overlapping_window(&meetings, &window);
            if overlapping.is_empty() {
                continue;
            }

            let overlap = merged_overlap_secs(
                &overlapping,
                window.start.as_secs(),
                window.end.as_secs(),
            );
            let density = (overlap as f64 / WINDOW_SECS as f64).clamp(0.0, 1.0);
            let provenance = overlapping.iter().map(|m| m.observation_id).collect();
            let conf_values: Vec<f64> = overlapping.iter().map(|m| m.confidence.get()).collect();
            let confidence = compute_from_values(1, 1, &conf_values);

            features.push(Feature {
                feature_id: FEATURE_ID.to_owned(),
                time_window: window,
                value: FeatureValue::Scalar(density),
                provenance,
                confidence,
                factors: Vec::new(),
            });
        }

        Ok(NodeOutput::features(features))
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::calendar_meeting::DATA_TYPE_CALENDAR_EVENT;
    use crate::FeatureEngine;

    fn cal_obs(id: u128, start: i64, end: i64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(start),
            "com.biofocus.macos.calendar",
            DATA_TYPE_CALENDAR_EVENT,
            json!({ "uid": format!("evt-{id}"), "start": start, "end": end, "busy": true }),
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn dense_meeting_fills_window() {
        // One 15m meeting ending at 1800 → density 1.0 at end=1800.
        let obs = vec![cal_obs(1, 900, 1800)];
        let mut engine = FeatureEngine::new();
        engine
            .register(MeetingDensityNode::new())
            .expect("register");
        let out = engine.run(&obs).expect("run");
        let last = out
            .features
            .iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .max_by_key(|f| f.time_window.end.as_secs())
            .expect("feature");
        assert_eq!(last.time_window.end.as_secs(), 1800);
        match last.value {
            FeatureValue::Scalar(v) => assert!((v - 1.0).abs() < 1e-9, "got {v}"),
            _ => panic!("expected scalar"),
        }
        assert_eq!(last.provenance.len(), 1);
    }

    #[test]
    fn empty_calendar_yields_empty() {
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
            .register(MeetingDensityNode::new())
            .expect("register");
        let out = engine.run(&[hr]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn half_window_busy_is_half_density() {
        // Last window end=900 → [0, 900]; meeting [450, 900] overlaps 450s → 0.5.
        let obs = vec![cal_obs(1, 450, 900)];
        let mut engine = FeatureEngine::new();
        engine
            .register(MeetingDensityNode::new())
            .expect("register");
        let out = engine.run(&obs).expect("run");
        let last = out
            .features
            .iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .max_by_key(|f| f.time_window.end.as_secs())
            .expect("feature");
        assert_eq!(last.time_window.end.as_secs(), 900);
        match last.value {
            FeatureValue::Scalar(v) => assert!((v - 0.5).abs() < 1e-9, "got {v}"),
            _ => panic!("expected scalar"),
        }
    }
}
