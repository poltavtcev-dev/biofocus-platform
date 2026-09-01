//! Rule: `High_Stress` Signal → calm Insight with Evidence.

use bio_spec::{EvidenceRef, Feature, Insight, Signal};
use uuid::Uuid;

use crate::rules::latest_feature;
use crate::{InsightRule, KnowledgeEngineResult};

/// Matches `feature_engine::HIGH_STRESS_SIGNAL_TYPE` (string contract; no crate dep).
pub const HIGH_STRESS_SIGNAL_TYPE: &str = "High_Stress";

/// Optional Feature id attached when present in the input snapshot.
pub const STRESS_INDEX_ID: &str = "StressIndex";

/// Stable rule id for [`HighStressPeriodRule`].
pub const RULE_HIGH_STRESS_PERIOD: &str = "high_stress_period_v1";

/// Emits one Insight when at least one `High_Stress` Signal is present.
#[derive(Debug, Default, Clone, Copy)]
pub struct HighStressPeriodRule;

impl InsightRule for HighStressPeriodRule {
    fn id(&self) -> &str {
        RULE_HIGH_STRESS_PERIOD
    }

    fn evaluate(
        &self,
        features: &[Feature],
        signals: &[Signal],
        _pattern: &crate::PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let matching: Vec<&Signal> = signals
            .iter()
            .filter(|s| s.signal_type == HIGH_STRESS_SIGNAL_TYPE)
            .collect();
        if matching.is_empty() {
            return Ok(Vec::new());
        }

        let mut evidence: Vec<EvidenceRef> = matching
            .iter()
            .map(|s| EvidenceRef::Signal(s.id))
            .collect();
        if let Some(stress) = latest_feature(features, STRESS_INDEX_ID) {
            evidence.push(EvidenceRef::Feature(stress.feature_id.clone()));
        }

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Sustained stress pattern".into(),
            description: "Stress stayed elevated long enough in this period to raise a High_Stress signal.".into(),
            category: "stress".into(),
            evidence_list: evidence,
            action_recommendation: Some(
                "A brief pause or slower pace may help when it fits your schedule.".into(),
            ),
        }])
    }
}
