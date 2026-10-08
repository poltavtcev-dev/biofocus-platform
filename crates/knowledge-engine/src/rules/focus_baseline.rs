//! Rule: live FocusScore vs recomputed recent afternoon baseline (ADR-008).

use bio_spec::{EvidenceRef, Feature, FeatureValue, Insight, Signal};
use uuid::Uuid;

use crate::pattern::PatternInputs;
use crate::rules::{latest_feature, scalar_value};
use crate::{InsightRule, KnowledgeEngineResult};

/// Catalog Feature id (same string as `context_switch::FOCUS_SCORE_ID`).
const FOCUS_SCORE_ID: &str = "FocusScore";

/// Stable rule id for [`FocusVsRecentBaselineRule`].
pub const RULE_FOCUS_VS_RECENT_BASELINE: &str = "focus_vs_recent_baseline_v1";

/// Minimum |current − baseline| (FocusScore points) to emit an Insight.
pub const FOCUS_BASELINE_DELTA: f64 = 10.0;

/// Floor for the live score and the prior windows.
///
/// Matches the feature-engine baseline gate: one-of-three Focus coverage is
/// about 0.33 and must still be comparable.
pub const FOCUS_BASELINE_CONFIDENCE_GATE: f64 = 0.30;

/// Minimum prior windows required for a usable baseline mean.
pub const FOCUS_BASELINE_MIN_WINDOWS: usize = 2;

/// Max prior windows consumed from [`PatternInputs::baseline_series`].
pub const FOCUS_BASELINE_MAX_WINDOWS: usize = 7;

/// Emits one calm Insight when live FocusScore diverges from the recent
/// afternoon baseline by ≥ [`FOCUS_BASELINE_DELTA`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FocusVsRecentBaselineRule;

impl InsightRule for FocusVsRecentBaselineRule {
    fn id(&self) -> &str {
        RULE_FOCUS_VS_RECENT_BASELINE
    }

    fn evaluate(
        &self,
        features: &[Feature],
        _signals: &[Signal],
        pattern: &PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let Some(current_feat) = latest_feature(features, FOCUS_SCORE_ID) else {
            return Ok(Vec::new());
        };
        if current_feat.confidence.get() < FOCUS_BASELINE_CONFIDENCE_GATE {
            return Ok(Vec::new());
        }
        let Some(current) = scalar_value(current_feat) else {
            return Ok(Vec::new());
        };

        let series: Vec<&Feature> = pattern
            .baseline_series
            .iter()
            .filter(|f| f.feature_id == FOCUS_SCORE_ID)
            .filter(|f| f.confidence.get() >= FOCUS_BASELINE_CONFIDENCE_GATE)
            .filter(|f| matches!(f.value, FeatureValue::Scalar(v) if v.is_finite()))
            .take(FOCUS_BASELINE_MAX_WINDOWS)
            .collect();

        if series.len() < FOCUS_BASELINE_MIN_WINDOWS {
            return Ok(Vec::new());
        }

        let mut sum = 0.0;
        let mut n = 0usize;
        for f in &series {
            if let FeatureValue::Scalar(v) = f.value {
                sum += v;
                n += 1;
            }
        }
        if n < FOCUS_BASELINE_MIN_WINDOWS {
            return Ok(Vec::new());
        }
        let baseline = sum / n as f64;
        let delta = current - baseline;
        if delta.abs() < FOCUS_BASELINE_DELTA {
            return Ok(Vec::new());
        }

        let (description, action) = if delta > 0.0 {
            (
                "Focus looks higher than your recent afternoon average.".to_owned(),
                Some(
                    "Noticing a stronger focus stretch than recent afternoons — keep the setup that is working if it still feels right."
                        .to_owned(),
                ),
            )
        } else {
            (
                "Focus looks lower than your recent afternoon average.".to_owned(),
                Some(
                    "A gentler afternoon stretch than your recent average — a short reset or quieter block can help when useful."
                        .to_owned(),
                ),
            )
        };

        Ok(vec![Insight {
            id: Uuid::now_v7(),
            title: "Focus relative to your recent average".into(),
            description,
            category: "pattern".into(),
            evidence_list: vec![EvidenceRef::Feature(FOCUS_SCORE_ID.into())],
            action_recommendation: action,
        }])
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        Confidence, Feature, FeatureValue, TimeWindow, UnixTimestamp,
    };

