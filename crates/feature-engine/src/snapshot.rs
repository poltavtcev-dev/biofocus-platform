//! Public Feature snapshot for dashboard / IPC consumers (P4-E1-T1).
//!
//! Built from the latest [`EngineOutput`](crate::EngineOutput) — Features +
//! optional Signals with provenance Observation ids. No raw Observation
//! payloads, no filesystem paths, no SQLite.

use bio_spec::{Feature, Signal};

use crate::EngineOutput;

/// Recent windowed Features (+ optional Signals) from the Feature Engine.
///
/// Idle / no evidence → empty vectors ([`Self::is_empty`]). Pure data; callers
/// cache and read without recomputing or busy-polling.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FeatureSnapshot {
    /// Derived Features in engine execution order (may include multiple windows).
    pub features: Vec<Feature>,
    /// Optional Signals from the same run (e.g. `High_Stress`).
    pub signals: Vec<Signal>,
}

impl FeatureSnapshot {
    /// Empty snapshot (idle / no evidence).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// `true` when no Features and no Signals.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.features.is_empty() && self.signals.is_empty()
    }

    /// Copy Features / Signals from an engine run (drops Observation payloads).
    #[must_use]
    pub fn from_engine_output(output: &EngineOutput) -> Self {
        Self {
            features: output.features.clone(),
            signals: output.signals.clone(),
        }
    }

    /// Consume an [`EngineOutput`] into a snapshot.
    #[must_use]
    pub fn from_engine_output_owned(output: EngineOutput) -> Self {
        Self {
            features: output.features,
            signals: output.signals,
        }
    }
}

impl From<EngineOutput> for FeatureSnapshot {
    fn from(output: EngineOutput) -> Self {
        Self::from_engine_output_owned(output)
    }
}

impl From<&EngineOutput> for FeatureSnapshot {
    fn from(output: &EngineOutput) -> Self {
        Self::from_engine_output(output)
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{Feature, FeatureValue, Severity, Signal, TimeWindow, UnixTimestamp};
    use uuid::Uuid;

    use super::*;

    fn sample_feature() -> Feature {
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("window");
        Feature {
            feature_id: "FocusScore".into(),
            time_window: window,
            value: FeatureValue::Scalar(72.5),
            provenance: vec![Uuid::from_u128(1)],
        }
    }

    fn sample_signal() -> Signal {
        Signal {
            id: Uuid::from_u128(9),
            signal_type: "High_Stress".into(),
            timestamp_start: UnixTimestamp::from_secs(900),
            timestamp_end: UnixTimestamp::from_secs(1260),
            severity: Severity::High,
        }
    }

    #[test]
    fn empty_snapshot_is_idle() {
        let snap = FeatureSnapshot::empty();
        assert!(snap.is_empty());
        assert!(snap.features.is_empty());
        assert!(snap.signals.is_empty());
    }

    #[test]
    fn from_engine_output_copies_features_and_signals() {
        let output = EngineOutput {
            features: vec![sample_feature()],
            signals: vec![sample_signal()],
        };
        let snap = FeatureSnapshot::from_engine_output(&output);
        assert!(!snap.is_empty());
        assert_eq!(snap.features.len(), 1);
        assert_eq!(snap.features[0].feature_id, "FocusScore");
        assert_eq!(snap.features[0].provenance.len(), 1);
        assert_eq!(snap.signals.len(), 1);
        assert_eq!(snap.signals[0].signal_type, "High_Stress");
        // Domain Feature has no Observation payload fields.
        let json = serde_json::to_value(&snap.features[0]).expect("serialize");
        let obj = json.as_object().expect("object");
        assert!(obj.contains_key("feature_id"));
        assert!(obj.contains_key("provenance"));
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("hrv"));
    }

    #[test]
    fn from_empty_engine_output_stays_empty() {
        let snap = FeatureSnapshot::from(EngineOutput::default());
        assert!(snap.is_empty());
    }
}
