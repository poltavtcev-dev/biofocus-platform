//! `RecoveryBetweenMeetings` v1 — free-gap minutes between busy meetings.
//!
//! # v1 formula (documented simplification)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus/Stress).
//! - **Input:** same busy `calendar_event` meetings as `MeetingDensity`
//!   (`uid` / `start` / `end`; `busy` / `all_day` rules identical).
//! - **Gap:** free interval `(prev.end → next.start)` between consecutive
//!   busy meetings sorted by start; overlapping / back-to-back → no gap.
//! - **Value:** mean length (minutes) of gaps that intersect the Feature
//!   window. Calm schedule metric — not a clinical recovery score.
//! - **Provenance:** Observation IDs of the two meetings bounding each
//!   counted gap (deduped).
//! - **Confidence (ADR-007):** single family; when emitted,
//!   `confidence = mean(bounding meeting Observation.confidence)`.
//! - No intersecting gaps → no Feature for that step.
//! - Tolerates partial calendars (malformed rows skipped).

use bio_spec::{Feature, FeatureValue};
use uuid::Uuid;

use crate::catalog::calendar_meeting::{
    collect_busy_meetings, gap_intersects_window, inter_meeting_gaps, meeting_time_span,
};
use crate::catalog::confidence::compute_from_values;
use crate::catalog::window::{sliding_window_ends, window_ending_at};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "RecoveryBetweenMeetings";

/// DAG node computing [`FEATURE_ID`] over sliding windows.
#[derive(Debug, Default, Clone)]
pub struct RecoveryBetweenMeetingsNode;

impl RecoveryBetweenMeetingsNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for RecoveryBetweenMeetingsNode {
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

        let gaps = inter_meeting_gaps(&meetings);
        if gaps.is_empty() {
            return Ok(NodeOutput::empty());
        }

        let mut features = Vec::new();
        for end in sliding_window_ends(min_ts, max_ts) {
            let window = window_ending_at(end);
            let mut gap_minutes: Vec<f64> = Vec::new();
            let mut provenance: Vec<Uuid> = Vec::new();

            for &(gap_start, gap_end, left_id, right_id) in &gaps {
                if !gap_intersects_window(gap_start, gap_end, &window) {
                    continue;
                }
                let secs = gap_end - gap_start;
                if secs <= 0 {
                    continue;
                }
                gap_minutes.push(secs as f64 / 60.0);
                if !provenance.contains(&left_id) {
                    provenance.push(left_id);
                }
                if !provenance.contains(&right_id) {
                    provenance.push(right_id);
                }
            }

            if gap_minutes.is_empty() {
                continue;
            }

            let mean = gap_minutes.iter().sum::<f64>() / gap_minutes.len() as f64;
            let conf_values: Vec<f64> = provenance
                .iter()
                .filter_map(|id| {
                    meetings
                        .iter()
                        .find(|m| m.observation_id == *id)
                        .map(|m| m.confidence.get())
                })
                .collect();
            let confidence = compute_from_values(1, 1, &conf_values);
            features.push(Feature {
                feature_id: FEATURE_ID.to_owned(),
                time_window: window,
                value: FeatureValue::Scalar(mean),
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
    use crate::catalog::meeting_density::MeetingDensityNode;
    use crate::catalog::{register_calendar_v1, WINDOW_SECS};
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
    fn mean_gap_minutes_between_meetings() {
        // Meeting A 900–1200, B 1500–1800 → gap 300s = 5 minutes.
        let obs = vec![cal_obs(1, 900, 1200), cal_obs(2, 1500, 1800)];
        let mut engine = FeatureEngine::new();
        engine
            .register(RecoveryBetweenMeetingsNode::new())
            .expect("register");
        let out = engine.run(&obs).expect("run");
        assert!(!out.features.is_empty());
        let last = out
            .features
            .iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .max_by_key(|f| f.time_window.end.as_secs())
            .expect("feature");
        match last.value {
            FeatureValue::Scalar(v) => assert!((v - 5.0).abs() < 1e-9, "got {v}"),
            _ => panic!("expected scalar"),
        }
        assert_eq!(last.provenance.len(), 2);
    }

    #[test]
    fn back_to_back_meetings_yield_no_recovery_feature() {
        let obs = vec![cal_obs(1, 900, 1200), cal_obs(2, 1200, 1500)];
        let mut engine = FeatureEngine::new();
        engine
            .register(RecoveryBetweenMeetingsNode::new())
            .expect("register");
        let out = engine.run(&obs).expect("run");
        assert!(
            out.features
                .iter()
                .all(|f| f.feature_id != FEATURE_ID),
            "no free gaps → no RecoveryBetweenMeetings"
        );
    }

    #[test]
    fn empty_window_no_calendar_is_idle_safe() {
        let mut engine = FeatureEngine::new();
        register_calendar_v1(&mut engine).expect("register");
        let out = engine.run(&[]).expect("run");
        assert!(out.is_empty());
    }

    #[test]
    fn dense_block_has_high_density_and_no_gaps() {
        // Three contiguous 5m blocks filling 900–1800.
        let obs = vec![
            cal_obs(1, 900, 1200),
            cal_obs(2, 1200, 1500),
            cal_obs(3, 1500, 1800),
        ];
        let mut engine = FeatureEngine::new();
        engine
            .register(MeetingDensityNode::new())
            .expect("density");
        engine
            .register(RecoveryBetweenMeetingsNode::new())
            .expect("recovery");
        let out = engine.run(&obs).expect("run");

        let density = out
            .features
            .iter()
            .filter(|f| f.feature_id == "MeetingDensity")
            .max_by_key(|f| f.time_window.end.as_secs())
            .expect("density");
        match density.value {
            FeatureValue::Scalar(v) => assert!((v - 1.0).abs() < 1e-9, "got {v}"),
            _ => panic!("expected scalar"),
        }
        assert!(out
            .features
            .iter()
            .all(|f| f.feature_id != FEATURE_ID));
        assert_eq!(density.time_window.end.as_secs() - density.time_window.start.as_secs(), WINDOW_SECS);
    }
}
