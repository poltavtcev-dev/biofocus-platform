//! Windowed derived metrics (`Feature`) with observation provenance.
//!
//! Feature-level [`Confidence`](crate::Confidence) (ADR-007) is distinct from
//! [`Observation::confidence`](crate::Observation): it scores trust in the
//! derived windowed metric (coverage × mean evidence Observation confidence).
//!
//! Optional [`ExplanationFactor`] breakdowns (P7-E2) describe calm input
//! weights/shares for “why this value” — not LLM prose and not clinical claims.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{Confidence, ObservationId, TimeWindow};

/// Stable feature name (e.g. `FocusScore`, `StressIndex`).
pub type FeatureId = String;

/// IDs of Observations that contributed to a Feature calculation.
pub type Provenance = Vec<ObservationId>;

/// One calm contribution factor for a Feature value (P7-E2).
///
/// Describes an input family’s share of the weighted blend — data composition,
/// not a diagnosis. Catalog nodes that emit factors use renormalized shares
/// that sum to `1.0` (± floating epsilon) over present components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationFactor {
    /// Stable factor id (e.g. `typing`, `stability`, `hrv`).
    pub id: String,
    /// Calm human label (e.g. `"Typing activity"`). Non-clinical.
    pub label: String,
    /// Contribution share after weight renormalization, in `[0.0, 1.0]`.
    pub share: f64,
}

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
    /// Derived trust in this Feature value (`[0.0, 1.0]`, ADR-007).
    ///
    /// Data-quality score — not a clinical claim. Empty / uncomputable windows
    /// omit the Feature rather than emitting `confidence = 0` alone.
    pub confidence: Confidence,
    /// Optional calm “why this value” factor breakdown (P7-E2).
    ///
    /// Empty / omitted when the catalog node does not emit factors yet, or when
    /// the window is empty (Feature itself omitted). Additive optional field —
    /// no SQLite schema; IPC may omit the key when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub factors: Vec<ExplanationFactor>,
}
