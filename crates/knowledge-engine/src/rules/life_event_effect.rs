//! Rule: before/after comparison around user-logged Life Events (coffee, walk).
//!
//! Non-clinical, descriptive only: "Focus was N points higher in the hour
//! after your walks than in the 45 minutes before". No causal claim.
//!
//! Inputs come from [`PatternInputs::life_events`] (retracted events already
//! removed by Core) and [`PatternInputs::recent_series`] (stepped FocusScore /
//! CognitiveLoad series). A Feature sample belongs to a window by its
//! `time_window.end`:
//! - **before**: `(t − 45 min, t]`
//! - **after**: `[t + 15 min, t + 60 min]` (skips the first 15 min so the
//!   15-min trailing Feature window no longer overlaps pre-event data).
//!
//! Confidence: the [`Insight`] contract has no numeric field, so the
//! description ends with a calm label (low / medium / high) derived from the
//! number of qualifying events × mean Feature confidence.

use bio_spec::{EvidenceRef, Feature, Insight, Signal};
use uuid::Uuid;

use crate::pattern::{LifeEventMark, PatternInputs};
use crate::rules::{scalar_value, COGNITIVE_LOAD_ID, FOCUS_SCORE_ID};
use crate::{InsightRule, KnowledgeEngineResult};

/// Stable rule id for [`LifeEventBeforeAfterRule`].
pub const RULE_LIFE_EVENT_BEFORE_AFTER: &str = "life_event_before_after_v1";
/// Insight category for Life Event comparisons.
pub const LIFE_EVENT_CATEGORY: &str = "life_event";
/// Kinds compared by this rule.
pub const LIFE_EVENT_EFFECT_KINDS: &[&str] = &["coffee", "walk"];
/// Pre-event window length (seconds).
pub const LIFE_EVENT_BEFORE_SECS: i64 = 45 * 60;
/// Post-event window start offset (seconds).
pub const LIFE_EVENT_AFTER_START_SECS: i64 = 15 * 60;
/// Post-event window end offset (seconds).
pub const LIFE_EVENT_AFTER_END_SECS: i64 = 60 * 60;
/// Minimum samples per side for one event to count.
pub const LIFE_EVENT_MIN_SAMPLES: usize = 2;
/// Minimum |mean delta| (score points) to surface an Insight.
pub const LIFE_EVENT_DELTA: f64 = 5.0;
/// Samples below this Feature confidence are ignored.
pub const LIFE_EVENT_CONFIDENCE_GATE: f64 = 0.4;
/// Max event ids cited as Evidence.
const MAX_EVENT_EVIDENCE: usize = 5;

/// See module docs.
#[derive(Debug, Default, Clone, Copy)]
pub struct LifeEventBeforeAfterRule;

/// Per-metric aggregate across qualifying events.
#[derive(Debug, Clone, PartialEq)]
struct MetricComparison {
    feature_id: &'static str,
    events: usize,
    mean_before: f64,
    mean_after: f64,
    mean_confidence: f64,
}

impl MetricComparison {
    fn delta(&self) -> f64 {
        self.mean_after - self.mean_before
    }
}

fn window_mean(series: &[Feature], feature_id: &str, lo_excl: i64, hi_incl: i64) -> Option<(f64, f64, usize)> {
    let mut sum = 0.0;
    let mut conf = 0.0;
    let mut n = 0usize;
    for f in series.iter().filter(|f| f.feature_id == feature_id) {
        let end = f.time_window.end.as_secs();
        if end <= lo_excl || end > hi_incl || f.confidence.get() < LIFE_EVENT_CONFIDENCE_GATE {
            continue;
        }
        if let Some(v) = scalar_value(f) {
            sum += v;
            conf += f.confidence.get();
            n += 1;
        }
    }
    (n >= LIFE_EVENT_MIN_SAMPLES).then(|| (sum / n as f64, conf / n as f64, n))
}

/// Compare `feature_id` before vs after each event; `None` when no event qualifies.
#[must_use]
fn compare_around(
    events: &[&LifeEventMark],
    series: &[Feature],
    feature_id: &'static str,
) -> Option<(MetricComparison, Vec<Uuid>)> {
    let mut before = 0.0;
    let mut after = 0.0;
    let mut conf = 0.0;
    let mut ids = Vec::new();
    for ev in events {
        let t = ev.timestamp.as_secs();
        let Some((b, bc, _)) = window_mean(series, feature_id, t - LIFE_EVENT_BEFORE_SECS, t) else {
            continue;
        };
        let Some((a, ac, _)) = window_mean(
            series,
            feature_id,
            t + LIFE_EVENT_AFTER_START_SECS - 1,
            t + LIFE_EVENT_AFTER_END_SECS,
        ) else {
            continue;
        };
        before += b;
        after += a;
        conf += (bc + ac) / 2.0;
        ids.push(ev.id);
    }
    let n = ids.len();
    (n > 0).then(|| {
        (
            MetricComparison {
                feature_id,
                events: n,
                mean_before: before / n as f64,
                mean_after: after / n as f64,
                mean_confidence: conf / n as f64,
            },
            ids,
        )
    })
}

