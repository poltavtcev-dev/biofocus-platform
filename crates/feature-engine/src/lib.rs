//! Feature calculation engine (FocusScore, StressIndex, FatigueIndex).
//!
//! Phase 3 (P3-E2): DAG scheduler + catalog v1 nodes.
//!
//! # Entrypoint
//!
//! - [`FeatureEngine::register`] — add a [`FeatureNode`]
//! - [`catalog::register_focus_v1`] — `ContextSwitchRate` + `FocusScore` (v1)
//! - [`FeatureEngine::run`] — topo compute → [`EngineOutput`]
//!
//! Empty DAG / empty snapshot → [`Ok`] with empty output (idle-friendly).
//! No UI, no SQLite schema, no busy-loop.

#![forbid(unsafe_code)]

pub mod catalog;

mod engine;
mod error;
mod node;

pub use bio_spec::{Feature, Observation, Signal};

pub use catalog::{
    register_focus_v1, ContextSwitchRateNode, FocusScoreNode, CONTEXT_SWITCH_RATE_ID,
    FOCUS_SCORE_ID, STEP_SECS, WINDOW_SECS,
};
pub use engine::{EngineOutput, FeatureEngine};
pub use error::{FeatureEngineError, FeatureEngineResult};
pub use node::{ComputeContext, FeatureNode, NodeId, NodeOutput};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "feature-engine";
