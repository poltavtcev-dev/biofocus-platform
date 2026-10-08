//! Combined quality stages: accept → dedupe → normalize.

use bio_spec::Observation;

use crate::PipelineResult;
use crate::dedupe::{DedupeState, dedupe_accepted};
use crate::intake::accept_owned;
use crate::normalize::{NormalizedBatch, normalize_deduped};

/// Run intake → dedupe → normalize on an owned Observation batch.
///
/// Empty input yields an empty [`NormalizedBatch`] (`Ok`, idle-friendly).
/// Reuses [`DedupeState`] across calls for cross-batch dedupe.
pub fn run_quality_pipeline(
    observations: Vec<Observation>,
    dedupe: &mut DedupeState,
) -> PipelineResult<NormalizedBatch> {
    let accepted = accept_owned(observations)?;
    let deduped = dedupe_accepted(dedupe, accepted)?;
    normalize_deduped(deduped)
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::run_quality_pipeline;
    use crate::dedupe::DedupeState;
    use crate::{DATA_TYPE_HEART_RATE, PipelineStage};

    fn sample(n: u64, id: &str) -> Observation {
        Observation::try_new(
            Uuid::parse_str(id).expect("uuid"),
            UnixTimestamp::from_secs(n as i64),
            "com.biofocus.test",
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 60.0 + n as f64 }),
            1.0,
        )
        .expect("valid observation")
    }

    #[test]
    fn empty_batch_is_ok_normalized() {
        let mut dedupe = DedupeState::new();
        let out = run_quality_pipeline(Vec::new(), &mut dedupe).expect("empty");
        assert!(out.is_empty());
        assert_eq!(out.stage(), PipelineStage::Normalized);
    }

    #[test]
    fn happy_path_reaches_normalized() {
        let mut dedupe = DedupeState::new();
        let out = run_quality_pipeline(
            vec![
                sample(1, "0190ecb5-7c2a-7123-8901-23456789abc1"),
                sample(2, "0190ecb5-7c2a-7123-8901-23456789abc2"),
            ],
            &mut dedupe,
        )
        .expect("ok");
        assert_eq!(out.len(), 2);
        assert_eq!(out.stage(), PipelineStage::Normalized);
        assert_eq!(out.observations()[0].payload["bpm"], 61.0);
    }
}
