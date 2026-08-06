//! Catalog Feature nodes (Phase 3 E2 + Phase 6 E3) — deterministic v1 formulas.
//!
//! | Node | Feature id | Task |
//! | :--- | :--- | :--- |
//! | [`ContextSwitchRateNode`] | `ContextSwitchRate` | P3-E2-T2 |
//! | [`FocusScoreNode`] | `FocusScore` | P3-E2-T2 |
//! | [`StressIndexNode`] | `StressIndex` (+ `High_Stress` Signal) | P3-E2-T3 |
//! | [`FatigueIndexNode`] | `FatigueIndex` | P3-E2-T3 |
//! | [`MeetingDensityNode`] | `MeetingDensity` | P6-E3-T2 |
//! | [`RecoveryBetweenMeetingsNode`] | `RecoveryBetweenMeetings` | P6-E3-T2 |
//!
//! Register Focus pair via [`register_focus_v1`]; Stress/Fatigue via
//! [`register_stress_v1`] (requires FocusScore already registered for Fatigue);
//! Calendar pair via [`register_calendar_v1`].

mod calendar_meeting;
mod context_switch_rate;
mod fatigue_index;
mod focus_score;
mod meeting_density;
mod recovery_between_meetings;
mod stress_index;
mod window;

pub use context_switch_rate::{ContextSwitchRateNode, FEATURE_ID as CONTEXT_SWITCH_RATE_ID};
pub use fatigue_index::{FatigueIndexNode, FEATURE_ID as FATIGUE_INDEX_ID};
pub use focus_score::{FocusScoreNode, FEATURE_ID as FOCUS_SCORE_ID};
pub use meeting_density::{MeetingDensityNode, FEATURE_ID as MEETING_DENSITY_ID};
pub use recovery_between_meetings::{
    RecoveryBetweenMeetingsNode, FEATURE_ID as RECOVERY_BETWEEN_MEETINGS_ID,
};
pub use stress_index::{
    StressIndexNode, FEATURE_ID as STRESS_INDEX_ID, HIGH_STRESS_MIN_DURATION_SECS,
    HIGH_STRESS_SIGNAL_TYPE, HIGH_STRESS_THRESHOLD,
};
pub use window::{STEP_SECS, WINDOW_SECS};

use crate::{FeatureEngine, FeatureEngineResult};

/// Registers `ContextSwitchRate` then `FocusScore` (DAG dependency order).
pub fn register_focus_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(ContextSwitchRateNode::new())?;
    engine.register(FocusScoreNode::new())?;
    Ok(())
}

/// Registers `StressIndex` then `FatigueIndex`.
///
/// `FatigueIndex` depends on `FocusScore` — call [`register_focus_v1`] first
/// (or otherwise register `FocusScore`) before [`FeatureEngine::run`].
pub fn register_stress_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(StressIndexNode::new())?;
    engine.register(FatigueIndexNode::new())?;
    Ok(())
}

/// Registers `MeetingDensity` then `RecoveryBetweenMeetings` (independent nodes).
pub fn register_calendar_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(MeetingDensityNode::new())?;
    engine.register(RecoveryBetweenMeetingsNode::new())?;
    Ok(())
}

/// Registers the full v1 catalog: Focus pair, Stress/Fatigue, then Calendar pair.
pub fn register_catalog_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    register_focus_v1(engine)?;
    register_stress_v1(engine)?;
    register_calendar_v1(engine)?;
    Ok(())
}
