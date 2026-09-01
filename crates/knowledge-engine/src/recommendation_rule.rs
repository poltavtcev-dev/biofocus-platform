//! Pluggable Recommendation rules (L4 / ADR-009).

use bio_spec::{Feature, Insight, Recommendation, Signal};

use crate::KnowledgeEngineResult;

/// Deterministic rule: Features + Signals + Insights → zero or more Recommendations.
///
/// Rules must not call LLMs, touch SQLite, or invent parallel metric models.
/// Empty `Ok(vec![])` means no match (not an error) — idle-friendly.
pub trait RecommendationRule: Send + Sync {
    /// Stable rule identifier (unique within recommendation registry).
    fn id(&self) -> &str;

    /// Evaluate inputs; empty `Ok(vec![])` means no match (not an error).
    fn evaluate(
        &self,
        features: &[Feature],
        signals: &[Signal],
        insights: &[Insight],
    ) -> KnowledgeEngineResult<Vec<Recommendation>>;
}
