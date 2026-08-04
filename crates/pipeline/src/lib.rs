//! Data quality, deduplication, and normalization pipeline.
//!
//! Phase 3 entrypoint: accept [`bio_spec::Observation`] batches for processing.
//! Later stages (dedupe → normalize → feature hook) land in subsequent tasks.
//!
//! # Entrypoint (P3-E1-T1)
//!
//! - [`accept_observations`] — primary API (`&[Observation]` → [`AcceptedBatch`])
//! - [`accept_owned`] / [`accept_iter`] — owned / iterator helpers
//!
//! Empty batches return [`Ok`] (idle-friendly). Success stage is
//! [`PipelineStage::AcceptedForProcessing`].

#![forbid(unsafe_code)]

mod error;
mod intake;

pub use error::{PipelineError, PipelineResult};
pub use intake::{accept_iter, accept_observations, accept_owned, AcceptedBatch};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "pipeline";

/// Marker for how far a batch has progressed through the pipeline.
///
/// Additional variants (e.g. `Deduped`, `Normalized`) will be added in T2/T3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Intake completed; batch is accepted for downstream processing.
    AcceptedForProcessing,
}
