//! Rule: elevated `ContextSwitchRate` (+ optional `FocusScore`) → calm Insight.

use bio_spec::{EvidenceRef, Feature, Insight, Signal};
use uuid::Uuid;

use crate::rules::{latest_feature, scalar_value};
use crate::{InsightRule, KnowledgeEngineResult};

/// Matches catalog Feature id `ContextSwitchRate`.
pub const CONTEXT_SWITCH_RATE_ID: &str = "ContextSwitchRate";

/// Optional supporting Feature id.
pub const FOCUS_SCORE_ID: &str = "FocusScore";

/// Switches-per-nominal-minute at/above which the rule fires (catalog units).
pub const CONTEXT_SWITCH_ELEVATED_THRESHOLD: f64 = 1.0;

/// Stable rule id for [`ContextSwitchElevatedRule`].
pub const RULE_CONTEXT_SWITCH_ELEVATED: &str = "context_switch_elevated_v1";

/// Emits one Insight when the latest `ContextSwitchRate` is elevated.
#[derive(Debug, Default, Clone, Copy)]
pub struct ContextSwitchElevatedRule;

impl InsightRule for ContextSwitchElevatedRule {
    fn id(&self) -> &str {
        RULE_CONTEXT_SWITCH_ELEVATED
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        _pattern: &crate::PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let Some(csr) = latest_feature(features, CONTEXT_SWITCH_RATE_ID) else {
            return Ok(Vec::new());
        };
        let Some(rate) = scalar_value(csr) else {
            return Ok(Vec::new());
        };
        if rate < CONTEXT_SWITCH_ELEVATED_THRESHOLD {
            return Ok(Vec::new());
        }

        let mut evidence = vec![EvidenceRef::Feature(csr.feature_id.clone())];
        if let Some(focus) = latest_feature(features, FOCUS_SCORE_ID) {
            evidence.push(EvidenceRef::Feature(focus.feature_id.clone()));
        }

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Frequent context changes".into(),
            description: "ContextSwitchRate was elevated in the latest window, a pattern that often aligns with a shallower FocusScore.".into(),
            category: "focus".into(),
            evidence_list: evidence,
            action_recommendation: Some(
                "Grouping similar tasks for a stretch can reduce switching when useful.".into(),
            ),
        }])
    }
}
