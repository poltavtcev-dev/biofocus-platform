//! Typed errors for the Feature Engine DAG.

use thiserror::Error;

/// Fallible Feature Engine operations.
pub type FeatureEngineResult<T> = Result<T, FeatureEngineError>;

/// Errors raised while registering nodes or running the DAG.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FeatureEngineError {
    /// Two nodes registered with the same id.
    #[error("duplicate feature DAG node id `{id}`")]
    DuplicateNode {
        /// Conflicting node id.
        id: String,
    },

    /// A node depends on an id that was never registered.
    #[error("node `{node}` depends on unknown node `{missing}`")]
    UnknownDependency {
        /// Node that declared the dependency.
        node: String,
        /// Missing dependency id.
        missing: String,
    },

    /// Dependency graph contains a cycle (not a DAG).
    #[error("feature DAG cycle detected involving node `{node}`")]
    CycleDetected {
        /// One node that participates in the cycle (Kahn leftover).
        node: String,
    },

    /// A registered node returned an error from [`crate::FeatureNode::compute`].
    #[error("feature node `{node}` failed: {message}")]
    NodeFailed {
        /// Node id.
        node: String,
        /// Failure detail from the node.
        message: String,
    },
}
