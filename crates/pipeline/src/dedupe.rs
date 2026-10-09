//! Deduplication stage — drop duplicate Observations in-memory only.
//!
//! # Deduplication rule (P3-E1-T2)
//!
//! An Observation is treated as a **duplicate** (and dropped from the pipeline
//! view) when **either** of the following was already recorded in the current
//! [`DedupeState`] window:
//!
//! 1. **Same `id`** — `Observation.id` already seen.
//! 2. **Same content key** — `(provider_id, data_type, timestamp_secs, payload
//!    JSON string)` already seen. Payload fingerprint is `serde_json::Value`'s
//!    compact `to_string()` form (stable for equal values constructed the same way).
//!
//! **First occurrence wins** (input order preserved among kept items). Later
//! duplicates are removed from the returned batch only.
//!
//! # Immutability
//!
//! This stage never writes to SQLite and never mutates persisted Observation
//! rows. The seen-set is process-local / in-memory.

use std::collections::HashSet;

use bio_spec::{Observation, ObservationId};

use crate::intake::AcceptedBatch;
use crate::{PipelineResult, PipelineStage};

/// Content fingerprint used for semantic dedupe (independent of `Observation.id`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ContentKey {
    provider_id: String,
    data_type: String,
    timestamp_secs: i64,
    payload_json: String,
}

impl ContentKey {
    fn from_observation(obs: &Observation) -> Self {
        Self {
            provider_id: obs.provider_id.clone(),
            data_type: obs.data_type.clone(),
            timestamp_secs: obs.timestamp.as_secs(),
            payload_json: obs.payload.to_string(),
        }
    }
}

/// In-memory dedupe window (seen-set) spanning one or more batches.
///
/// Idle-safe: pure sync HashSet updates; no threads, timers, or busy-loops.
#[derive(Debug, Default, Clone)]
pub struct DedupeState {
    seen_ids: HashSet<ObservationId>,
    seen_content: HashSet<ContentKey>,
}

impl DedupeState {
    /// Empty seen-set (start of a dedupe window).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct Observation ids recorded in this window.
    #[must_use]
    pub fn seen_id_count(&self) -> usize {
        self.seen_ids.len()
    }

    /// Number of distinct content keys recorded in this window.
    #[must_use]
    pub fn seen_content_count(&self) -> usize {
        self.seen_content.len()
    }

    /// Clear the window (forget all seen ids / content keys).
    pub fn clear(&mut self) {
        self.seen_ids.clear();
        self.seen_content.clear();
    }

    fn is_duplicate(&self, obs: &Observation) -> bool {
        let content = ContentKey::from_observation(obs);
        self.seen_ids.contains(&obs.id) || self.seen_content.contains(&content)
    }

    fn remember(&mut self, obs: &Observation) {
        self.seen_ids.insert(obs.id);
        self.seen_content.insert(ContentKey::from_observation(obs));
    }
}

/// Batch after deduplication (`PipelineStage::Deduped`).
#[derive(Debug, Clone, PartialEq)]
pub struct DedupedBatch {
    observations: Vec<Observation>,
    dropped_count: usize,
    stage: PipelineStage,
}

impl DedupedBatch {
    /// Pipeline stage marker after a successful dedupe call.
    #[must_use]
    pub const fn stage(&self) -> PipelineStage {
        self.stage
    }

    /// Kept Observations (first occurrence of each id / content key).
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Number of kept Observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// `true` when no Observations remain after dedupe (valid idle / all-dup outcome).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// How many Observations were dropped as duplicates in this call.
    #[must_use]
    pub const fn dropped_count(&self) -> usize {
        self.dropped_count
    }

    /// Consume the batch and return owned kept Observations.
    #[must_use]
    pub fn into_observations(self) -> Vec<Observation> {
        self.observations
    }
}

/// Deduplicate a slice of Observations against (and updating) `state`.
///
/// # Contract
/// - **Empty batch** → [`Ok`] with empty [`DedupedBatch`] (idle-friendly).
/// - **Non-empty** → keep first occurrence per rule; drop later duplicates;
///   stage [`PipelineStage::Deduped`].
/// - Pure / synchronous: no I/O, no threads, no busy-loop.
/// - Does **not** touch SQLite.
///
/// # Errors
/// Reserved for future policy failures ([`crate::PipelineError::StageFailed`]).
/// The default T2 path always returns [`Ok`].
pub fn dedupe_observations(
    state: &mut DedupeState,
    batch: &[Observation],
) -> PipelineResult<DedupedBatch> {
    dedupe_owned(state, batch.to_vec())
}

/// Deduplicate an owned Observation list without an extra slice clone.
pub fn dedupe_owned(
    state: &mut DedupeState,
    observations: Vec<Observation>,
) -> PipelineResult<DedupedBatch> {
    let mut kept = Vec::with_capacity(observations.len());
    let mut dropped_count = 0usize;

    for obs in observations {
        if state.is_duplicate(&obs) {
            dropped_count = dropped_count.saturating_add(1);
            continue;
        }
        state.remember(&obs);
        kept.push(obs);
    }

    Ok(DedupedBatch {
        observations: kept,
        dropped_count,
        stage: PipelineStage::Deduped,
    })
}

