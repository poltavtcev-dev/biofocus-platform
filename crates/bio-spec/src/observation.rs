//! Immutable biometric / context facts (`Observation`).

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{SpecError, SpecResult, UnixTimestamp};

/// Observation primary key. New IDs **should** be UUIDv7 (`docs/02-domain-model.md`).
pub type ObservationId = Uuid;

/// Stable provider / plugin identifier (e.g. `com.biofocus.applehealth`).
pub type ProviderId = String;

/// Observation kind discriminator (e.g. `heart_rate`, `context_window`).
pub type DataType = String;

/// Confidence score in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Confidence(f64);

impl Confidence {
    /// Validates and wraps a confidence value.
    pub fn try_new(value: f64) -> SpecResult<Self> {
        if !(0.0..=1.0).contains(&value) || value.is_nan() {
            return Err(SpecError::InvalidConfidence(value));
        }
        Ok(Self(value))
    }

    /// Clamps a finite value into `[0.0, 1.0]`; non-finite → `0.0`.
    ///
    /// Preferred for derived Feature confidence (ADR-007) where the formula
    /// already saturates and callers must not fail the DAG on float noise.
    #[must_use]
    pub fn saturating_from(value: f64) -> Self {
        if !value.is_finite() {
            return Self(0.0);
        }
        Self(value.clamp(0.0, 1.0))
    }

    /// Full confidence (`1.0`).
    pub const ONE: Self = Self(1.0);

    /// Zero confidence (`0.0`).
    pub const ZERO: Self = Self(0.0);

    /// Returns the inner `f64` in `[0.0, 1.0]`.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// Immutable atomic biometric or context fact.
///
/// After persistence, Observations must not be edited (`docs/02-domain-model.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// UUIDv7 preferred for time-sortable identifiers.
    pub id: ObservationId,
    /// Event time as Unix seconds UTC.
    pub timestamp: UnixTimestamp,
    pub provider_id: ProviderId,
    pub data_type: DataType,
    /// Provider-specific JSON payload (schema varies by `data_type`).
    pub payload: JsonValue,
    pub confidence: Confidence,
}

impl Observation {
    /// Constructs an Observation after validating `confidence`.
    pub fn try_new(
        id: ObservationId,
        timestamp: UnixTimestamp,
        provider_id: impl Into<ProviderId>,
        data_type: impl Into<DataType>,
        payload: JsonValue,
        confidence: f64,
    ) -> SpecResult<Self> {
        Ok(Self {
            id,
            timestamp,
            provider_id: provider_id.into(),
            data_type: data_type.into(),
            payload,
            confidence: Confidence::try_new(confidence)?,
        })
    }
}
