//! Windowed derived metrics (`Feature`) with observation provenance.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{ObservationId, TimeWindow};

/// Stable feature name (e.g. `FocusScore`, `StressIndex`).
pub type FeatureId = String;

/// IDs of Observations that contributed to a Feature calculation.
pub type Provenance = Vec<ObservationId>;

/// Feature output: scalar or structured JSON (`docs/02-domain-model.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureValue {
    Scalar(f64),
    Object(JsonValue),
}

/// Computed characteristic over a time window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub feature_id: FeatureId,
    pub time_window: TimeWindow,
    pub value: FeatureValue,
    /// Observation IDs used as evidence for this metric.
    pub provenance: Provenance,
}
