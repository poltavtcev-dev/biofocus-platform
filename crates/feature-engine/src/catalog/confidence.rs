//! Feature-level confidence helpers (ADR-007).
//!
//! ```text
//! coverage = present_input_slots / expected_input_slots
//! mean_obs = mean(Observation.confidence of evidence)
//! Feature.confidence = clamp(coverage × mean_obs, 0.0, 1.0)
//! ```
//!
//! Empty / uncomputable windows omit the Feature (callers never invoke these
//! helpers for idle steps). Confidence is data quality, not a clinical claim.

use bio_spec::{Confidence, Observation};

/// v1 Feature confidence from input-slot coverage and evidence Observations.
///
/// - `expected_input_slots` — catalog-declared input families for this Feature
/// - `present_input_slots` — how many of those families contributed this window
/// - `evidence` — Observations used as provenance (may be empty → `0.0`)
#[must_use]
pub fn compute_feature_confidence(
    expected_input_slots: usize,
    present_input_slots: usize,
    evidence: &[&Observation],
) -> Confidence {
    let coverage = if expected_input_slots == 0 {
        0.0
    } else {
        (present_input_slots.min(expected_input_slots) as f64) / expected_input_slots as f64
    };
    let mean_obs = mean_observation_confidence(evidence);
    Confidence::saturating_from(coverage * mean_obs)
}

/// Mean of `Observation.confidence` over evidence; empty → `0.0`.
#[must_use]
pub fn mean_observation_confidence(evidence: &[&Observation]) -> f64 {
    if evidence.is_empty() {
        return 0.0;
    }
    evidence.iter().map(|o| o.confidence.get()).sum::<f64>() / evidence.len() as f64
}

/// Single-family Features (e.g. StressIndex / CSR / calendar): when emitted,
/// coverage = 1.0 and confidence = mean evidence Observation confidence.
#[must_use]
pub fn single_family_confidence(evidence: &[&Observation]) -> Confidence {
    let present = usize::from(!evidence.is_empty());
    compute_feature_confidence(1, present, evidence)
}

/// Coverage × mean of raw confidence values (when Observations are not at hand).
#[must_use]
pub fn compute_from_values(
    expected_input_slots: usize,
    present_input_slots: usize,
    confidence_values: &[f64],
) -> Confidence {
    let coverage = if expected_input_slots == 0 {
        0.0
    } else {
        (present_input_slots.min(expected_input_slots) as f64) / expected_input_slots as f64
    };
    let mean_obs = if confidence_values.is_empty() {
        0.0
    } else {
        confidence_values.iter().sum::<f64>() / confidence_values.len() as f64
    };
    Confidence::saturating_from(coverage * mean_obs)
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn obs(id: u128, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(1),
            "test",
            "hrv",
            json!({ "rmssd_ms": 40.0 }),
            confidence,
        )
        .expect("obs")
    }

    #[test]
    fn full_coverage_high_obs_confidence() {
        let a = obs(1, 1.0);
        let b = obs(2, 1.0);
        let evidence = [&a, &b];
        let c = compute_feature_confidence(3, 3, &evidence);
        assert!((c.get() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn partial_coverage_lowers_confidence() {
        let a = obs(1, 1.0);
        let evidence = [&a];
        let rich = compute_feature_confidence(3, 3, &evidence);
        let thin = compute_feature_confidence(3, 1, &evidence);
        assert!(thin.get() < rich.get());
        assert!((thin.get() - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let a = obs(1, 0.4);
        let evidence = [&a];
        let c = single_family_confidence(&evidence);
        assert!((c.get() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn empty_evidence_is_zero() {
        assert_eq!(single_family_confidence(&[]).get(), 0.0);
    }
}
