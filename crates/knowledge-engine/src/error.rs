//! Typed errors for Knowledge Engine evaluation.

use thiserror::Error;

/// Fallible Knowledge Engine operations.
pub type KnowledgeEngineResult<T> = Result<T, KnowledgeEngineError>;

/// Errors raised while registering rules or producing Insights.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineError {
    /// Two rules registered with the same id.
    #[error("duplicate insight rule id `{id}`")]
    DuplicateRule {
        /// Conflicting rule id.
        id: String,
    },

    /// A registered rule returned a failure from [`crate::InsightRule::evaluate`].
    #[error("insight rule `{rule}` failed: {message}")]
    RuleFailed {
        /// Rule id.
        rule: String,
        /// Failure detail from the rule.
        message: String,
    },
}
