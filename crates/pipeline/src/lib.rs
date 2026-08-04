//! Data quality, deduplication, and normalization pipeline.
//!
//! Phase 3: accept [`bio_spec::Observation`] batches, then dedupe (normalize →
//! feature hook land in subsequent tasks).
//!
//! # Entrypoint
//!
//! - [`accept_observations`] — intake (`&[Observation]` → [`AcceptedBatch`])
//! - [`dedupe_observations`] / [`dedupe_accepted`] — dedupe against [`DedupeState`]
//!
//! Empty batches return [`Ok`] (idle-friendly) at every stage.
//!
//! # Deduplication rule (P3-E1-T2)
//!
//! See [`dedupe`] module: drop when same `id` **or** same
//! `(provider_id, data_type, timestamp, payload JSON)` already seen in the
//! in-memory window. First wins; SQLite rows are never rewritten.

#![forbid(unsafe_code)]

mod dedupe;
mod error;
mod intake;

pub use dedupe::{
    dedupe_accepted, dedupe_observations, dedupe_owned, DedupedBatch, DedupeState,
};
pub use error::{PipelineError, PipelineResult};
pub use intake::{accept_iter, accept_observations, accept_owned, AcceptedBatch};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "pipeline";

/// Marker for how far a batch has progressed through the pipeline.
///
/// Additional variants (e.g. `Normalized`) will be added in T3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Intake completed; batch is accepted for downstream processing.
    AcceptedForProcessing,
    /// Deduplication completed; kept Observations are unique in the seen window.
    Deduped,
}
