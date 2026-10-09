//! Deterministic offline markdown + LLM prompt from Features and Insights.

use bio_spec::{EvidenceRef, Feature, FeatureValue, Insight};

use crate::error::{ReportEngineError, ReportResult};

/// Offline report document: human markdown plus an optional-LLM prompt string.
///
/// Both fields are produced without network I/O. The optional LLM adapter
/// ([`crate::interpret_report`]) may consume [`Self::llm_prompt`]; it must
/// **interpret** only — never compute Features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDocument {
    /// Calm markdown summary suitable for Dashboard / copy-paste.
    pub markdown: String,
    /// Prompt string wrapping the same facts for an optional local LLM.
    pub llm_prompt: String,
}

/// Builds a deterministic offline report from Features (+ Insights).
///
/// - Empty / minimal inputs → calm empty or minimal markdown (documented format).
/// - Non-empty inputs → stable output for the same Feature / Insight set
///   (sorted by id / window; fixed scalar formatting).
/// - No Feature math; values are rendered as already computed.
pub fn build_report(features: &[Feature], insights: &[Insight]) -> ReportResult<ReportDocument> {
    let markdown = render_markdown(features, insights)?;
    let llm_prompt = render_llm_prompt(&markdown);
    Ok(ReportDocument {
        markdown,
        llm_prompt,
    })
}

pub(crate) fn render_markdown(features: &[Feature], insights: &[Insight]) -> ReportResult<String> {
    let mut out = String::new();
    out.push_str("# BioFocus report\n\n");
    out.push_str(
        "_Offline summary from Features and Insights. Not a medical assessment._\n\n",
    );

    if features.is_empty() && insights.is_empty() {
        out.push_str("## Summary\n\n");
        out.push_str("Nothing to summarize for this period yet.\n");
        return Ok(out);
    }

    render_features_section(&mut out, features)?;
    render_insights_section(&mut out, insights);
    Ok(out)
}

pub(crate) fn render_features_section(
    out: &mut String,
    features: &[Feature],
) -> ReportResult<()> {
    out.push_str("## Расписание\n\n");
    if features.is_empty() {
        out.push_str("_No Features in this period._\n\n");
        return Ok(());
    }

    out.push_str(
        "_Минутные окна склеены в отрезки. Время местное. \
         Это не медицинская оценка._\n\n",
    );

    let offset = local_offset_secs();
    for (feature_id, samples) in group_features(features)? {
        out.push_str(&format!("### {}\n\n", feature_label(&feature_id)));
        out.push_str(&format!("_Код метрики: {feature_id}._\n\n"));
        for run in merge_runs(&samples) {
            let start = format_local(run.start, offset);
            let end = format_local(run.end, offset);
            out.push_str(&format!("- **{start} – {end}**"));
            if run.count > 1 {
                out.push_str(&format!(" ({} окон)", run.count));
            }
            out.push('\n');
            out.push_str(&format!(
                "  - В начале {}, в конце {}. Диапазон {}–{}.\n",
                run.first, run.last, run.low, run.high
            ));
            out.push_str(&format!("  - {}\n", trend_line(&run)));
        }
        out.push('\n');
    }
    Ok(())
}

pub(crate) fn render_insights_section(out: &mut String, insights: &[Insight]) {
    out.push_str("## Insights\n\n");
    if insights.is_empty() {
        out.push_str("_No Insights matched for this period._\n");
        return;
    }

    let mut sorted: Vec<&Insight> = insights.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    for insight in sorted {
        out.push_str(&format!("### {}\n\n", insight.title));
        out.push_str(&format!("{}\n\n", insight.description));
        out.push_str(&format!("- **Category:** {}\n", insight.category));
        out.push_str(&format!(
            "- **Evidence:** {}\n",
            format_evidence(&insight.evidence_list)
        ));
        if let Some(action) = &insight.action_recommendation {
            out.push_str(&format!("- **Suggestion:** {action}\n"));
        }
        out.push('\n');
    }
}

fn render_llm_prompt(markdown: &str) -> String {
    let mut out = String::new();
    out.push_str(
        "You are interpreting a BioFocus local wellness summary. \
         Use only the facts in the report below. Do not invent metrics, \
         diagnoses, or clinical claims. Keep a calm, non-evaluative tone. \
         Do not recompute Features — interpret the given values only.\n\n",
    );
    wrap_markdown_for_llm(&mut out, markdown);
    out.push_str(
        "Write the summary in Russian, in a few short paragraphs. \
         Use the clock times from the schedule. \
         Say which sources are present and which are missing. \
         Do not invent watch, phone, or computer data. \
         Prefer gentle observations over advice.\n",
    );
    out
}

/// Wraps report markdown between `---` fences for an LLM prompt body.
pub(crate) fn wrap_markdown_for_llm(out: &mut String, markdown: &str) {
    out.push_str("---\n");
    out.push_str(markdown);
    if !markdown.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n\n");
}

