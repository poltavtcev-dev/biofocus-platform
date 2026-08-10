//! Deterministic markdown reports, versioned prompt packs, and optional local LLM.
//!
//! Phase 4:
//! - **P4-E3-T1:** [`build_report`] → offline [`ReportDocument`] (no network).
//! - **P4-E3-T2:** opt-in [`interpret_report`] / [`interpret_llm_prompt`] against a
//!   local OpenAI-compatible endpoint (Ollama). Default **OFF**.
//!
//! Phase 11:
//! - **P11-E2:** named/versioned [`build_report_with_pack`] over Features /
//!   Insights / Recommendations (ADR-011). Default pack: [`DEFAULT_PROMPT_PACK_ID`]
//!   @ [`DEFAULT_PROMPT_PACK_VERSION`].
//!
//! # Entrypoint
//!
//! - [`build_report`] — `&[Feature]` + `&[Insight]` → [`ReportDocument`] (Phase 4)
//! - [`build_report_with_pack`] — pack `id` + `version` + Features / Insights /
//!   Recommendations → [`ReportDocument`]
//! - [`ReportDocument::markdown`] — calm offline summary
//! - [`ReportDocument::llm_prompt`] — same facts wrapped for interpret-only LLM
//! - [`LocalLlmConfig::from_env`] / [`interpret_report`] — explicit opt-in HTTP
//!
//! Empty inputs → calm minimal markdown (still `Ok`). The crate never computes
//! Features or Recommendations; it only renders values already produced by
//! `feature-engine` / `knowledge-engine`. The LLM path interprets
//! [`ReportDocument::llm_prompt`] only — hosts must not call it on app startup.

#![forbid(unsafe_code)]

mod builder;
mod error;
mod llm;
mod packs;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, FeatureValue, Insight, InsightId, Recommendation,
    RecommendationId, TimeWindow,
};

pub use builder::{build_report, ReportDocument};
pub use error::{ReportEngineError, ReportResult};
pub use llm::{
    interpret_llm_prompt, interpret_report, LocalLlmConfig, LOCAL_LLM_BASE_URL_ENV, LOCAL_LLM_ENV,
    LOCAL_LLM_MODEL_ENV, LOCAL_LLM_TIMEOUT_SECS_ENV,
};
pub use packs::{
    build_report_with_pack, default_prompt_pack, list_prompt_packs, PromptPackRef,
    DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "report-engine";
