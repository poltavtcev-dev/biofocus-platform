//! Observation intake — first pipeline stage («accepted for processing»).

use bio_spec::Observation;

use crate::{PipelineResult, PipelineStage};

/// Owned batch that passed intake and is ready for downstream stages.
///
/// Observations are a passthrough of the input (order preserved). Pass to
/// [`crate::dedupe_accepted`] for dedupe, then [`crate::normalize_deduped`].
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptedBatch {
    observations: Vec<Observation>,
    stage: PipelineStage,
}

impl AcceptedBatch {
    /// Pipeline stage marker after a successful intake call.
    #[must_use]
    pub const fn stage(&self) -> PipelineStage {
        self.stage
    }

    /// Borrowed view of accepted Observations (input order).
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Number of accepted Observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// `true` when the batch is empty (valid idle outcome).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Consume the batch and return owned Observations.
    #[must_use]
    pub fn into_observations(self) -> Vec<Observation> {
        self.observations
    }
}

/// Accept a slice of [`Observation`]s for pipeline processing.
///
/// # Contract
/// - **Empty batch** → [`Ok`] with [`AcceptedBatch::is_empty`] (idle-friendly;
///   not an error).
/// - **Non-empty** → [`Ok`] with the same Observations (cloned), stage
///   [`PipelineStage::AcceptedForProcessing`].
/// - Pure / synchronous: no I/O, no threads, no busy-loop.
///
/// # Errors
/// Returns [`Err`] only if a future intake policy rejects the batch
/// ([`crate::PipelineError::IntakeRejected`]). The default T1 path does not reject.
pub fn accept_observations(batch: &[Observation]) -> PipelineResult<AcceptedBatch> {
    accept_owned(batch.to_vec())
}

/// Accept an owned Observation batch without an extra slice clone.
///
/// Same empty-batch contract as [`accept_observations`].
pub fn accept_owned(observations: Vec<Observation>) -> PipelineResult<AcceptedBatch> {
    Ok(AcceptedBatch {
        observations,
        stage: PipelineStage::AcceptedForProcessing,
    })
}

/// Accept Observations from any iterator (collected once, then intake).
///
/// Same empty-batch contract as [`accept_observations`].
pub fn accept_iter<I>(iter: I) -> PipelineResult<AcceptedBatch>
where
    I: IntoIterator<Item = Observation>,
{
    accept_owned(iter.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::{accept_iter, accept_observations, accept_owned, AcceptedBatch};
    use crate::PipelineStage;

    fn sample_observation(n: u64) -> Observation {
        Observation::try_new(
            Uuid::nil(),
            UnixTimestamp::from_secs(n as i64),
            "com.biofocus.test",
            "heart_rate",
            json!({ "bpm": 60 + n }),
            1.0,
        )
        .expect("valid test observation")
    }

    #[test]
    fn accept_observations_happy_path() {
        let batch = vec![sample_observation(1), sample_observation(2)];
        let accepted = accept_observations(&batch).expect("intake succeeds");
        assert_eq!(accepted.stage(), PipelineStage::AcceptedForProcessing);
        assert_eq!(accepted.len(), 2);
        assert!(!accepted.is_empty());
        assert_eq!(accepted.observations()[0].timestamp.as_secs(), 1);
        assert_eq!(accepted.observations()[1].timestamp.as_secs(), 2);
        let owned = accepted.into_observations();
        assert_eq!(owned.len(), 2);
    }

    #[test]
    fn accept_observations_empty_batch_is_ok() {
        let accepted: AcceptedBatch = accept_observations(&[]).expect("empty is Ok");
        assert!(accepted.is_empty());
        assert_eq!(accepted.len(), 0);
        assert_eq!(accepted.stage(), PipelineStage::AcceptedForProcessing);
        assert!(accepted.into_observations().is_empty());
    }

    #[test]
    fn accept_owned_and_iter_match_slice_path() {
        let a = sample_observation(10);
        let b = sample_observation(20);
        let from_owned = accept_owned(vec![a.clone(), b.clone()]).expect("owned");
        let from_iter = accept_iter([a, b]).expect("iter");
        assert_eq!(from_owned, from_iter);
        assert_eq!(from_owned.stage(), PipelineStage::AcceptedForProcessing);
    }
}