fn format_feature_value(value: &FeatureValue) -> ReportResult<String> {
    match value {
        FeatureValue::Scalar(n) => Ok(format_scalar(*n)),
        FeatureValue::Object(json) => serde_json::to_string(json).map_err(|err| {
            ReportEngineError::BuildFailed {
                message: format!("failed to render Feature object value: {err}"),
            }
        }),
    }
}

/// Fixed decimal formatting for deterministic markdown across platforms.
fn format_scalar(n: f64) -> String {
    if !n.is_finite() {
        return "n/a".into();
    }
    format!("{n:.4}")
}

pub(crate) fn format_evidence(list: &[EvidenceRef]) -> String {
    if list.is_empty() {
        return "_none_".into();
    }
    list.iter()
        .map(|e| match e {
            EvidenceRef::Feature(id) => format!("feature:{id}"),
            EvidenceRef::Signal(id) => format!("signal:{id}"),
            EvidenceRef::Insight(id) => format!("insight:{id}"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

struct FeatureSample {
    start: i64,
    end: i64,
    value: String,
    scalar: Option<f64>,
}

struct FeatureRun {
    start: i64,
    end: i64,
    count: usize,
    first: String,
    last: String,
    low: String,
    high: String,
    first_scalar: Option<f64>,
    last_scalar: Option<f64>,
}

fn group_features(features: &[Feature]) -> ReportResult<Vec<(String, Vec<FeatureSample>)>> {
    let mut sorted: Vec<&Feature> = features.iter().collect();
    sorted.sort_by(|a, b| {
        (
            a.feature_id.as_str(),
            a.time_window.start.as_secs(),
            a.time_window.end.as_secs(),
        )
            .cmp(&(
                b.feature_id.as_str(),
                b.time_window.start.as_secs(),
                b.time_window.end.as_secs(),
            ))
    });

    let mut groups: Vec<(String, Vec<FeatureSample>)> = Vec::new();
    for feature in sorted {
        let sample = FeatureSample {
            start: feature.time_window.start.as_secs(),
            end: feature.time_window.end.as_secs(),
            value: format_feature_value(&feature.value)?,
            scalar: match &feature.value {
                FeatureValue::Scalar(n) if n.is_finite() => Some(*n),
                _ => None,
            },
        };
        if let Some((id, samples)) = groups.last_mut() {
            if id == &feature.feature_id {
                samples.push(sample);
                continue;
            }
        }
        groups.push((feature.feature_id.clone(), vec![sample]));
    }
    Ok(groups)
}

fn merge_runs(samples: &[FeatureSample]) -> Vec<FeatureRun> {
    let mut runs: Vec<FeatureRun> = Vec::new();
    for sample in samples {
        if let Some(run) = runs.last_mut() {
            if sample.start <= run.end.saturating_add(120) {
                run.end = run.end.max(sample.end);
                run.count += 1;
                run.last = sample.value.clone();
                run.last_scalar = sample.scalar;
                if let Some(n) = sample.scalar {
                    let low = run.low.parse::<f64>().unwrap_or(n);
                    let high = run.high.parse::<f64>().unwrap_or(n);
                    if n < low {
                        run.low = format_scalar(n);
                    }
                    if n > high {
                        run.high = format_scalar(n);
                    }
                }
                continue;
            }
        }
        runs.push(FeatureRun {
            start: sample.start,
            end: sample.end,
            count: 1,
            first: sample.value.clone(),
            last: sample.value.clone(),
            low: sample.value.clone(),
            high: sample.value.clone(),
            first_scalar: sample.scalar,
            last_scalar: sample.scalar,
        });
    }
    runs
}

fn trend_line(run: &FeatureRun) -> String {
    match (run.first_scalar, run.last_scalar) {
        (Some(start), Some(end)) if end < start - 5.0 => {
            "К концу отрезка значение заметно ниже, чем в начале.".into()
        }
        (Some(start), Some(end)) if end > start + 5.0 => {
            "К концу отрезка значение заметно выше, чем в начале.".into()
        }
        (Some(_), Some(_)) => "За отрезок значение почти не сдвинулось.".into(),
        _ => "Числовое сравнение начала и конца недоступно.".into(),
    }
}

fn feature_label(id: &str) -> String {
    match id {
        "AttentionStability" => "Устойчивость фокуса",
        "FocusScore" => "Фокус",
        "DeepWorkScore" => "Длинный фокус",
        "CognitiveLoad" => "Суммарная нагрузка",
        "SustainedLoadIndicator" => "Длительная нагрузка",
        "StressIndex" => "Напряжение",
        "FatigueIndex" => "Усталость",
        "RecoveryScore" => "Восстановление",
        "ContextSwitchRate" => "Переключения приложений",
        "MeetingDensity" => "Плотность встреч",
        "NotificationPressure" => "Давление уведомлений",
        "DistractionScore" => "Фрагментация внимания",
        "SleepDebt" => "Недосып",
        "EnergyScore" => "Энергия",
        "ActivityBalance" => "Баланс активности",
        "CircadianOffset" => "Совпадение графика",
        "DeskAwayPresence" => "Вне стола",
        other => other,
    }
    .to_owned()
}

/// Local clock `7 окт 2026, 17:40` for a Unix UTC second.
pub fn format_unix_local(unix: i64, offset_secs: i32) -> String {
    format_local(unix, offset_secs)
}

fn format_local(unix: i64, offset_secs: i32) -> String {
    let local = unix.saturating_add(i64::from(offset_secs));
    let days = local.div_euclid(86_400);
    let sod = local.rem_euclid(86_400);
    let hour = sod / 3_600;
    let minute = (sod % 3_600) / 60;
    let (year, month, day) = civil_from_days(days);
    let months = [
        "янв", "фев", "мар", "апр", "мая", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
    ];
    let month_name = months
        .get((month as usize).saturating_sub(1))
        .copied()
        .unwrap_or("?");
    format!("{day} {month_name} {year}, {hour:02}:{minute:02}")
}

/// Howard Hinnant civil_from_days. `days` is days since 1970-01-01.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year as i32, month as u32, day as u32)
}

fn local_offset_secs() -> i32 {
    chrono::Local::now().offset().local_minus_utc()
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        Confidence, EvidenceRef, Feature, FeatureValue, Insight, TimeWindow, UnixTimestamp,
    };
    use uuid::Uuid;

    use super::*;

    fn window(start: i64, end: i64) -> TimeWindow {
        TimeWindow::try_new(
            UnixTimestamp::from_secs(start),
            UnixTimestamp::from_secs(end),
        )
        .expect("valid test window")
    }

    fn feature(id: &str, start: i64, end: i64, value: f64) -> Feature {
        Feature {
            feature_id: id.into(),
            time_window: window(start, end),
            value: FeatureValue::Scalar(value),
            provenance: Vec::new(),
            confidence: Confidence::ONE,
            factors: Vec::new(),
        }
    }

    fn insight(id: Uuid, title: &str) -> Insight {
        Insight {
            id,
            title: title.into(),
            description: "Calm description.".into(),
            category: "focus".into(),
            evidence_list: vec![EvidenceRef::Feature("FocusScore".into())],
            action_recommendation: Some("A short pause may help when it fits.".into()),
        }
    }

    #[test]
    fn empty_inputs_yield_calm_minimal_report() {
        let doc = build_report(&[], &[]).expect("empty report builds");
        assert!(doc.markdown.contains("# BioFocus report"));
        assert!(doc.markdown.contains("Nothing to summarize"));
        assert!(!doc.markdown.contains("| Feature |"));
        assert!(doc.llm_prompt.contains(&doc.markdown));
        assert!(doc.llm_prompt.contains("Do not invent metrics"));
    }

    #[test]
    fn non_empty_inputs_are_stable_across_calls() {
        let id_a = Uuid::parse_str("01900000-0000-7000-8000-000000000001").expect("uuid");
        let id_b = Uuid::parse_str("01900000-0000-7000-8000-000000000002").expect("uuid");

        // Intentionally unsorted inputs — builder must order for stability.
        let features = vec![
            feature("StressIndex", 100, 200, 61.5),
            feature("FocusScore", 0, 900, 72.0),
            feature("FocusScore", 900, 1800, 68.25),
        ];
        let insights = vec![
            insight(id_b, "Second"),
            insight(id_a, "First"),
        ];

        let a = build_report(&features, &insights).expect("build a");
        let b = build_report(&features, &insights).expect("build b");
        assert_eq!(a, b);

        let focus_pos = a.markdown.find("### Фокус").expect("focus heading");
        let stress_pos = a.markdown.find("### Напряжение").expect("stress heading");
        assert!(focus_pos < stress_pos);
        assert!(a.markdown.contains("72.0000"));
        assert!(a.markdown.contains("68.2500"));
        assert!(a.markdown.contains("61.5000"));
        assert!(a.markdown.contains("2 окон"));

        // Insight order by UUID
        let first = a.markdown.find("### First").expect("first insight");
        let second = a.markdown.find("### Second").expect("second insight");
        assert!(first < second);
        assert!(a.llm_prompt.contains("interpret"));
        assert!(a.llm_prompt.contains("---\n# BioFocus report"));
    }

    #[test]
    fn features_only_minimal_insights_section() {
        let features = vec![feature("FatigueIndex", 0, 60, 40.0)];
        let doc = build_report(&features, &[]).expect("features-only");
        assert!(doc.markdown.contains("### Усталость"));
        assert!(doc.markdown.contains("_No Insights matched"));
    }

    #[test]
    fn insights_only_minimal_features_section() {
        let id = Uuid::parse_str("01900000-0000-7000-8000-000000000099").expect("uuid");
        let insights = vec![insight(id, "Solo")];
        let doc = build_report(&[], &insights).expect("insights-only");
        assert!(doc.markdown.contains("_No Features in this period._"));
        assert!(doc.markdown.contains("### Solo"));
    }
}
