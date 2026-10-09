//! Heart-rate variability compared with the person's own recent days.
//!
//! Fires only when `HrvVsBaseline` is already present (five earlier days of one
//! method). Copy stays observational. SDNN and RMSSD are not named as a diagnosis.

use bio_spec::{EvidenceRef, Feature, FeatureValue, Insight, Signal};
use uuid::Uuid;

use crate::rules::latest_feature;
use crate::{InsightRule, KnowledgeEngineResult};

/// Feature id produced by `feature_engine` wearable vitals.
pub const HRV_VS_BASELINE_ID: &str = "HrvVsBaseline";

/// Stable rule id for [`HrvVsBaselineRule`].
pub const RULE_HRV_VS_BASELINE: &str = "hrv_vs_baseline_v1";

/// Milliseconds below the personal median before an Insight is emitted.
pub const HRV_BASELINE_DELTA_MS: f64 = -5.0;

/// Emits one calm Insight when variability sits below the personal median.
#[derive(Debug, Default, Clone, Copy)]
pub struct HrvVsBaselineRule;

impl InsightRule for HrvVsBaselineRule {
    fn id(&self) -> &str {
        RULE_HRV_VS_BASELINE
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        _pattern: &crate::PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let Some(feature) = latest_feature(features, HRV_VS_BASELINE_ID) else {
            return Ok(Vec::new());
        };
        let Some(delta) = object_f64(feature, "delta_ms") else {
            return Ok(Vec::new());
        };
        if delta > HRV_BASELINE_DELTA_MS {
            return Ok(Vec::new());
        }

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Вариабельность на фоне обычных дней".into(),
            description: "Вариабельность пульса ниже середины ваших недавних дней."
                .into(),
            category: "pattern".into(),
            evidence_list: vec![EvidenceRef::Feature(HRV_VS_BASELINE_ID.into())],
            action_recommendation: Some(
                "Более тихий вечер — один из вариантов, если это всё ещё уместно.".into(),
            ),
        }])
    }
}

fn object_f64(feature: &Feature, key: &str) -> Option<f64> {
    match &feature.value {
        FeatureValue::Object(value) => value
            .get(key)
            .and_then(|v| v.as_f64())
            .filter(|n| n.is_finite()),
        FeatureValue::Scalar(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{Confidence, Feature, FeatureValue, TimeWindow, UnixTimestamp};
    use serde_json::json;

    use super::*;
    use crate::KnowledgeEngine;

    fn hrv_feature(delta_ms: f64) -> Feature {
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(1_000),
            UnixTimestamp::from_secs(2_000),
        )
        .expect("window");
        Feature {
            feature_id: HRV_VS_BASELINE_ID.to_owned(),
            time_window: window,
            value: FeatureValue::Object(json!({
                "method": "sdnn",
                "ms": 40.0,
                "baseline_ms": 40.0 - delta_ms,
                "delta_ms": delta_ms,
                "baseline_days": 6
            })),
            provenance: Vec::new(),
            confidence: Confidence::saturating_from(1.0),
            factors: Vec::new(),
        }
    }

    #[test]
    fn lower_than_usual_emits_and_small_gap_does_not() {
        let mut engine = KnowledgeEngine::new();
        engine.register(HrvVsBaselineRule).expect("reg");
        let quiet = engine
            .evaluate(std::slice::from_ref(&hrv_feature(-1.0)), &[])
            .expect("eval");
        assert!(quiet.is_empty());
        let lower = engine
            .evaluate(std::slice::from_ref(&hrv_feature(-8.0)), &[])
            .expect("eval");
        assert_eq!(lower.len(), 1);
        let blob = format!(
            "{} {} {}",
            lower[0].title,
            lower[0].description,
            lower[0].action_recommendation.as_deref().unwrap_or("")
        )
        .to_lowercase();
        for word in [
            "apnea",
            "arrhythmia",
            "disease",
            "illness",
            "burnout",
            "stress",
            "diagnos",
        ] {
            assert!(!blob.contains(word), "{word} in {blob}");
        }
    }

    #[test]
    fn missing_hrv_emits_nothing() {
        let mut engine = KnowledgeEngine::new();
        engine.register(HrvVsBaselineRule).expect("reg");
        let out = engine.evaluate(&[], &[]).expect("eval");
        assert!(out.is_empty());
    }
}
