//! Knowledge Engine: Features + Signals → Insights with Evidence.

use bio_spec::{Feature, Insight, Signal};

use crate::error::{KnowledgeEngineError, KnowledgeEngineResult};
use crate::rule::InsightRule;

/// Deterministic Insight generator over Feature / Signal inputs.
///
/// Default engine has **no product rules** (empty `Ok` is valid). Register rules
/// via [`Self::register`] or [`crate::register_insights_v1`].
#[derive(Default)]
pub struct KnowledgeEngine {
    rules: Vec<Box<dyn InsightRule>>,
}

impl KnowledgeEngine {
    /// Empty engine (no rules → always empty Insight list).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of registered rules.
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Register a rule. Duplicate [`InsightRule::id`] → error.
    pub fn register(&mut self, rule: impl InsightRule + 'static) -> KnowledgeEngineResult<()> {
        let id = rule.id().to_owned();
        if self.rules.iter().any(|r| r.id() == id) {
            return Err(KnowledgeEngineError::DuplicateRule { id });
        }
        self.rules.push(Box::new(rule));
        Ok(())
    }

    /// Run all rules; concatenate Insights. Empty inputs / no matches → `Ok([])`.
    pub fn evaluate(
        &self,
        features: &[Feature],
        signals: &[Signal],
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let mut insights = Vec::new();
        for rule in &self.rules {
            let batch = rule.evaluate(features, signals)?;
            insights.extend(batch);
        }
        Ok(insights)
    }
}

/// Convenience: evaluate with a fresh empty engine (no rules → empty Insights).
pub fn generate_insights(
    features: &[Feature],
    signals: &[Signal],
) -> KnowledgeEngineResult<Vec<Insight>> {
    KnowledgeEngine::new().evaluate(features, signals)
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        Confidence, EvidenceRef, Feature, FeatureValue, Insight, Severity, Signal, TimeWindow,
        UnixTimestamp,
    };
    use uuid::Uuid;

    use super::*;
    use crate::KnowledgeEngineError;

    /// Test-only rule: emits one Insight when any Feature or Signal is present.
    struct ScaffoldEchoRule;

    impl InsightRule for ScaffoldEchoRule {
        fn id(&self) -> &str {
            "scaffold_echo"
        }

        fn evaluate(
            &self,
            features: &[Feature],
            signals: &[Signal],
        ) -> KnowledgeEngineResult<Vec<Insight>> {
            let mut evidence = Vec::new();
            if let Some(f) = features.first() {
                evidence.push(EvidenceRef::Feature(f.feature_id.clone()));
            }
            if let Some(s) = signals.first() {
                evidence.push(EvidenceRef::Signal(s.id));
            }
            if evidence.is_empty() {
                return Ok(Vec::new());
            }
            Ok(vec![Insight {
                id: Uuid::now_v7(),
                title: "Scaffold insight".into(),
                description: "Test scaffold — not a product rule.".into(),
                category: "scaffold".into(),
                evidence_list: evidence,
                action_recommendation: None,
            }])
        }
    }

    struct FailingRule;

    impl InsightRule for FailingRule {
        fn id(&self) -> &str {
            "failing"
        }

        fn evaluate(
            &self,
            _features: &[Feature],
            _signals: &[Signal],
        ) -> KnowledgeEngineResult<Vec<Insight>> {
            Err(KnowledgeEngineError::RuleFailed {
                rule: self.id().into(),
                message: "intentional".into(),
            })
        }
    }

    fn sample_feature() -> Feature {
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("valid window");
        Feature {
            feature_id: "FocusScore".into(),
            time_window: window,
            value: FeatureValue::Scalar(72.5),
            provenance: vec![Uuid::from_u128(1)],
            confidence: Confidence::ONE,
        }
    }

    fn sample_signal() -> Signal {
        Signal {
            id: Uuid::from_u128(9),
            signal_type: "High_Stress".into(),
            timestamp_start: UnixTimestamp::from_secs(900),
            timestamp_end: UnixTimestamp::from_secs(1260),
            severity: Severity::High,
        }
    }

    #[test]
    fn empty_engine_returns_empty_insights() {
        let engine = KnowledgeEngine::new();
        let out = engine
            .evaluate(&[sample_feature()], &[sample_signal()])
            .expect("evaluate");
        assert!(out.is_empty());
    }

    #[test]
    fn generate_insights_helper_is_empty_without_rules() {
        let out = generate_insights(&[], &[]).expect("generate");
        assert!(out.is_empty());
    }

    #[test]
    fn empty_input_with_rule_yields_empty_vec() {
        let mut engine = KnowledgeEngine::new();
        engine.register(ScaffoldEchoRule).expect("register");
        let out = engine.evaluate(&[], &[]).expect("evaluate");
        assert!(out.is_empty());
    }

    #[test]
    fn no_match_yields_empty_vec() {
        // Rule only matches when Feature or Signal present — empty slices = no match.
        let mut engine = KnowledgeEngine::new();
        engine.register(ScaffoldEchoRule).expect("register");
        let out = engine.evaluate(&[], &[]).expect("evaluate");
        assert!(out.is_empty());
    }

    #[test]
    fn happy_path_produces_at_least_one_insight_with_evidence() {
        let mut engine = KnowledgeEngine::new();
        engine.register(ScaffoldEchoRule).expect("register");
        let feature = sample_feature();
        let signal = sample_signal();
        let out = engine
            .evaluate(std::slice::from_ref(&feature), std::slice::from_ref(&signal))
            .expect("evaluate");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].title, "Scaffold insight");
        assert!(
            out[0]
                .evidence_list
                .contains(&EvidenceRef::Feature("FocusScore".into()))
        );
        assert!(
            out[0]
                .evidence_list
                .contains(&EvidenceRef::Signal(signal.id))
        );
    }

    #[test]
    fn duplicate_rule_id_errors() {
        let mut engine = KnowledgeEngine::new();
        engine.register(ScaffoldEchoRule).expect("register");
        let err = engine.register(ScaffoldEchoRule).expect_err("duplicate");
        assert_eq!(
            err,
            KnowledgeEngineError::DuplicateRule {
                id: "scaffold_echo".into(),
            }
        );
    }

    #[test]
    fn rule_failure_propagates() {
        let mut engine = KnowledgeEngine::new();
        engine.register(FailingRule).expect("register");
        let err = engine.evaluate(&[], &[]).expect_err("fail");
        assert_eq!(
            err,
            KnowledgeEngineError::RuleFailed {
                rule: "failing".into(),
                message: "intentional".into(),
            }
        );
    }
}
