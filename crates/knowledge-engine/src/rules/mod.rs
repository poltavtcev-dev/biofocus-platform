//! Product Insight rules (P4-E2-T2) — deterministic, no LLM / SQLite.
//!
//! | Rule id | Trigger | Evidence |
//! | :--- | :--- | :--- |
//! | [`RULE_HIGH_STRESS_PERIOD`] | Signal `High_Stress` | Signal id(s); optional `StressIndex` |
//! | [`RULE_CONTEXT_SWITCH_ELEVATED`] | latest `ContextSwitchRate` ≥ threshold | `ContextSwitchRate`; optional `FocusScore` |
//!
//! Host registration:
//! ```ignore
//! let mut engine = KnowledgeEngine::new();
//! register_insights_v1(&mut engine)?;
//! let insights = engine.evaluate(&features, &signals)?;
//! ```
//!
//! Default [`crate::KnowledgeEngine::new`] stays empty — call [`register_insights_v1`]
//! (or register rules individually) before expecting product Insights.

mod context_switch;
mod high_stress;

pub use context_switch::{
    ContextSwitchElevatedRule, CONTEXT_SWITCH_ELEVATED_THRESHOLD, CONTEXT_SWITCH_RATE_ID,
    FOCUS_SCORE_ID, RULE_CONTEXT_SWITCH_ELEVATED,
};
pub use high_stress::{
    HighStressPeriodRule, HIGH_STRESS_SIGNAL_TYPE, RULE_HIGH_STRESS_PERIOD, STRESS_INDEX_ID,
};

use bio_spec::{Feature, FeatureValue};

use crate::{KnowledgeEngine, KnowledgeEngineResult};

/// Registers v1 product Insight rules (≥2): High_Stress period + elevated ContextSwitch.
pub fn register_insights_v1(engine: &mut KnowledgeEngine) -> KnowledgeEngineResult<()> {
    engine.register(HighStressPeriodRule)?;
    engine.register(ContextSwitchElevatedRule)?;
    Ok(())
}

/// Latest Feature with `feature_id` by `time_window.end` (ties → last in slice order).
pub(crate) fn latest_feature<'a>(features: &'a [Feature], feature_id: &str) -> Option<&'a Feature> {
    features
        .iter()
        .filter(|f| f.feature_id == feature_id)
        .max_by_key(|f| f.time_window.end.as_secs())
}

pub(crate) fn scalar_value(feature: &Feature) -> Option<f64> {
    match feature.value {
        FeatureValue::Scalar(v) => Some(v),
        FeatureValue::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        EvidenceRef, Feature, FeatureValue, Severity, Signal, TimeWindow, UnixTimestamp,
    };
    use uuid::Uuid;

    use super::*;
    use crate::KnowledgeEngine;

    fn feature(id: &str, end: i64, value: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window = TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
            .expect("window");
        Feature {
            feature_id: id.into(),
            time_window: window,
            value: FeatureValue::Scalar(value),
            provenance: vec![],
        }
    }

    fn high_stress_signal(id: u128) -> Signal {
        Signal {
            id: Uuid::from_u128(id),
            signal_type: HIGH_STRESS_SIGNAL_TYPE.into(),
            timestamp_start: UnixTimestamp::from_secs(900),
            timestamp_end: UnixTimestamp::from_secs(1261),
            severity: Severity::High,
        }
    }

    #[test]
    fn register_insights_v1_adds_two_rules() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        assert_eq!(engine.rule_count(), 2);
    }

    #[test]
    fn high_stress_triggers_with_signal_evidence() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let stress = feature(STRESS_INDEX_ID, 1800, 80.0);
        let signal = high_stress_signal(42);
        let out = engine
            .evaluate(std::slice::from_ref(&stress), std::slice::from_ref(&signal))
            .expect("evaluate");
        let insight = out
            .iter()
            .find(|i| i.category == "stress")
            .expect("stress insight");
        assert!(insight
            .evidence_list
            .contains(&EvidenceRef::Signal(signal.id)));
        assert!(insight
            .evidence_list
            .contains(&EvidenceRef::Feature(STRESS_INDEX_ID.into())));
        assert!(!insight.title.is_empty());
        assert!(!insight.description.is_empty());
    }

    #[test]
    fn high_stress_no_trigger_without_signal() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let stress = feature(STRESS_INDEX_ID, 1800, 80.0);
        let out = engine.evaluate(std::slice::from_ref(&stress), &[]).expect("evaluate");
        assert!(out.iter().all(|i| i.category != "stress"));
    }

    #[test]
    fn context_switch_triggers_with_feature_evidence() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let csr = feature(CONTEXT_SWITCH_RATE_ID, 1800, CONTEXT_SWITCH_ELEVATED_THRESHOLD);
        let focus = feature(FOCUS_SCORE_ID, 1800, 28.0);
        let out = engine
            .evaluate(&[csr, focus], &[])
            .expect("evaluate");
        let insight = out
            .iter()
            .find(|i| i.category == "focus")
            .expect("focus insight");
        assert!(insight
            .evidence_list
            .contains(&EvidenceRef::Feature(CONTEXT_SWITCH_RATE_ID.into())));
        assert!(insight
            .evidence_list
            .contains(&EvidenceRef::Feature(FOCUS_SCORE_ID.into())));
    }

    #[test]
    fn context_switch_no_trigger_below_threshold() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let csr = feature(
            CONTEXT_SWITCH_RATE_ID,
            1800,
            CONTEXT_SWITCH_ELEVATED_THRESHOLD - 0.01,
        );
        let out = engine.evaluate(std::slice::from_ref(&csr), &[]).expect("evaluate");
        assert!(out.iter().all(|i| i.category != "focus"));
    }

    #[test]
    fn empty_input_yields_no_insights() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let out = engine.evaluate(&[], &[]).expect("evaluate");
        assert!(out.is_empty());
    }

    #[test]
    fn both_rules_can_fire_together() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let csr = feature(CONTEXT_SWITCH_RATE_ID, 1800, 2.0);
        let signal = high_stress_signal(7);
        let out = engine
            .evaluate(std::slice::from_ref(&csr), std::slice::from_ref(&signal))
            .expect("evaluate");
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn copy_avoids_clinical_words() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let csr = feature(CONTEXT_SWITCH_RATE_ID, 1800, 2.0);
        let signal = high_stress_signal(1);
        let out = engine
            .evaluate(std::slice::from_ref(&csr), std::slice::from_ref(&signal))
            .expect("evaluate");
        let banned = ["diagnos", "disorder", "patholog", "unhealthy", "dangerous", "medical"];
        for insight in &out {
            let blob = format!(
                "{} {} {}",
                insight.title,
                insight.description,
                insight.action_recommendation.as_deref().unwrap_or("")
            )
            .to_lowercase();
            for word in banned {
                assert!(
                    !blob.contains(word),
                    "clinical/evaluative term `{word}` in: {blob}"
                );
            }
        }
    }
}
