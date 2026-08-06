//! Deterministic Features/Signals → Menubar alert level (P3-E3-T1).
//!
//! # v1 rules
//!
//! Evaluated in order (first match wins):
//!
//! 1. **Red** — any Signal with `signal_type == "High_Stress"` in the
//!    [`EngineOutput`](crate::EngineOutput).
//! 2. **Yellow** — latest (by `time_window.end`) scalar `StressIndex` **or**
//!    `FatigueIndex` is **>** [`YELLOW_FEATURE_THRESHOLD`] (60.0).
//! 3. **Green** — otherwise, including empty output (idle-safe: no alarm
//!    without evidence).
//!
//! Pure / synchronous. No I/O, no UI, no SQLite.

use bio_spec::{Feature, FeatureValue, Signal};

use crate::catalog::{
    FATIGUE_INDEX_ID, HIGH_STRESS_SIGNAL_TYPE, STRESS_INDEX_ID,
};
use crate::EngineOutput;

/// Elevated Feature threshold for Yellow (below catalog High_Stress contiguous
/// trigger of 75, but already notable for Menubar caution).
pub const YELLOW_FEATURE_THRESHOLD: f64 = 60.0;

/// Traffic-light alert level for Menubar (🟢 / 🟡 / 🔴).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertLevel {
    /// Calm / idle / insufficient evidence.
    Green,
    /// Elevated stress or fatigue without a High_Stress Signal.
    Yellow,
    /// High_Stress Signal present.
    Red,
}

impl AlertLevel {
    /// Stable lowercase label for status / IPC payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
        }
    }
}

/// Map an [`EngineOutput`] to an [`AlertLevel`] using the v1 rules above.
#[must_use]
pub fn map_alert_level(output: &EngineOutput) -> AlertLevel {
    if has_high_stress_signal(&output.signals) {
        return AlertLevel::Red;
    }
    if latest_scalar(&output.features, STRESS_INDEX_ID).is_some_and(|v| v > YELLOW_FEATURE_THRESHOLD)
        || latest_scalar(&output.features, FATIGUE_INDEX_ID)
            .is_some_and(|v| v > YELLOW_FEATURE_THRESHOLD)
    {
        return AlertLevel::Yellow;
    }
    AlertLevel::Green
}

fn has_high_stress_signal(signals: &[Signal]) -> bool {
    signals
        .iter()
        .any(|s| s.signal_type == HIGH_STRESS_SIGNAL_TYPE)
}

/// Latest Feature by window end for a given feature id; returns scalar value.
fn latest_scalar(features: &[Feature], feature_id: &str) -> Option<f64> {
    features
        .iter()
        .filter(|f| f.feature_id == feature_id)
        .max_by_key(|f| f.time_window.end.as_secs())
        .and_then(|f| match f.value {
            FeatureValue::Scalar(v) if v.is_finite() => Some(v),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use bio_spec::{Confidence, Feature, FeatureValue, Severity, Signal, TimeWindow, UnixTimestamp};
    use uuid::Uuid;

    use super::*;
    use crate::EngineOutput;

    fn window(end: i64) -> TimeWindow {
        TimeWindow::try_new(
            UnixTimestamp::from_secs(end - 900),
            UnixTimestamp::from_secs(end),
        )
        .expect("window")
    }

    fn scalar_feature(feature_id: &str, end: i64, value: f64) -> Feature {
        Feature {
            feature_id: feature_id.to_owned(),
            time_window: window(end),
            value: FeatureValue::Scalar(value),
            provenance: Vec::new(),
            confidence: Confidence::ONE,
        }
    }

    fn high_stress_signal() -> Signal {
        Signal {
            id: Uuid::from_u128(1),
            signal_type: HIGH_STRESS_SIGNAL_TYPE.to_owned(),
            timestamp_start: UnixTimestamp::from_secs(900),
            timestamp_end: UnixTimestamp::from_secs(1260),
            severity: Severity::High,
        }
    }

    #[test]
    fn empty_output_is_green() {
        assert_eq!(map_alert_level(&EngineOutput::default()), AlertLevel::Green);
    }

    #[test]
    fn calm_features_are_green() {
        let out = EngineOutput {
            features: vec![
                scalar_feature(STRESS_INDEX_ID, 1800, 40.0),
                scalar_feature(FATIGUE_INDEX_ID, 1800, 55.0),
            ],
            signals: Vec::new(),
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Green);
    }

    #[test]
    fn elevated_stress_without_signal_is_yellow() {
        let out = EngineOutput {
            features: vec![scalar_feature(STRESS_INDEX_ID, 1800, 61.0)],
            signals: Vec::new(),
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Yellow);
    }

    #[test]
    fn elevated_fatigue_without_signal_is_yellow() {
        let out = EngineOutput {
            features: vec![scalar_feature(FATIGUE_INDEX_ID, 1800, 70.0)],
            signals: Vec::new(),
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Yellow);
    }

    #[test]
    fn stress_at_threshold_is_still_green() {
        // Strict `>` — 60.0 alone is Green.
        let out = EngineOutput {
            features: vec![scalar_feature(STRESS_INDEX_ID, 1800, YELLOW_FEATURE_THRESHOLD)],
            signals: Vec::new(),
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Green);
    }

    #[test]
    fn high_stress_signal_is_red() {
        let out = EngineOutput {
            features: vec![scalar_feature(STRESS_INDEX_ID, 1800, 40.0)],
            signals: vec![high_stress_signal()],
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Red);
    }

    #[test]
    fn high_stress_overrides_yellow_features() {
        let out = EngineOutput {
            features: vec![
                scalar_feature(STRESS_INDEX_ID, 1800, 90.0),
                scalar_feature(FATIGUE_INDEX_ID, 1800, 90.0),
            ],
            signals: vec![high_stress_signal()],
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Red);
    }

    #[test]
    fn latest_window_wins_for_yellow() {
        let out = EngineOutput {
            features: vec![
                scalar_feature(STRESS_INDEX_ID, 1200, 90.0),
                scalar_feature(STRESS_INDEX_ID, 1800, 10.0),
            ],
            signals: Vec::new(),
        };
        assert_eq!(map_alert_level(&out), AlertLevel::Green);
    }

    #[test]
    fn as_str_labels() {
        assert_eq!(AlertLevel::Green.as_str(), "green");
        assert_eq!(AlertLevel::Yellow.as_str(), "yellow");
        assert_eq!(AlertLevel::Red.as_str(), "red");
    }
}
