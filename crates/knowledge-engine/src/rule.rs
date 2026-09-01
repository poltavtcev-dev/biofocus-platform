//! Pluggable Insight rules (see [`crate::rules`] / [`crate::register_insights_v1`]).

use bio_spec::{Feature, Insight, Signal};

use crate::pattern::PatternInputs;
use crate::KnowledgeEngineResult;

/// Deterministic rule: Features + Signals → zero or more Insights with Evidence.
///
/// Rules must not call LLMs, touch SQLite, or invent parallel metric models.
/// Pattern / baseline rules read optional [`PatternInputs`] (recompute-on-read
/// series from Core) — snapshot-only rules ignore it.
pub trait InsightRule: Send + Sync {
    /// Stable rule identifier (unique within one [`crate::KnowledgeEngine`]).
    fn id(&self) -> &str;

    /// Evaluate inputs; empty `Ok(vec![])` means no match (not an error).
    fn evaluate(
        &self,
        features: &[Feature],
        signals: &[Signal],
        pattern: &PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>>;
}
