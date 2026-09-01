//! Rule: elevated `CognitiveLoad` → calm combined-demand Insight (ADR-028).

use bio_spec::{EvidenceRef, Feature, Insight, Signal};
use uuid::Uuid;

use super::context_switch::CONTEXT_SWITCH_RATE_ID;
use crate::rules::{latest_feature, scalar_value};
use crate::{InsightRule, KnowledgeEngineResult};

/// Catalog Feature id `CognitiveLoad` (required).
pub const COGNITIVE_LOAD_ID: &str = "CognitiveLoad";

/// Optional Evidence Feature ids when present in the snapshot.
pub const MEETING_DENSITY_ID: &str = "MeetingDensity";
pub const NOTIFICATION_PRESSURE_ID: &str = "NotificationPressure";

/// Latest scalar CognitiveLoad at/above which the rule fires (0–100; ADR-028 ±10 OK).
pub const COGNITIVE_LOAD_ELEVATED_THRESHOLD: f64 = 60.0;

/// Stable rule id for [`CognitiveLoadElevatedRule`].
pub const RULE_COGNITIVE_LOAD_ELEVATED: &str = "cognitive_load_elevated_v1";

/// Calm Insight category — distinct from stress / focus / pattern.
pub const DEMAND_CATEGORY: &str = "demand";

/// Emits one Insight when the latest `CognitiveLoad` scalar is elevated.
#[derive(Debug, Default, Clone, Copy)]
pub struct CognitiveLoadElevatedRule;

impl InsightRule for CognitiveLoadElevatedRule {
    fn id(&self) -> &str {
        RULE_COGNITIVE_LOAD_ELEVATED
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        _pattern: &crate::PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let Some(load) = latest_feature(features, COGNITIVE_LOAD_ID) else {
            return Ok(Vec::new());
        };
        let Some(value) = scalar_value(load) else {
            return Ok(Vec::new());
        };
        if value < COGNITIVE_LOAD_ELEVATED_THRESHOLD {
            return Ok(Vec::new());
        }

        let mut evidence = vec![EvidenceRef::Feature(load.feature_id.clone())];
        for id in [
            MEETING_DENSITY_ID,
            CONTEXT_SWITCH_RATE_ID,
            NOTIFICATION_PRESSURE_ID,
        ] {
            if let Some(f) = latest_feature(features, id) {
                evidence.push(EvidenceRef::Feature(f.feature_id.clone()));
            }
        }

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Combined demand looked elevated".into(),
            description: "Combined demand looked elevated in this window.".into(),
            category: DEMAND_CATEGORY.into(),
            evidence_list: evidence,
            action_recommendation: Some(
                "If it fits, easing parallel demands for a stretch may help.".into(),
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
        let rule = CognitiveLoadElevatedRule;
        let features = vec![
            feature(COGNITIVE_LOAD_ID, 1800, COGNITIVE_LOAD_ELEVATED_THRESHOLD),
            feature(MEETING_DENSITY_ID, 1800, 40.0),
            feature(CONTEXT_SWITCH_RATE_ID, 1800, 1.2),
            feature(NOTIFICATION_PRESSURE_ID, 1800, 55.0),
        ];
        let out = rule
            .evaluate(&features, &[], &PatternInputs::default())
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, DEMAND_CATEGORY);
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(COGNITIVE_LOAD_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(MEETING_DENSITY_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(CONTEXT_SWITCH_RATE_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(NOTIFICATION_PRESSURE_ID.into())));
        let blob = format!(
            "{} {} {}",
            out[0].title,
            out[0].description,
            out[0].action_recommendation.as_deref().unwrap_or("")
        )
        .to_lowercase();
        assert!(blob.contains("combined demand"));
        assert!(blob.contains("elevated"));
        for banned in [
            "overload",
            "burnout",
            "burned out",
            "workplace",
            "diagnos",
            "disorder",
            "patholog",
            "medical",
            "employer",
        ] {
            assert!(!blob.contains(banned), "banned `{banned}` in {blob}");
        }
    }

    #[test]
    fn omits_when_absent() {
        let rule = CognitiveLoadElevatedRule;
        let out = rule
            .evaluate(&[], &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_below_threshold() {
        let rule = CognitiveLoadElevatedRule;
        let f = feature(
            COGNITIVE_LOAD_ID,
            1800,
            COGNITIVE_LOAD_ELEVATED_THRESHOLD - 0.01,
        );
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_non_scalar() {
        let rule = CognitiveLoadElevatedRule;
        let f = object_feature(COGNITIVE_LOAD_ID, 1800);
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn optional_evidence_not_required() {
        let rule = CognitiveLoadElevatedRule;
        let f = feature(COGNITIVE_LOAD_ID, 1800, 75.0);
        let out = rule
            .evaluate(std::slice::from_ref(&f), &[], &PatternInputs::default())
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].evidence_list.len(), 1);
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(COGNITIVE_LOAD_ID.into())));
    }
}
