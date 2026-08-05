//! Deterministic markdown reports and optional local LLM interpretation.
//!
//! Phase 4:
//! - **P4-E3-T1:** [`build_report`] → offline [`ReportDocument`] (no network).
//! - **P4-E3-T2:** opt-in [`interpret_report`] / [`interpret_llm_prompt`] against a
//!   local OpenAI-compatible endpoint (Ollama). Default **OFF**.
//!
//! # Entrypoint
//!
//! - [`build_report`] — `&[Feature]` + `&[Insight]` → [`ReportDocument`]
//! - [`ReportDocument::markdown`] — calm offline summary
//! - [`ReportDocument::llm_prompt`] — same facts wrapped for interpret-only LLM
//! - [`LocalLlmConfig::from_env`] / [`interpret_report`] — explicit opt-in HTTP
//!
//! Empty inputs → calm minimal markdown (still `Ok`). The crate never computes
//! Features; it only renders values already produced by `feature-engine` /
//! `knowledge-engine`. The LLM path interprets [`ReportDocument::llm_prompt`]
//! only — hosts must not call it on app startup.

#![forbid(unsafe_code)]

mod builder;
mod error;
mod llm;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, FeatureValue, Insight, InsightId, TimeWindow,
};

pub use builder::{build_report, ReportDocument};
pub use error::{ReportEngineError, ReportResult};
pub use llm::{
    interpret_llm_prompt, interpret_report, LocalLlmConfig, LOCAL_LLM_BASE_URL_ENV, LOCAL_LLM_ENV,
    LOCAL_LLM_MODEL_ENV, LOCAL_LLM_TIMEOUT_SECS_ENV,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "report-engine";
