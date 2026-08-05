//! Feature calculation engine (FocusScore, StressIndex, FatigueIndex).
//!
//! Phase 3 (P3-E2 / E3): DAG scheduler + catalog v1 nodes + alert mapping.
//! Phase 4 (P4-E1-T1): [`FeatureSnapshot`] for dashboard / IPC (cached read).
//!
//! # Entrypoint
//!
//! - [`FeatureEngine::register`] — add a [`FeatureNode`]
//! - [`catalog::register_focus_v1`] — `ContextSwitchRate` + `FocusScore` (v1)
//! - [`catalog::register_stress_v1`] — `StressIndex` + `FatigueIndex` (v1; needs Focus)
//! - [`catalog::register_catalog_v1`] — Focus + Stress/Fatigue together
//! - [`FeatureEngine::run`] — topo compute → [`EngineOutput`]
//! - [`FeatureSnapshot::from_engine_output`] — Features + Signals for IPC/dashboard
//! - [`map_alert_level`] — [`EngineOutput`] → [`AlertLevel`] { Green, Yellow, Red }
//!
//! Empty DAG / empty snapshot → [`Ok`] with empty output (idle-friendly).
//! Empty / calm output → [`AlertLevel::Green`]. No UI, no SQLite schema, no busy-loop.

#![forbid(unsafe_code)]

pub mod alert;
pub mod catalog;
pub mod snapshot;

mod engine;
mod error;
mod node;

pub use alert::{map_alert_level, AlertLevel, YELLOW_FEATURE_THRESHOLD};
pub use bio_spec::{Feature, FeatureValue, Observation, Signal};

pub use catalog::{
    register_catalog_v1, register_focus_v1, register_stress_v1, ContextSwitchRateNode,
    FatigueIndexNode, FocusScoreNode, StressIndexNode, CONTEXT_SWITCH_RATE_ID, FATIGUE_INDEX_ID,
    FOCUS_SCORE_ID, HIGH_STRESS_MIN_DURATION_SECS, HIGH_STRESS_SIGNAL_TYPE, HIGH_STRESS_THRESHOLD,
    STEP_SECS, STRESS_INDEX_ID, WINDOW_SECS,
};
pub use engine::{EngineOutput, FeatureEngine};
pub use error::{FeatureEngineError, FeatureEngineResult};
pub use node::{ComputeContext, FeatureNode, NodeId, NodeOutput};
pub use snapshot::FeatureSnapshot;

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "feature-engine";
