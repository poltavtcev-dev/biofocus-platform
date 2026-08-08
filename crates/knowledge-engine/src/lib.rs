//! Pattern rules, Insight generation, and Recommendations with Evidence.
//!
//! Phase 4 (P4-E2-T2): pluggable [`InsightRule`]s + v1 product rules via
//! [`register_insights_v1`]. Types from `bio-spec`. No SQLite, UI, or LLM.
//! Phase 8 (P8-E2-T1): [`PatternInputs`] + `focus_vs_recent_baseline_v1` (ADR-008).
//! Phase 9 (P9-E2-T1 / ADR-009): [`RecommendationRule`] + `focus_dip_pace_hint_v1`.
//!
//! # Entrypoint
//!
//! - [`KnowledgeEngine::new`] / [`KnowledgeEngine::register`]
//! - [`register_insights_v1`] — host should call this (or register rules) before
//!   expecting product Insights; empty/unregistered engine still returns `Ok([])`
//! - [`register_recommendations_v1`] — Recommendation rules (after Insights)
//! - [`KnowledgeEngine::evaluate`] — Features + Signals → Insights
//! - [`KnowledgeEngine::evaluate_with_pattern`] — + recompute-on-read baseline series
//! - [`KnowledgeEngine::evaluate_recommendations`] — Features + Signals + Insights → Recommendations
//! - [`generate_insights`] — empty-engine convenience (always `Ok([])`)
//!
//! Empty rules / empty inputs / no matches → [`Ok`] with empty `Vec` (idle-friendly).

#![forbid(unsafe_code)]

mod engine;
mod error;
mod pattern;
mod recommendation_rule;
mod rule;
mod rules;

pub use bio_spec::{
    EvidenceRef, Feature, FeatureId, Insight, InsightId, Recommendation, RecommendationId, Signal,
    SignalId, SignalType,
};

pub use engine::{generate_insights, KnowledgeEngine};
pub use error::{KnowledgeEngineError, KnowledgeEngineResult};
pub use pattern::PatternInputs;
pub use recommendation_rule::RecommendationRule;
pub use rule::InsightRule;
pub use rules::{
    register_insights_v1, register_recommendations_v1, ContextSwitchElevatedRule,
    FocusDipPaceHintRule, FocusVsRecentBaselineRule, HighStressPeriodRule,
    CONTEXT_SWITCH_ELEVATED_THRESHOLD, CONTEXT_SWITCH_RATE_ID, FOCUS_BASELINE_CONFIDENCE_GATE,
    FOCUS_BASELINE_DELTA, FOCUS_BASELINE_MAX_WINDOWS, FOCUS_BASELINE_MIN_WINDOWS, FOCUS_SCORE_ID,
    HIGH_STRESS_SIGNAL_TYPE, RULE_CONTEXT_SWITCH_ELEVATED, RULE_FOCUS_DIP_PACE_HINT,
    RULE_FOCUS_VS_RECENT_BASELINE, RULE_HIGH_STRESS_PERIOD, STRESS_INDEX_ID,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "knowledge-engine";
