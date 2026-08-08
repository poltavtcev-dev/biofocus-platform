//! Rule: calm pace hint when Focus looks lower than recent baseline (ADR-009).

use bio_spec::{EvidenceRef, Feature, Insight, Recommendation, Signal};
use uuid::Uuid;

use crate::rules::{latest_feature, FOCUS_BASELINE_CONFIDENCE_GATE, FOCUS_SCORE_ID};
use crate::{KnowledgeEngineResult, RecommendationRule};

/// Stable rule id for [`FocusDipPaceHintRule`].
pub const RULE_FOCUS_DIP_PACE_HINT: &str = "focus_dip_pace_hint_v1";

/// Pattern Insight category from `focus_vs_recent_baseline_v1`.
const PATTERN_CATEGORY: &str = "pattern";

/// Emits one calm pace/pause Recommendation when a Focus-below-baseline pattern
/// Insight is present and live `FocusScore` passes the confidence gate.
#[derive(Debug, Default, Clone, Copy)]
pub struct FocusDipPaceHintRule;

impl RecommendationRule for FocusDipPaceHintRule {
    fn id(&self) -> &str {
        RULE_FOCUS_DIP_PACE_HINT
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        insights: &[Insight],
    ) -> KnowledgeEngineResult<Vec<Recommendation>> {
        let Some(focus) = latest_feature(features, FOCUS_SCORE_ID) else {
            return Ok(Vec::new());
        };
        if focus.confidence.get() < FOCUS_BASELINE_CONFIDENCE_GATE {
            return Ok(Vec::new());
        }

        let Some(pattern) = find_focus_below_baseline_insight(insights) else {
            return Ok(Vec::new());
        };

        Ok(vec![Recommendation {
            id: Uuid::now_v7(),
            title: "A gentler pace may help".into(),
            suggestion: "If it fits your schedule, a short pause or slightly slower pace may help when focus looks lower than your recent average.".into(),
            category: "pace".into(),
            evidence_list: vec![
                EvidenceRef::Feature(FOCUS_SCORE_ID.into()),
                EvidenceRef::Insight(pattern.id),
            ],
        }])
    }
}

/// Match pattern Insight indicating Focus lower than recent average.
fn find_focus_below_baseline_insight(insights: &[Insight]) -> Option<&Insight> {
    insights.iter().find(|insight| {
        if insight.category != PATTERN_CATEGORY {
            return false;
        }
        let lower = insight.description.to_lowercase().contains("lower");
        let cites_focus = insight
            .evidence_list
            .iter()
            .any(|e| matches!(e, EvidenceRef::Feature(id) if id == FOCUS_SCORE_ID));
        lower && cites_focus
    })
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        Confidence, Feature, FeatureValue, Insight, TimeWindow, UnixTimestamp,
    };
    use uuid::Uuid;

    use super::*;
    use crate::KnowledgeEngine;

    fn focus(end: i64, confidence: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
                .expect("window");
        Feature {
            feature_id: FOCUS_SCORE_ID.into(),
            time_window: window,
            value: FeatureValue::Scalar(40.0),
            provenance: vec![],
            confidence: Confidence::saturating_from(confidence),
            factors: Vec::new(),
        }
    }

    fn lower_pattern_insight() -> Insight {
        Insight {
            id: Uuid::from_u128(11),
            title: "Focus relative to your recent average".into(),
            description: "Focus looks lower than your recent afternoon average.".into(),
            category: "pattern".into(),
            evidence_list: vec![EvidenceRef::Feature(FOCUS_SCORE_ID.into())],
            action_recommendation: None,
        }
    }

    fn higher_pattern_insight() -> Insight {
        Insight {
            id: Uuid::from_u128(12),
            title: "Focus relative to your recent average".into(),
            description: "Focus looks higher than your recent afternoon average.".into(),
            category: "pattern".into(),
            evidence_list: vec![EvidenceRef::Feature(FOCUS_SCORE_ID.into())],
            action_recommendation: None,
        }
    }

    #[test]
    fn rich_inputs_emit_with_feature_and_insight_evidence() {
        let rule = FocusDipPaceHintRule;
        let focus = focus(10_000, 1.0);
        let insight = lower_pattern_insight();
        let out = rule
            .evaluate(
                std::slice::from_ref(&focus),
                &[],
                std::slice::from_ref(&insight),
            )
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, "pace");
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(FOCUS_SCORE_ID.into())));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Insight(insight.id)));
        let blob = format!("{} {}", out[0].title, out[0].suggestion).to_lowercase();
        for word in ["diagnos", "disorder", "patholog", "unhealthy", "medical", "prescri"] {
            assert!(!blob.contains(word), "clinical term in {blob}");
        }
        assert!(blob.contains("pace") || blob.contains("pause"));
    }

    #[test]
    fn omits_without_matching_insight() {
        let rule = FocusDipPaceHintRule;
        let focus = focus(10_000, 1.0);
        let out = rule
            .evaluate(std::slice::from_ref(&focus), &[], &[])
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_when_focus_is_higher_than_baseline() {
        let rule = FocusDipPaceHintRule;
        let focus = focus(10_000, 1.0);
        let insight = higher_pattern_insight();
        let out = rule
            .evaluate(
                std::slice::from_ref(&focus),
                &[],
                std::slice::from_ref(&insight),
            )
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_on_low_confidence_focus() {
        let rule = FocusDipPaceHintRule;
        let focus = focus(10_000, 0.2);
        let insight = lower_pattern_insight();
        let out = rule
            .evaluate(
                std::slice::from_ref(&focus),
                &[],
                std::slice::from_ref(&insight),
            )
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn registered_via_recommendations_v1() {
        let mut engine = KnowledgeEngine::new();
        crate::register_recommendations_v1(&mut engine).expect("register");
        assert!(engine.recommendation_rule_count() >= 1);
    }
}