    use super::*;
    use crate::KnowledgeEngine;

    fn focus(end: i64, value: f64, confidence: f64) -> Feature {
        let start = end.saturating_sub(900);
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
                .expect("window");
        Feature {
            feature_id: FOCUS_SCORE_ID.into(),
            time_window: window,
            value: FeatureValue::Scalar(value),
            provenance: vec![],
            confidence: Confidence::saturating_from(confidence),
            factors: Vec::new(),
        }
    }

    #[test]
    fn thin_series_omits_insight() {
        let rule = FocusVsRecentBaselineRule;
        let current = focus(10_000, 80.0, 1.0);
        let pattern = PatternInputs::with_baseline_series(vec![focus(1_000, 50.0, 1.0)]);
        let out = rule
            .evaluate(std::slice::from_ref(&current), &[], &pattern)
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn rich_series_emits_with_evidence_and_calm_copy() {
        let rule = FocusVsRecentBaselineRule;
        let current = focus(10_000, 85.0, 1.0);
        let series = vec![
            focus(1_000, 60.0, 1.0),
            focus(2_000, 62.0, 1.0),
            focus(3_000, 58.0, 0.9),
        ];
        let pattern = PatternInputs::with_baseline_series(series);
        let out = rule
            .evaluate(std::slice::from_ref(&current), &[], &pattern)
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, "pattern");
        assert!(out[0].description.contains("higher"));
        assert!(out[0]
            .evidence_list
            .contains(&EvidenceRef::Feature(FOCUS_SCORE_ID.into())));
        let blob = format!(
            "{} {} {}",
            out[0].title,
            out[0].description,
            out[0].action_recommendation.as_deref().unwrap_or("")
        )
        .to_lowercase();
        for word in ["diagnos", "disorder", "patholog", "unhealthy", "medical"] {
            assert!(!blob.contains(word), "clinical term in {blob}");
        }
    }

    #[test]
    fn one_of_three_coverage_still_compares() {
        let rule = FocusVsRecentBaselineRule;
        let current = focus(10_000, 80.0, 0.33);
        let pattern = PatternInputs::with_baseline_series(vec![
            focus(1_000, 50.0, 0.33),
            focus(2_000, 52.0, 0.33),
        ]);
        let out = rule
            .evaluate(std::slice::from_ref(&current), &[], &pattern)
            .expect("ok");
        assert_eq!(out.len(), 1);
        assert!(out[0].description.contains("higher"));
    }

    #[test]
    fn low_confidence_current_omits() {
        let rule = FocusVsRecentBaselineRule;
        let current = focus(10_000, 90.0, 0.2);
        let pattern = PatternInputs::with_baseline_series(vec![
            focus(1_000, 50.0, 1.0),
            focus(2_000, 52.0, 1.0),
        ]);
        let out = rule
            .evaluate(std::slice::from_ref(&current), &[], &pattern)
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn small_delta_omits() {
        let rule = FocusVsRecentBaselineRule;
        let current = focus(10_000, 65.0, 1.0);
        let pattern = PatternInputs::with_baseline_series(vec![
            focus(1_000, 60.0, 1.0),
            focus(2_000, 62.0, 1.0),
        ]);
        let out = rule
            .evaluate(std::slice::from_ref(&current), &[], &pattern)
            .expect("ok");
        assert!(out.is_empty());
    }

    #[test]
    fn registered_via_insights_v1() {
        let mut engine = KnowledgeEngine::new();
        crate::register_insights_v1(&mut engine).expect("register");
        assert!(engine.rule_count() >= 3);
    }
}
