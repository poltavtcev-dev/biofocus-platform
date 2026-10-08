//! Data quality, deduplication, and normalization pipeline.
//!
//! Phase 3: accept [`bio_spec::Observation`] batches → dedupe → normalize
//! (Feature Engine hook lands in subsequent tasks).
//!
//! # Entrypoint
//!
//! - [`accept_observations`] — intake (`&[Observation]` → [`AcceptedBatch`])
//! - [`dedupe_observations`] / [`dedupe_accepted`] — dedupe against [`DedupeState`]
//! - [`normalize_observations`] / [`normalize_deduped`] — canonical payloads
//! - [`run_quality_pipeline`] — accept → dedupe → normalize in one call
//!
//! Typical flow: `accept_*` → `dedupe_*` → `normalize_*` (or [`run_quality_pipeline`]).
//!
//! Empty batches return [`Ok`] (idle-friendly) at every stage.
//!
//! # Deduplication rule (P3-E1-T2)
//!
//! See [`dedupe`] module: drop when same `id` **or** same
//! `(provider_id, data_type, timestamp, payload JSON)` already seen in the
//! in-memory window. First wins; SQLite rows are never rewritten.
//!
//! # Normalization (P3-E1-T3)
//!
//! See [`normalize`] module: known `data_type` payloads → canon + unit
//! calibration; unknown types pass through unchanged. Unparseable known types
//! are skipped (not an error). SQLite rows are never rewritten.

#![forbid(unsafe_code)]

mod dedupe;
mod error;
mod intake;
mod normalize;
mod quality;
mod source_select;

pub use dedupe::{DedupeState, DedupedBatch, dedupe_accepted, dedupe_observations, dedupe_owned};
pub use error::{PipelineError, PipelineResult};
pub use intake::{AcceptedBatch, accept_iter, accept_observations, accept_owned};
pub use normalize::{
    DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_AMBIENT_LIGHT, DATA_TYPE_CONTEXT_WINDOW,
    DATA_TYPE_HEART_RATE, DATA_TYPE_HRV, DATA_TYPE_KEYSTROKES, DATA_TYPE_NOW_PLAYING,
    DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_SLEEP_INTERVAL, DATA_TYPE_STEP_COUNT, NormalizedBatch,
    normalize_deduped, normalize_observations, normalize_owned,
};
pub use quality::{run_quality_pipeline, run_quality_pipeline_with};
pub use source_select::{SourcePriority, select_sources};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "pipeline";

/// Marker for how far a batch has progressed through the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Intake completed; batch is accepted for downstream processing.
    AcceptedForProcessing,
    /// Deduplication completed; kept Observations are unique in the seen window.
    Deduped,
    /// Normalization completed; known-type payloads are in canonical form.
    Normalized,
    /// One `src.kind` kept per wearable bucket. Storage still has every row.
    SourceSelected,
}
