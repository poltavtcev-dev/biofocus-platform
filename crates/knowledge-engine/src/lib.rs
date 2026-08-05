//! Pattern rules and Insight generation with Evidence.
//!
//! Phase 4 (P4-E2-T1): skeleton — domain types from `bio-spec`, pluggable
//! [`InsightRule`]s, [`KnowledgeEngine::evaluate`] → `Result<Vec<Insight>>`.
//! Product rules (≥2) land in P4-E2-T2. No SQLite, UI, or LLM.
//!
//! # Entrypoint
//!
//! - [`KnowledgeEngine::new`] / [`KnowledgeEngine::register`]
//! - [`KnowledgeEngine::evaluate`] — Features + Signals → Insights
//! - [`generate_insights`] — empty-engine convenience (always `Ok([])`)
//!
//! Empty rules / empty inputs / no matches → [`Ok`] with empty `Vec` (idle-friendly).

#![forbid(unsafe_code)]

mod engine;
mod error;
mod rule;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, Insight, InsightId, Signal, SignalId, SignalType,
};

pub use engine::{generate_insights, KnowledgeEngine};
pub use error::{KnowledgeEngineError, KnowledgeEngineResult};
pub use rule::InsightRule;

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "knowledge-engine";