/// Deduplicate an intake [`AcceptedBatch`] against `state`.
pub fn dedupe_accepted(
    state: &mut DedupeState,
    accepted: AcceptedBatch,
) -> PipelineResult<DedupedBatch> {
    dedupe_owned(state, accepted.into_observations())
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::{DedupeState, dedupe_accepted, dedupe_observations};
    use crate::PipelineStage;
    use crate::intake::accept_observations;

    fn obs_with(id: Uuid, ts: i64, provider: &str, data_type: &str, bpm: u64) -> Observation {
        Observation::try_new(
            id,
            UnixTimestamp::from_secs(ts),
            provider,
            data_type,
            json!({ "bpm": bpm }),
            1.0,
        )
        .expect("valid test observation")
    }

    #[test]
    fn in_batch_duplicate_id_keeps_first() {
        let id = Uuid::nil();
        let a = obs_with(id, 1, "com.biofocus.test", "heart_rate", 60);
        let b = obs_with(id, 2, "com.biofocus.test", "heart_rate", 70);
        let mut state = DedupeState::new();
        let out = dedupe_observations(&mut state, &[a, b]).expect("dedupe ok");
        assert_eq!(out.stage(), PipelineStage::Deduped);
        assert_eq!(out.len(), 1);
        assert_eq!(out.dropped_count(), 1);
        assert_eq!(out.observations()[0].timestamp.as_secs(), 1);
        assert_eq!(state.seen_id_count(), 1);
    }

    #[test]
    fn in_batch_duplicate_content_key_keeps_first() {
        let a = obs_with(
            Uuid::from_u128(1),
            100,
            "com.biofocus.test",
            "heart_rate",
            72,
        );
        // Different id, same content key → duplicate
        let b = obs_with(
            Uuid::from_u128(2),
            100,
            "com.biofocus.test",
            "heart_rate",
            72,
        );
        let mut state = DedupeState::new();
        let out = dedupe_observations(&mut state, &[a, b]).expect("dedupe ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out.dropped_count(), 1);
        assert_eq!(out.observations()[0].id, Uuid::from_u128(1));
        assert_eq!(state.seen_content_count(), 1);
    }

    #[test]
    fn cross_batch_seen_set_drops_duplicates() {
        let first = obs_with(
            Uuid::from_u128(10),
            50,
            "com.biofocus.test",
            "heart_rate",
            65,
        );
        let same_id = obs_with(Uuid::from_u128(10), 51, "com.biofocus.other", "hrv", 1);
        let same_content = obs_with(
            Uuid::from_u128(11),
            50,
            "com.biofocus.test",
            "heart_rate",
            65,
        );
        let unique = obs_with(
            Uuid::from_u128(12),
            60,
            "com.biofocus.test",
            "heart_rate",
            80,
        );

        let mut state = DedupeState::new();
        let batch1 = dedupe_observations(&mut state, &[first]).expect("batch1");
        assert_eq!(batch1.len(), 1);
        assert_eq!(batch1.dropped_count(), 0);

        let batch2 =
            dedupe_observations(&mut state, &[same_id, same_content, unique]).expect("batch2");
        assert_eq!(batch2.len(), 1);
        assert_eq!(batch2.dropped_count(), 2);
        assert_eq!(batch2.observations()[0].id, Uuid::from_u128(12));
        assert_eq!(state.seen_id_count(), 2);
    }

    #[test]
    fn empty_batch_is_ok_idle_friendly() {
        let mut state = DedupeState::new();
        let out = dedupe_observations(&mut state, &[]).expect("empty ok");
        assert!(out.is_empty());
        assert_eq!(out.dropped_count(), 0);
        assert_eq!(out.stage(), PipelineStage::Deduped);
        assert_eq!(state.seen_id_count(), 0);
    }

    #[test]
    fn distinct_observations_all_kept() {
        let batch = vec![
            obs_with(Uuid::from_u128(1), 1, "p", "heart_rate", 60),
            obs_with(Uuid::from_u128(2), 2, "p", "heart_rate", 61),
        ];
        let mut state = DedupeState::new();
        let out = dedupe_observations(&mut state, &batch).expect("ok");
        assert_eq!(out.len(), 2);
        assert_eq!(out.dropped_count(), 0);
    }

    #[test]
    fn dedupe_accepted_from_intake() {
        let a = obs_with(Uuid::from_u128(1), 1, "p", "heart_rate", 60);
        let dup = a.clone();
        let accepted = accept_observations(&[a, dup]).expect("intake");
        let mut state = DedupeState::new();
        let out = dedupe_accepted(&mut state, accepted).expect("dedupe");
        assert_eq!(out.stage(), PipelineStage::Deduped);
        assert_eq!(out.len(), 1);
        assert_eq!(out.dropped_count(), 1);
    }

    #[test]
    fn clear_resets_window() {
        let obs = obs_with(Uuid::from_u128(1), 1, "p", "heart_rate", 60);
        let mut state = DedupeState::new();
        let _ = dedupe_observations(&mut state, &[obs.clone()]).expect("first");
        state.clear();
        let again = dedupe_observations(&mut state, &[obs]).expect("after clear");
        assert_eq!(again.len(), 1);
        assert_eq!(again.dropped_count(), 0);
    }
}
