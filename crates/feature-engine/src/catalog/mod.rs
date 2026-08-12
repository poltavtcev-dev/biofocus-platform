//! Catalog Feature nodes (Phase 3 E2 + Phase 6 E3 + Phase 7 E3 + Phase 10 E3 +
//! Phase 12 E3 + Phase 13 E3 + Phase 16 E2 + Phase 17 E3 + Phase 18 E3 +
//! Phase 20 E2 + Phase 21 E2 + Phase 22 E2 + Phase 23 E2 + Phase 24 E2) — deterministic v1 formulas.
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
//! | [`NotificationPressureNode`] | `NotificationPressure` | P18-E3-T1 |
//! | [`CognitiveLoadNode`] | `CognitiveLoad` | P20-E2-T1 |
//! | [`DeepWorkScoreNode`] | `DeepWorkScore` | P21-E2-T1 |
//! | [`AttentionStabilityNode`] | `AttentionStability` | P22-E2-T1 |
//! | [`DeskAwayPresenceNode`] | `DeskAwayPresence` | P23-E2-T1 |
//! | [`CircadianOffsetNode`] | `CircadianOffset` | P24-E2-T1 |
//!
//! Register Focus pair via [`register_focus_v1`]; Stress/Fatigue via
//! [`register_stress_v1`] (requires FocusScore already registered for Fatigue);
//! Calendar pair via [`register_calendar_v1`]; Recovery via [`register_recovery_v1`];
//! Distraction via [`register_distraction_v1`] (requires `ContextSwitchRate`);
//! Ambient media via [`register_ambient_v1`]; Git activity via [`register_git_v1`];
//! Ambient light via [`register_ambient_light_v1`]; Wearable via [`register_wearable_v1`];
//! Notification via [`register_notification_v1`]; CognitiveLoad via
//! [`register_cognitive_v1`] (requires MeetingDensity + CSR + NotificationPressure);
//! DeepWorkScore via [`register_deep_work_v1`] (requires FocusScore + CSR);
//! AttentionStability via [`register_attention_stability_v1`] (requires FocusScore + CSR);
//! DeskAwayPresence via [`register_desk_away_v1`] (Observation-level; independent);
//! CircadianOffset via [`register_circadian_v1`] (Observation-level timing; independent).

mod activity_balance;
mod ambient_light_share;
mod ambient_media_share;
mod attention_stability;
mod calendar_meeting;
mod circadian_offset;
mod cognitive_load;
mod confidence;
mod context_switch_rate;
mod deep_work_score;
mod desk_away_presence;
mod distraction_score;
mod energy_score;
mod fatigue_index;
mod focus_score;
mod git_activity_rate;
mod hrv;
mod meeting_density;
mod notification_pressure;
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
pub use attention_stability::{AttentionStabilityNode, FEATURE_ID as ATTENTION_STABILITY_ID};
pub use circadian_offset::{CircadianOffsetNode, FEATURE_ID as CIRCADIAN_OFFSET_ID};
pub use cognitive_load::{CognitiveLoadNode, FEATURE_ID as COGNITIVE_LOAD_ID};
pub use context_switch_rate::{ContextSwitchRateNode, FEATURE_ID as CONTEXT_SWITCH_RATE_ID};
pub use deep_work_score::{DeepWorkScoreNode, FEATURE_ID as DEEP_WORK_SCORE_ID};
pub use desk_away_presence::{DeskAwayPresenceNode, FEATURE_ID as DESK_AWAY_PRESENCE_ID, MIN_STEPS_AWAY};
pub use distraction_score::{DistractionScoreNode, FEATURE_ID as DISTRACTION_SCORE_ID};
pub use energy_score::{EnergyScoreNode, FEATURE_ID as ENERGY_SCORE_ID};
pub use fatigue_index::{FatigueIndexNode, FEATURE_ID as FATIGUE_INDEX_ID};
pub use focus_score::{FocusScoreNode, FEATURE_ID as FOCUS_SCORE_ID};
pub use git_activity_rate::{GitActivityRateNode, FEATURE_ID as GIT_ACTIVITY_RATE_ID};
pub use meeting_density::{MeetingDensityNode, FEATURE_ID as MEETING_DENSITY_ID};
pub use notification_pressure::{
    NotificationPressureNode, FEATURE_ID as NOTIFICATION_PRESSURE_ID, SATURATION_COUNT,
};
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

/// Registers `NotificationPressure` (independent; `notification_event` Observations).
pub fn register_notification_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(NotificationPressureNode::new())?;
    Ok(())
}

/// Registers `CognitiveLoad` (Feature-level composite; ADR-021).
///
/// Depends on `MeetingDensity`, `ContextSwitchRate`, and `NotificationPressure` —
/// call [`register_focus_v1`], [`register_calendar_v1`], and
/// [`register_notification_v1`] first (or otherwise register those nodes) before
/// [`FeatureEngine::run`].
pub fn register_cognitive_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(CognitiveLoadNode::new())?;
    Ok(())
}

/// Registers `DeepWorkScore` (Feature-level composite; ADR-022).
///
/// Depends on `FocusScore` and `ContextSwitchRate` — call [`register_focus_v1`]
/// first (or otherwise register those nodes) before [`FeatureEngine::run`].
pub fn register_deep_work_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(DeepWorkScoreNode::new())?;
    Ok(())
}

/// Registers `AttentionStability` (Feature-level composite; ADR-023).
///
/// Depends on `FocusScore` and `ContextSwitchRate` — call [`register_focus_v1`]
/// first (or otherwise register those nodes) before [`FeatureEngine::run`].
pub fn register_attention_stability_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(AttentionStabilityNode::new())?;
    Ok(())
}

/// Registers `DeskAwayPresence` (Observation-level; ADR-024 / P23-E2).
///
/// Independent of other Feature nodes — uses keystrokes / context_window /
/// step_count / life_event walk only. No GPS.
pub fn register_desk_away_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(DeskAwayPresenceNode::new())?;
    Ok(())
}

/// Registers `CircadianOffset` (Observation-level timing; ADR-025 / P24-E2).
///
/// Independent of other Feature nodes — sleep_interval + keystrokes /
/// context_window (optional steps / active_energy / workout). No Feature-level
/// magnitude proxies.
pub fn register_circadian_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(CircadianOffsetNode::new())?;
    Ok(())
}

/// Registers the full v1 catalog: Focus, Stress/Fatigue, Calendar, Recovery,
/// Distraction, Ambient media, Git activity, Ambient light, Wearable,
/// Notification pressure, CognitiveLoad, DeepWorkScore, AttentionStability,
/// DeskAwayPresence, CircadianOffset.
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
    register_notification_v1(engine)?;
    register_cognitive_v1(engine)?;
    register_deep_work_v1(engine)?;
    register_attention_stability_v1(engine)?;
    register_desk_away_v1(engine)?;
    register_circadian_v1(engine)?;
    Ok(())
}
