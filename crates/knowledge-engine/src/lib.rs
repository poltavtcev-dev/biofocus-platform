//! Pattern rules and Insight generation with Evidence.
//!
//! Phase 4 (P4-E2-T2): pluggable [`InsightRule`]s + v1 product rules via
//! [`register_insights_v1`]. Types from `bio-spec`. No SQLite, UI, or LLM.
//!
//! # Entrypoint
//!
//! - [`KnowledgeEngine::new`] / [`KnowledgeEngine::register`]
//! - [`register_insights_v1`] — host should call this (or register rules) before
//!   expecting product Insights; empty/unregistered engine still returns `Ok([])`
//! - [`KnowledgeEngine::evaluate`] — Features + Signals → Insights
//! - [`generate_insights`] — empty-engine convenience (always `Ok([])`)
//!
//! Empty rules / empty inputs / no matches → [`Ok`] with empty `Vec` (idle-friendly).

#![forbid(unsafe_code)]

mod engine;
mod error;
mod rule;
mod rules;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, Insight, InsightId, Signal, SignalId, SignalType,
};

pub use engine::{generate_insights, KnowledgeEngine};
pub use error::{KnowledgeEngineError, KnowledgeEngineResult};
pub use rule::InsightRule;
pub use rules::{
    register_insights_v1, ContextSwitchElevatedRule, HighStressPeriodRule,
    CONTEXT_SWITCH_ELEVATED_THRESHOLD, CONTEXT_SWITCH_RATE_ID, FOCUS_SCORE_ID,
    HIGH_STRESS_SIGNAL_TYPE, RULE_CONTEXT_SWITCH_ELEVATED, RULE_HIGH_STRESS_PERIOD, STRESS_INDEX_ID,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "knowledge-engine";
