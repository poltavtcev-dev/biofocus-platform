//! Feature calculation engine (FocusScore, StressIndex, FatigueIndex).
//!
//! Phase 3 (P3-E2-T1): DAG scheduler skeleton — register nodes, topological
//! run over an in-memory normalized [`Observation`] snapshot → [`Feature`] /
//! optional [`Signal`]. Catalog formulas land in P3-E2-T2+.
//!
//! # Entrypoint
//!
//! - [`FeatureEngine::register`] — add a [`FeatureNode`]
//! - [`FeatureEngine::run`] — topo compute → [`EngineOutput`]
//!
//! Empty DAG / empty snapshot → [`Ok`] with empty output (idle-friendly).
//! No UI, no SQLite schema, no busy-loop.

#![forbid(unsafe_code)]

mod engine;
mod error;
mod node;

pub use bio_spec::{Feature, Observation, Signal};

pub use engine::{EngineOutput, FeatureEngine};
pub use error::{FeatureEngineError, FeatureEngineResult};
pub use node::{ComputeContext, FeatureNode, NodeId, NodeOutput};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "feature-engine";
