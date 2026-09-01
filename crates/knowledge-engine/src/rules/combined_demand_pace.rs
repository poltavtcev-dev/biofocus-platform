//! Rule: calm pace hint when combined-demand Insight is present (ADR-028 / ADR-009 style).

use bio_spec::{EvidenceRef, Feature, Insight, Recommendation, Signal};
use uuid::Uuid;

use crate::rules::{latest_feature, COGNITIVE_LOAD_ID, DEMAND_CATEGORY};
use crate::{KnowledgeEngineResult, RecommendationRule};

/// Stable rule id for [`CombinedDemandPaceHintRule`].
pub const RULE_COMBINED_DEMAND_PACE_HINT: &str = "combined_demand_pace_hint_v1";

/// Emits one gentle pace/pause Recommendation when a `cognitive_load_elevated_v1`
/// Insight is present. Optional latest `CognitiveLoad` is attached as Evidence.
#[derive(Debug, Default, Clone, Copy)]
pub struct CombinedDemandPaceHintRule;

impl RecommendationRule for CombinedDemandPaceHintRule {
    fn id(&self) -> &str {
        RULE_COMBINED_DEMAND_PACE_HINT
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        insights: &[Insight],
    ) -> KnowledgeEngineResult<Vec<Recommendation>> {
        let Some(demand) = find_cognitive_load_elevated_insight(insights) else {
            return Ok(Vec::new());
        };

        let mut evidence = vec![EvidenceRef::Insight(demand.id)];
        if latest_feature(features, COGNITIVE_LOAD_ID).is_some() {
            evidence.push(EvidenceRef::Feature(COGNITIVE_LOAD_ID.into()));
        }

        Ok(vec![Recommendation {
            id: Uuid::now_v7(),
            title: "Ease the pace for a moment".into(),
            suggestion: "If it fits your schedule, a short pause or fewer parallel demands may help when combined demand looks elevated.".into(),
            category: "pace".into(),
            evidence_list: evidence,
        }])
    }
}

/// Match the evaluate-on-read Insight from `cognitive_load_elevated_v1`
/// (category + CognitiveLoad Evidence + calm copy stance — ADR-009 style).
fn find_cognitive_load_elevated_insight(insights: &[Insight]) -> Option<&Insight> {
    insights.iter().find(|insight| {
        if insight.category != DEMAND_CATEGORY {
            return false;
        }
        let elevated_demand = {
            let d = insight.description.to_lowercase();
            d.contains("combined demand") && d.contains("elevated")
        };
        let cites_load = insight.evidence_list.iter().any(|e| {
            matches!(e, EvidenceRef::Feature(id) if id == COGNITIVE_LOAD_ID)
        });
        elevated_demand && cites_load
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

    fn cognitive_load(end: i64, value: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
                .expect("window");
        Feature {
            feature_id: COGNITIVE_LOAD_ID.into(),
            time_window: window,
            value: FeatureValue::Scalar(value),
            provenance: vec![],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        }
    }

    fn demand_insight() -> Insight {
        Insight {
            id: Uuid::from_u128(21),
            title: "Combined demand looked elevated".into(),
            description: "Combined demand looked elevated in this window.".into(),
            category: DEMAND_CATEGORY.into(),
            evidence_list: vec![EvidenceRef::Feature(COGNITIVE_LOAD_ID.into())],
            action_recommendation: None,
        }
    }

    fn unrelated_insight() -> Insight {
        Insight {
            id: Uuid::from_u128(22),
            title: "Prolonged load looked elevated".into(),
            description: "Prolonged load looked elevated in this window.".into(),
            category: "prolonged_load".into(),
            evidence_list: vec![EvidenceRef::Feature("SustainedLoadIndicator".into())],
            action_recommendation: None,
        }
    }

    #[test]
    fn emits_when_demand_insight_present() {
        let rule = CombinedDemandPaceHintRule;
        let load = cognitive_load(10_000, 70.0);
        let insight = demand_insight();
        let out = rule
            .evaluate(
                std::slice::from_ref(&load),
                &[],
                std::slice::from_ref(&insight),
            )
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, "pace");
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Insight(insight.id)));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(COGNITIVE_LOAD_ID.into())));
        let blob = format!("{} {}", out[0].title, out[0].suggestion).to_lowercase();
        assert!(blob.contains("pace") || blob.contains("pause"));
        for banned in [
            "diagnos",
            "disorder",
            "patholog",
            "medical",
            "prescri",
            "clinical",
            "employer",
            "coach",
            "burnout",
        ] {
            assert!(!blob.contains(banned), "banned `{banned}` in {blob}");
        }
    }

    #[test]
    fn omits_without_matching_insight() {
        let rule = CombinedDemandPaceHintRule;
        let load = cognitive_load(10_000, 70.0);
        let out = rule
            .evaluate(std::slice::from_ref(&load), &[], &[])
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn omits_on_unrelated_insight() {
        let rule = CombinedDemandPaceHintRule;
        let load = cognitive_load(10_000, 70.0);
        let insight = unrelated_insight();
        let out = rule
            .evaluate(
                std::slice::from_ref(&load),
                &[],
                std::slice::from_ref(&insight),
            )
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn emits_without_live_cognitive_load_feature() {
        let rule = CombinedDemandPaceHintRule;
        let insight = demand_insight();
        let out = rule
            .evaluate(&[], &[], std::slice::from_ref(&insight))
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Insight(insight.id)));
        assert!(!out[0]
            .evidence_list
            .iter()
            .any(|e| matches!(e, EvidenceRef::Feature(_))));
    }

    #[test]
    fn registered_via_recommendations_v1() {
        let mut engine = KnowledgeEngine::new();
        crate::register_recommendations_v1(&mut engine).expect("register");
        assert!(engine.recommendation_rule_count() >= 2);
    }
}
