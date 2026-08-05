//! Deterministic markdown reports and offline LLM prompt strings.
//!
//! Phase 4 (P4-E3-T1): build human-readable markdown (+ prompt wrapper) from
//! Features and Insights. No network. Optional local LLM HTTP → P4-E3-T2.
//!
//! # Entrypoint
//!
//! - [`build_report`] — `&[Feature]` + `&[Insight]` → [`ReportDocument`]
//! - [`ReportDocument::markdown`] — calm offline summary
//! - [`ReportDocument::llm_prompt`] — same facts wrapped for interpret-only LLM
//!
//! Empty inputs → calm minimal markdown (still `Ok`). The crate never computes
//! Features; it only renders values already produced by `feature-engine` /
//! `knowledge-engine`.

#![forbid(unsafe_code)]

mod builder;
mod error;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, FeatureValue, Insight, InsightId, TimeWindow,
};

pub use builder::{build_report, ReportDocument};
pub use error::{ReportEngineError, ReportResult};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "report-engine";
