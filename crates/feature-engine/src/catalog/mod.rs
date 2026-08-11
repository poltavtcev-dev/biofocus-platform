//! Catalog Feature nodes (Phase 3 E2 + Phase 6 E3 + Phase 7 E3 + Phase 10 E3 +
//! Phase 12 E3 + Phase 13 E3 + Phase 16 E2 + Phase 17 E3) — deterministic v1 formulas.
//!
//! | Node | Feature id | Task |
//! | :--- | :--- | :--- |
//! | [`ContextSwitchRateNode`] | `ContextSwitchRate` | P3-E2-T2 |
//! | [`FocusScoreNode`] | `FocusScore` | P3-E2-T2 |
//! | [`StressIndexNode`] | `StressIndex` (+ `High_Stress` Signal) | P3-E2-T3 |
//! | [`FatigueIndexNode`] | `FatigueIndex` | P3-E2-T3 |
//! | [`MeetingDensityNode`] | `MeetingDensity` | P6-E3-T2 |
//! | [`RecoveryBetweenMeetingsNode`] | `RecoveryBetweenMeetings` | P6-E3-T2 |
//! | [`RecoveryScoreNode`] | `RecoveryScore` | P7-E3-T1 |
//! | [`DistractionScoreNode`] | `DistractionScore` | P10-E3-T1 |
//! | [`AmbientMediaShareNode`] | `AmbientMediaShare` | P12-E3-T1 |
//! | [`GitActivityRateNode`] | `GitActivityRate` | P13-E3-T1 |
//! | [`AmbientLightShareNode`] | `AmbientLightShare` | P16-E2-T1 |
//! | [`ActivityBalanceNode`] | `ActivityBalance` | P17-E3-T1 |
//! | [`EnergyScoreNode`] | `EnergyScore` | P17-E3-T1 |
//! | [`SleepDebtNode`] | `SleepDebt` | P17-E3-T1 |
//!
//! Register Focus pair via [`register_focus_v1`]; Stress/Fatigue via
//! [`register_stress_v1`] (requires FocusScore already registered for Fatigue);
//! Calendar pair via [`register_calendar_v1`]; Recovery via [`register_recovery_v1`];
//! Distraction via [`register_distraction_v1`] (requires `ContextSwitchRate`);
//! Ambient media via [`register_ambient_v1`]; Git activity via [`register_git_v1`];
//! Ambient light via [`register_ambient_light_v1`]; Wearable via [`register_wearable_v1`].

mod activity_balance;
mod ambient_light_share;
mod ambient_media_share;
mod calendar_meeting;
mod confidence;
mod context_switch_rate;
mod distraction_score;
mod energy_score;
mod fatigue_index;
mod focus_score;
mod git_activity_rate;
mod hrv;
mod meeting_density;
mod recovery_between_meetings;
mod recovery_score;
mod sleep_debt;
mod stress_index;
mod window;

pub use confidence::{
    compute_feature_confidence, compute_from_values, mean_observation_confidence,
    single_family_confidence,
};

pub use activity_balance::{ActivityBalanceNode, FEATURE_ID as ACTIVITY_BALANCE_ID};
pub use ambient_light_share::{AmbientLightShareNode, FEATURE_ID as AMBIENT_LIGHT_SHARE_ID};
pub use ambient_media_share::{AmbientMediaShareNode, FEATURE_ID as AMBIENT_MEDIA_SHARE_ID};
pub use context_switch_rate::{ContextSwitchRateNode, FEATURE_ID as CONTEXT_SWITCH_RATE_ID};
pub use distraction_score::{DistractionScoreNode, FEATURE_ID as DISTRACTION_SCORE_ID};
pub use energy_score::{EnergyScoreNode, FEATURE_ID as ENERGY_SCORE_ID};
pub use fatigue_index::{FatigueIndexNode, FEATURE_ID as FATIGUE_INDEX_ID};
pub use focus_score::{FocusScoreNode, FEATURE_ID as FOCUS_SCORE_ID};
pub use git_activity_rate::{GitActivityRateNode, FEATURE_ID as GIT_ACTIVITY_RATE_ID};
pub use meeting_density::{MeetingDensityNode, FEATURE_ID as MEETING_DENSITY_ID};
pub use recovery_between_meetings::{
    RecoveryBetweenMeetingsNode, FEATURE_ID as RECOVERY_BETWEEN_MEETINGS_ID,
};
pub use recovery_score::{RecoveryScoreNode, FEATURE_ID as RECOVERY_SCORE_ID};
pub use sleep_debt::{SleepDebtNode, FEATURE_ID as SLEEP_DEBT_ID};
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

/// Registers `RecoveryScore` (independent; HRV + optional heart_rate).
pub fn register_recovery_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(RecoveryScoreNode::new())?;
    Ok(())
}

/// Registers `DistractionScore` (depends on `ContextSwitchRate` for optional CSR).
///
/// Call [`register_focus_v1`] first (or otherwise register `ContextSwitchRate`)
/// before [`FeatureEngine::run`].
pub fn register_distraction_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(DistractionScoreNode::new())?;
    Ok(())
}

/// Registers `AmbientMediaShare` (independent; `now_playing` Observations).
pub fn register_ambient_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(AmbientMediaShareNode::new())?;
    Ok(())
}

/// Registers `GitActivityRate` (independent; `git_activity` Observations).
pub fn register_git_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(GitActivityRateNode::new())?;
    Ok(())
}

/// Registers `AmbientLightShare` (independent; `ambient_light` Observations).
pub fn register_ambient_light_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(AmbientLightShareNode::new())?;
    Ok(())
}

/// Registers wearable catalog Features (ADR-018 / P17-E3): ActivityBalance,
/// EnergyScore, SleepDebt.
pub fn register_wearable_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(ActivityBalanceNode::new())?;
    engine.register(EnergyScoreNode::new())?;
    engine.register(SleepDebtNode::new())?;
    Ok(())
}

/// Registers the full v1 catalog: Focus, Stress/Fatigue, Calendar, Recovery,
/// Distraction, Ambient media, Git activity, Ambient light, Wearable.
pub fn register_catalog_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    register_focus_v1(engine)?;
    register_stress_v1(engine)?;
    register_calendar_v1(engine)?;
    register_recovery_v1(engine)?;
    register_distraction_v1(engine)?;
    register_ambient_v1(engine)?;
    register_git_v1(engine)?;
    register_ambient_light_v1(engine)?;
    register_wearable_v1(engine)?;
    Ok(())
}
