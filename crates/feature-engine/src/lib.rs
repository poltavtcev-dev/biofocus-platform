//! Feature calculation engine (FocusScore, StressIndex, FatigueIndex, MeetingDensity,
//! RecoveryScore, DistractionScore, AmbientMediaShare, GitActivityRate, AmbientLightShare,
//! ActivityBalance, EnergyScore, SleepDebt).
//!
//! Phase 3 (P3-E2 / E3): DAG scheduler + catalog v1 nodes + alert mapping.
//! Phase 4 (P4-E1-T1): [`FeatureSnapshot`] for dashboard / IPC (cached read).
//! Phase 6 (P6-E3-T2): Calendar Features from `calendar_event` Observations.
//! Phase 7 (P7-E3-T1): `RecoveryScore` from HRV + optional heart_rate.
//! Phase 8 (P8-E2-T1): [`baseline`] recompute-on-read FocusScore afternoon series (ADR-008).
//! Phase 10 (P10-E3-T1): `DistractionScore` from `browser_category` (+ optional CSR).
//! Phase 12 (P12-E3-T1): `AmbientMediaShare` from `now_playing` Observations.
//! Phase 13 (P13-E3-T1): `GitActivityRate` from `git_activity` Observations.
//! Phase 16 (P16-E2-T1): `AmbientLightShare` from `ambient_light` Observations.
//! Phase 17 (P17-E3-T1): Wearable Features + `run_with_step` for chart ranges (ADR-018).
//!
//! # Entrypoint
//!
//! - [`FeatureEngine::register`] — add a [`FeatureNode`]
//! - [`catalog::register_focus_v1`] — `ContextSwitchRate` + `FocusScore` (v1)
//! - [`catalog::register_stress_v1`] — `StressIndex` + `FatigueIndex` (v1; needs Focus)
//! - [`catalog::register_calendar_v1`] — `MeetingDensity` + `RecoveryBetweenMeetings` (v1)
//! - [`catalog::register_recovery_v1`] — `RecoveryScore` (v1; HRV + optional HR)
//! - [`catalog::register_distraction_v1`] — `DistractionScore` (v1; needs CSR)
//! - [`catalog::register_ambient_v1`] — `AmbientMediaShare` (v1; `now_playing`)
//! - [`catalog::register_git_v1`] — `GitActivityRate` (v1; `git_activity`)
//! - [`catalog::register_ambient_light_v1`] — `AmbientLightShare` (v1; `ambient_light`)
//! - [`catalog::register_wearable_v1`] — `ActivityBalance` + `EnergyScore` + `SleepDebt` (v1)
//! - [`catalog::register_catalog_v1`] — Focus + Stress/Fatigue + Calendar + Recovery + Distraction + Ambient media + Git + Ambient light + Wearable
//! - [`baseline::recompute_focus_afternoon_baseline`] — bounded prior-day Focus means
//! - [`FeatureEngine::run`] / [`FeatureEngine::run_with_step`] — topo compute → [`EngineOutput`]
//! - [`FeatureSnapshot::from_engine_output`] — Features + Signals for IPC/dashboard
//! - [`map_alert_level`] — [`EngineOutput`] → [`AlertLevel`] { Green, Yellow, Red }
//!
//! Empty DAG / empty snapshot → [`Ok`] with empty output (idle-friendly).
//! Empty / calm output → [`AlertLevel::Green`]. No UI, no SQLite schema, no busy-loop.

#![forbid(unsafe_code)]

pub mod alert;
pub mod baseline;
pub mod catalog;
pub mod snapshot;

mod engine;
mod error;
mod node;

pub use alert::{map_alert_level, AlertLevel, YELLOW_FEATURE_THRESHOLD};
pub use baseline::{
    baseline_lookback_start, recompute_focus_afternoon_baseline, utc_day_start,
    AFTERNOON_END_HOUR_UTC, AFTERNOON_START_HOUR_UTC, BASELINE_CONFIDENCE_GATE,
    BASELINE_MAX_WINDOWS, BASELINE_MIN_WINDOWS,
};
pub use bio_spec::{Feature, FeatureValue, Observation, Signal};

pub use catalog::{
    register_ambient_light_v1, register_ambient_v1, register_calendar_v1, register_catalog_v1,
    register_distraction_v1, register_focus_v1, register_git_v1, register_recovery_v1,
    register_stress_v1, register_wearable_v1, ActivityBalanceNode, AmbientLightShareNode,
    AmbientMediaShareNode, ContextSwitchRateNode, DistractionScoreNode, EnergyScoreNode,
    FatigueIndexNode, FocusScoreNode, GitActivityRateNode, MeetingDensityNode,
    RecoveryBetweenMeetingsNode, RecoveryScoreNode, SleepDebtNode, StressIndexNode,
    ACTIVITY_BALANCE_ID, AMBIENT_LIGHT_SHARE_ID, AMBIENT_MEDIA_SHARE_ID, CONTEXT_SWITCH_RATE_ID,
    DISTRACTION_SCORE_ID, ENERGY_SCORE_ID, FATIGUE_INDEX_ID, FOCUS_SCORE_ID, GIT_ACTIVITY_RATE_ID,
    HIGH_STRESS_MIN_DURATION_SECS, HIGH_STRESS_SIGNAL_TYPE, HIGH_STRESS_THRESHOLD,
    MEETING_DENSITY_ID, RECOVERY_BETWEEN_MEETINGS_ID, RECOVERY_SCORE_ID, SLEEP_DEBT_ID, STEP_SECS,
    STRESS_INDEX_ID, WINDOW_SECS,
};
pub use engine::{EngineOutput, FeatureEngine};
pub use error::{FeatureEngineError, FeatureEngineResult};
pub use node::{ComputeContext, FeatureNode, NodeId, NodeOutput};
pub use snapshot::FeatureSnapshot;

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "feature-engine";
