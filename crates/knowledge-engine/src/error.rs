//! Typed errors for Knowledge Engine evaluation.

use thiserror::Error;

/// Fallible Knowledge Engine operations.
pub type KnowledgeEngineResult<T> = Result<T, KnowledgeEngineError>;

/// Errors raised while registering rules or producing Insights / Recommendations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineError {
    /// Two rules registered with the same id (Insight or Recommendation registry).
    #[error("duplicate rule id `{id}`")]
    DuplicateRule {
        /// Conflicting rule id.
        id: String,
    },

    /// A registered rule returned a failure from evaluate.
    #[error("rule `{rule}` failed: {message}")]
    RuleFailed {
        /// Rule id.
        rule: String,
        /// Failure detail from the rule.
        message: String,
    },
}
