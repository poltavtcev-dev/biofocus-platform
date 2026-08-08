//! Product Insight + Recommendation rules — deterministic, no LLM / SQLite.
//!
//! | Rule id | Trigger | Evidence |
//! | :--- | :--- | :--- |
//! | [`RULE_HIGH_STRESS_PERIOD`] | Signal `High_Stress` | Signal id(s); optional `StressIndex` |
//! | [`RULE_CONTEXT_SWITCH_ELEVATED`] | latest `ContextSwitchRate` ≥ threshold | `ContextSwitchRate`; optional `FocusScore` |
//! | [`RULE_FOCUS_VS_RECENT_BASELINE`] | live FocusScore vs afternoon baseline | `FocusScore` |
//! | [`RULE_FOCUS_DIP_PACE_HINT`] | Focus-below-baseline pattern Insight | `FocusScore` + Insight id |
//!
//! Host registration:
//! ```ignore
//! let mut engine = KnowledgeEngine::new();
//! register_insights_v1(&mut engine)?;
//! register_recommendations_v1(&mut engine)?;
//! let (insights, recommendations) =
//!     engine.evaluate_insights_and_recommendations(&features, &signals, &pattern)?;
//! ```
//!
//! Default [`crate::KnowledgeEngine::new`] stays empty — call the register helpers
//! (or register rules individually) before expecting product Insights / Recommendations.

mod context_switch;
mod focus_baseline;
mod focus_dip_pace;
mod high_stress;

pub use context_switch::{
    ContextSwitchElevatedRule, CONTEXT_SWITCH_ELEVATED_THRESHOLD, CONTEXT_SWITCH_RATE_ID,
    FOCUS_SCORE_ID, RULE_CONTEXT_SWITCH_ELEVATED,
};
pub use focus_baseline::{
    FocusVsRecentBaselineRule, FOCUS_BASELINE_CONFIDENCE_GATE, FOCUS_BASELINE_DELTA,
    FOCUS_BASELINE_MAX_WINDOWS, FOCUS_BASELINE_MIN_WINDOWS, RULE_FOCUS_VS_RECENT_BASELINE,
};
pub use focus_dip_pace::{FocusDipPaceHintRule, RULE_FOCUS_DIP_PACE_HINT};
pub use high_stress::{
    HighStressPeriodRule, HIGH_STRESS_SIGNAL_TYPE, RULE_HIGH_STRESS_PERIOD, STRESS_INDEX_ID,
};

use bio_spec::{Feature, FeatureValue};

use crate::{KnowledgeEngine, KnowledgeEngineResult};

/// Registers v1 product Insight rules: High_Stress, elevated ContextSwitch,
/// Focus vs recent afternoon baseline (ADR-008).
pub fn register_insights_v1(engine: &mut KnowledgeEngine) -> KnowledgeEngineResult<()> {
    engine.register(HighStressPeriodRule)?;
    engine.register(ContextSwitchElevatedRule)?;
    engine.register(FocusVsRecentBaselineRule)?;
    Ok(())
}

/// Registers v1 product Recommendation rules (ADR-009 / P9-E2-T1).
pub fn register_recommendations_v1(engine: &mut KnowledgeEngine) -> KnowledgeEngineResult<()> {
    engine.register_recommendation(FocusDipPaceHintRule)?;
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
        Confidence, EvidenceRef, Feature, FeatureValue, Severity, Signal, TimeWindow, UnixTimestamp,
    };
    use uuid::Uuid;

    use super::*;
    use crate::{KnowledgeEngine, PatternInputs};

    fn feature(id: &str, end: i64, value: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window = TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
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
    fn register_insights_v1_adds_three_rules() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        assert_eq!(engine.rule_count(), 3);
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
    fn both_snapshot_rules_can_fire_together() {
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
    fn baseline_rule_fires_with_pattern_series() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let current = feature(FOCUS_SCORE_ID, 10_000, 85.0);
        let pattern = PatternInputs::with_baseline_series(vec![
            feature(FOCUS_SCORE_ID, 1_000, 60.0),
            feature(FOCUS_SCORE_ID, 2_000, 58.0),
            feature(FOCUS_SCORE_ID, 3_000, 62.0),
        ]);
        let out = engine
            .evaluate_with_pattern(std::slice::from_ref(&current), &[], &pattern)
            .expect("evaluate");
        let insight = out
            .iter()
            .find(|i| i.category == "pattern")
            .expect("pattern insight");
        assert!(insight.description.contains("higher"));
    }

    #[test]
    fn copy_avoids_clinical_words() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let csr = feature(CONTEXT_SWITCH_RATE_ID, 1800, 2.0);
        let signal = high_stress_signal(1);
        let focus = feature(FOCUS_SCORE_ID, 10_000, 90.0);
        let pattern = PatternInputs::with_baseline_series(vec![
            feature(FOCUS_SCORE_ID, 1_000, 50.0),
            feature(FOCUS_SCORE_ID, 2_000, 52.0),
        ]);
        let out = engine
            .evaluate_with_pattern(&[csr, focus], std::slice::from_ref(&signal), &pattern)
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

    #[test]
    fn focus_dip_recommendation_fires_after_lower_baseline_insight() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("insights");
        register_recommendations_v1(&mut engine).expect("recommendations");
        // Current Focus well below afternoon baseline → pattern Insight + pace Recommendation.
        let current = feature(FOCUS_SCORE_ID, 10_000, 40.0);
        let pattern = PatternInputs::with_baseline_series(vec![
            feature(FOCUS_SCORE_ID, 1_000, 70.0),
            feature(FOCUS_SCORE_ID, 2_000, 72.0),
            feature(FOCUS_SCORE_ID, 3_000, 68.0),
        ]);
        let (insights, recommendations) = engine
            .evaluate_insights_and_recommendations(std::slice::from_ref(&current), &[], &pattern)
            .expect("evaluate");
        let pattern_insight = insights
            .iter()
            .find(|i| i.category == "pattern")
            .expect("pattern insight");
        assert!(pattern_insight.description.contains("lower"));
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(FOCUS_SCORE_ID.into())));
        assert!(recommendations[0]
            .evidence_list
            .contains(&EvidenceRef::Insight(pattern_insight.id)));
    }

    #[test]
    fn recommendations_empty_without_registration() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("insights");
        let current = feature(FOCUS_SCORE_ID, 10_000, 40.0);
        let pattern = PatternInputs::with_baseline_series(vec![
            feature(FOCUS_SCORE_ID, 1_000, 70.0),
            feature(FOCUS_SCORE_ID, 2_000, 72.0),
        ]);
        let insights = engine
            .evaluate_with_pattern(std::slice::from_ref(&current), &[], &pattern)
            .expect("insights");
        let recommendations = engine
            .evaluate_recommendations(std::slice::from_ref(&current), &[], &insights)
            .expect("recommendations");
        assert!(recommendations.is_empty());
    }
}