/// Calm confidence label from event count and data quality.
#[must_use]
fn confidence_label(events: usize, mean_confidence: f64) -> &'static str {
    let score = (events as f64 / 4.0).min(1.0) * mean_confidence.clamp(0.0, 1.0);
        if score >= 0.75 {
        "высокая"
    } else if score >= 0.4 {
        "средняя"
    } else {
        "низкая"
    }
}

fn kind_noun(kind: &str, n: usize) -> &'static str {
    match (kind, n == 1) {
        ("coffee", true) => "кофе",
        ("coffee", false) => "кофе",
        ("walk", true) => "прогулка",
        (_, false) => "прогулки",
        _ => "событие",
    }
}

fn direction(delta: f64) -> &'static str {
    if delta >= 0.0 {
        "выше"
    } else {
        "ниже"
    }
}

impl InsightRule for LifeEventBeforeAfterRule {
    fn id(&self) -> &str {
        RULE_LIFE_EVENT_BEFORE_AFTER
    }

    fn evaluate(
        &self,
        _features: &[Feature],
        _signals: &[Signal],
        pattern: &PatternInputs,
    ) -> KnowledgeEngineResult<Vec<Insight>> {
        let mut out = Vec::new();
        for kind in LIFE_EVENT_EFFECT_KINDS {
            let events: Vec<&LifeEventMark> =
                pattern.life_events.iter().filter(|e| e.kind == *kind).collect();
            if events.is_empty() {
                continue;
            }
            let focus = compare_around(&events, &pattern.recent_series, FOCUS_SCORE_ID)
                .filter(|(m, _)| m.delta().abs() >= LIFE_EVENT_DELTA);
            let load = compare_around(&events, &pattern.recent_series, COGNITIVE_LOAD_ID)
                .filter(|(m, _)| m.delta().abs() >= LIFE_EVENT_DELTA);
            if focus.is_none() && load.is_none() {
                continue;
            }

            let mut parts = Vec::new();
            let mut evidence = Vec::new();
            let mut ids: Vec<Uuid> = Vec::new();
            let mut events_n = 0usize;
            let mut conf_sum = 0.0;
            let mut conf_n = 0.0;
            for (label, cmp) in [("Фокус", &focus), ("общая нагрузка", &load)] {
                if let Some((m, used)) = cmp {
                    parts.push(format!(
                        "{label} в среднем {:.0} после против {:.0} до ({} на {:.0})",
                        m.mean_after,
                        m.mean_before,
                        direction(m.delta()),
                        m.delta().abs()
                    ));
                    evidence.push(EvidenceRef::Feature(m.feature_id.into()));
                    events_n = events_n.max(m.events);
                    conf_sum += m.mean_confidence;
                    conf_n += 1.0;
                    for id in used {
                        if !ids.contains(id) {
                            ids.push(*id);
                        }
                    }
                }
            }
            ids.sort();
            for id in ids.iter().rev().take(MAX_EVENT_EVIDENCE) {
                evidence.push(EvidenceRef::Observation(*id));
            }
            let confidence = confidence_label(events_n, conf_sum / conf_n);
            let noun = kind_noun(kind, events_n);
            let title = match *kind {
                "coffee" => "Вокруг записей про кофе",
                _ => "Вокруг прогулок",
            };
            out.push(Insight {
                id: Uuid::now_v7(),
                title: title.into(),
                description: format!(
                    "По {events_n} недавним записям ({noun}): {}. Сравнение: 45 мин до и 15–60 мин после. Личная закономерность, не правило. Уверенность: {confidence}.",
                    parts.join("; ")
                ),
                category: LIFE_EVENT_CATEGORY.into(),
                evidence_list: evidence,
                action_recommendation: None,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{Confidence, FeatureValue, TimeWindow, UnixTimestamp};

    use super::*;
    use crate::KnowledgeEngine;

    const T: i64 = 1_800_000_000;

    fn f(id: &str, end: i64, value: f64) -> Feature {
        Feature {
            feature_id: id.into(),
            time_window: TimeWindow::try_new(
                UnixTimestamp::from_secs(end - 900),
                UnixTimestamp::from_secs(end),
            )
            .expect("window"),
            value: FeatureValue::Scalar(value),
            provenance: vec![],
            confidence: Confidence::try_new(0.9).expect("c"),
            factors: Vec::new(),
        }
    }

    /// 5-min stepped series: `before` value until `t`, `after` from t+5min.
    fn stepped(id: &str, t: i64, before: f64, after: f64) -> Vec<Feature> {
        (-12..=12)
            .map(|k| {
                let end = t + k * 300;
                f(id, end, if end <= t { before } else { after })
            })
            .collect()
    }

    fn mark(id: u128, kind: &str, ts: i64) -> LifeEventMark {
        LifeEventMark {
            id: Uuid::from_u128(id),
            kind: kind.into(),
            timestamp: UnixTimestamp::from_secs(ts),
        }
    }

    fn eval(pattern: &PatternInputs) -> Vec<Insight> {
        let mut engine = KnowledgeEngine::new();
        engine.register(LifeEventBeforeAfterRule).expect("register");
        engine.evaluate_with_pattern(&[], &[], pattern).expect("eval")
    }

    #[test]
    fn walk_with_higher_focus_after_emits_insight_with_evidence() {
        let mut series = stepped(FOCUS_SCORE_ID, T, 55.0, 68.0);
        series.extend(stepped(COGNITIVE_LOAD_ID, T, 60.0, 61.0));
        let pattern = PatternInputs::empty().with_life_events(vec![mark(7, "walk", T)], series);
        let out = eval(&pattern);
        assert_eq!(out.len(), 1);
        let insight = &out[0];
        assert_eq!(insight.category, LIFE_EVENT_CATEGORY);
        assert_eq!(insight.title, "Вокруг прогулок");
        assert!(insight.description.contains("Фокус в среднем 68 после против 55 до (выше на 13)"), "{}", insight.description);
        assert!(!insight.description.contains("общая нагрузка"), "load delta 1 < threshold");
        assert!(insight.description.contains("Уверенность: низкая"), "single event → low");
        assert!(insight.evidence_list.contains(&EvidenceRef::Feature(FOCUS_SCORE_ID.into())));
        assert!(insight.evidence_list.contains(&EvidenceRef::Observation(Uuid::from_u128(7))));
    }

    #[test]
    fn coffee_lower_demand_mentions_combined_demand() {
        let series = stepped(COGNITIVE_LOAD_ID, T, 70.0, 58.0);
        let pattern = PatternInputs::empty().with_life_events(vec![mark(1, "coffee", T)], series);
        let out = eval(&pattern);
        assert_eq!(out.len(), 1);
        assert!(out[0].description.contains("общая нагрузка в среднем 58 после против 70 до (ниже на 12)"));
        assert!(out[0].evidence_list.contains(&EvidenceRef::Feature(COGNITIVE_LOAD_ID.into())));
    }

    #[test]
    fn small_delta_or_missing_data_is_silent() {
        let series = stepped(FOCUS_SCORE_ID, T, 60.0, 62.0);
        let pattern = PatternInputs::empty().with_life_events(vec![mark(1, "walk", T)], series);
        assert!(eval(&pattern).is_empty(), "delta 2 < 5");

        // Only "before" data (event too recent for an after window).
        let series: Vec<Feature> = stepped(FOCUS_SCORE_ID, T, 40.0, 80.0)
            .into_iter()
            .filter(|f| f.time_window.end.as_secs() <= T)
            .collect();
        let pattern = PatternInputs::empty().with_life_events(vec![mark(1, "walk", T)], series);
        assert!(eval(&pattern).is_empty());
    }

    #[test]
    fn lunch_and_workout_are_not_compared() {
        let series = stepped(FOCUS_SCORE_ID, T, 40.0, 80.0);
        let pattern = PatternInputs::empty()
            .with_life_events(vec![mark(1, "lunch", T), mark(2, "workout", T)], series);
        assert!(eval(&pattern).is_empty());
    }

    #[test]
    fn retracted_event_absent_from_inputs_gives_no_insight() {
        // Core removes retracted events before building PatternInputs; with no
        // remaining marks the rule must stay silent even if the series moved.
        let series = stepped(FOCUS_SCORE_ID, T, 40.0, 80.0);
        let pattern = PatternInputs::empty().with_life_events(Vec::new(), series);
        assert!(eval(&pattern).is_empty());
    }

    #[test]
    fn several_events_raise_confidence() {
        let mut series = Vec::new();
        let mut marks = Vec::new();
        for i in 0..4 {
            let t = T + i * 3 * 3600;
            series.extend(stepped(FOCUS_SCORE_ID, t, 50.0, 60.0));
            marks.push(mark(i as u128 + 1, "coffee", t));
        }
        let out = eval(&PatternInputs::empty().with_life_events(marks, series));
        assert_eq!(out.len(), 1);
        assert!(out[0].description.starts_with("По 4 недавним записям (кофе)"));
        assert!(out[0].description.contains("Уверенность: высокая"), "{}", out[0].description);
        let obs_refs = out[0]
            .evidence_list
            .iter()
            .filter(|e| matches!(e, EvidenceRef::Observation(_)))
            .count();
        assert_eq!(obs_refs, 4);
    }

    #[test]
    fn confidence_label_thresholds() {
        assert_eq!(confidence_label(1, 1.0), "низкая");
        assert_eq!(confidence_label(2, 0.9), "средняя");
        assert_eq!(confidence_label(4, 0.8), "высокая");
        assert_eq!(confidence_label(4, 0.5), "средняя");
    }
}
