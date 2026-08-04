//! Transient change / anomaly events (`Signal`).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::UnixTimestamp;

/// Signal primary key.
pub type SignalId = Uuid;

/// Signal kind (e.g. `HR_Spike`, `Context_Switch`, `Inactivity_Period`).
pub type SignalType = String;

/// Relative severity of a signal for alert routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Detected state change or anomaly in time.
///
/// Lifecycle is **transient**: typically computed in memory for alerts
/// (`docs/02-domain-model.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    pub id: SignalId,
    #[serde(rename = "type")]
    pub signal_type: SignalType,
    pub timestamp_start: UnixTimestamp,
    pub timestamp_end: UnixTimestamp,
    pub severity: Severity,
}
