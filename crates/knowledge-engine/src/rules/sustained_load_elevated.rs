//! Rule: elevated `SustainedLoadIndicator` → calm prolonged-load Insight (ADR-028).

use bio_spec::{EvidenceRef, Feature, Insight, Signal};
use uuid::Uuid;

use super::cognitive_load_elevated::MEETING_DENSITY_ID;
use super::high_stress::STRESS_INDEX_ID;
use crate::rules::{latest_feature, scalar_value};
use crate::{InsightRule, KnowledgeEngineResult};

/// Catalog Feature id `SustainedLoadIndicator` (required).
pub const SUSTAINED_LOAD_INDICATOR_ID: &str = "SustainedLoadIndicator";

/// Optional Evidence Feature id when present in the snapshot.
pub const FATIGUE_INDEX_ID: &str = "FatigueIndex";

/// Latest scalar SustainedLoadIndicator at/above which the rule fires (0–100; ADR-028 ±10 OK).
pub const SUSTAINED_LOAD_ELEVATED_THRESHOLD: f64 = 60.0;

/// Stable rule id for [`SustainedLoadElevatedRule`].
pub const RULE_SUSTAINED_LOAD_ELEVATED: &str = "sustained_load_elevated_v1";

/// Calm Insight category — distinct from demand / stress / High_Stress burst.
pub const PROLONGED_LOAD_CATEGORY: &str = "prolonged_load";

/// Emits one Insight when the latest `SustainedLoadIndicator` scalar is elevated.
#[derive(Debug, Default, Clone, Copy)]
pub struct SustainedLoadElevatedRule;

impl InsightRule for SustainedLoadElevatedRule {
    fn id(&self) -> &str {
        RULE_SUSTAINED_LOAD_ELEVATED
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        _pattern: &crate::PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let Some(load) = latest_feature(features, SUSTAINED_LOAD_INDICATOR_ID) else {
            return Ok(Vec::new());
        };
        let Some(value) = scalar_value(load) else {
            return Ok(Vec::new());
        };
        if value < SUSTAINED_LOAD_ELEVATED_THRESHOLD {
            return Ok(Vec::new());
        }

        let mut evidence = vec![EvidenceRef::Feature(load.feature_id.clone())];
        for id in [STRESS_INDEX_ID, FATIGUE_INDEX_ID, MEETING_DENSITY_ID] {
            if let Some(f) = latest_feature(features, id) {
                evidence.push(EvidenceRef::Feature(f.feature_id.clone()));
            }
        }

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Длительная нагрузка была выше обычного".into(),
            description: "Длительная нагрузка в этом окне была выше обычного.".into(),
            category: PROLONGED_LOAD_CATEGORY.into(),
            evidence_list: evidence,
            action_recommendation: Some(
                "Более короткий отрезок или спокойнее темп позже могут помочь, если это уместно.".into(),
            ),
        }])
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{Confidence, Feature, FeatureValue, TimeWindow, UnixTimestamp};

    use super::*;
    use crate::PatternInputs;

    fn feature(id: &str, end: i64, value: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
                .expect("window");
        Feature {
            feature_id: id.into(),
            time_window: window,
            value: FeatureValue::Scalar(value),
            provenance: vec![],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        }
    }

    fn object_feature(id: &str, end: i64) -> Feature {
        let start = end.saturating_sub(900);
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
                .expect("window");
        Feature {
            feature_id: id.into(),
            time_window: window,
            value: FeatureValue::Object(serde_json::Value::Null),
            provenance: vec![],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        }
    }

    #[test]
    fn emits_at_threshold_with_optional_evidence() {
        let rule = SustainedLoadElevatedRule;
        let features = vec![
            feature(
                SUSTAINED_LOAD_INDICATOR_ID,
                1800,
                SUSTAINED_LOAD_ELEVATED_THRESHOLD,
            ),
            feature(STRESS_INDEX_ID, 1800, 70.0),
            feature(FATIGUE_INDEX_ID, 1800, 65.0),
            feature(MEETING_DENSITY_ID, 1800, 30.0),
        ];
        let out = rule
            .evaluate(&features, &[], &PatternInputs::default())
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, PROLONGED_LOAD_CATEGORY);
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(SUSTAINED_LOAD_INDICATOR_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(STRESS_INDEX_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(FATIGUE_INDEX_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(MEETING_DENSITY_ID.into())));
        let blob = format!(
            "{} {} {}",
            out[0].title,
            out[0].description,
            out[0].action_recommendation.as_deref().unwrap_or("")
        )
        .to_lowercase();
        assert!(blob.contains("длительная нагрузка"));
        assert!(blob.contains("выше обычного"));
        for banned in [
            "burned out",
            "burnout",
            "exhaust",
            "diagnos",
            "disorder",
            "patholog",
            "medical",
            "surveillance",
            "manager",
            "workplace",
            "combined demand",
        ] {
            assert!(!blob.contains(banned), "banned `{banned}` in {blob}");
        }
    }

    #[test]
    fn omits_when_absent() {
        let rule = SustainedLoadElevatedRule;
        let out = rule
            .evaluate(&[], &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_below_threshold() {
        let rule = SustainedLoadElevatedRule;
        let f = feature(
            SUSTAINED_LOAD_INDICATOR_ID,
            1800,
            SUSTAINED_LOAD_ELEVATED_THRESHOLD - 0.01,
        );
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_non_scalar() {
        let rule = SustainedLoadElevatedRule;
        let f = object_feature(SUSTAINED_LOAD_INDICATOR_ID, 1800);
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn distinct_from_cognitive_load_feature() {
        let rule = SustainedLoadElevatedRule;
        // CognitiveLoad alone must not fire this rule.
        let f = feature("CognitiveLoad", 1800, 90.0);
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }
}
