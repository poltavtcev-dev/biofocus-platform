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
    out.push_str("## Features\n\n");
    if features.is_empty() {
        out.push_str("_No Features in this period._\n\n");
        return Ok(());
    }

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

    out.push_str("| Feature | Window (UTC s) | Value |\n");
    out.push_str("| :--- | :--- | :--- |\n");
    for feature in sorted {
        let window = format!(
            "{}–{}",
            feature.time_window.start.as_secs(),
            feature.time_window.end.as_secs()
        );
        let value = format_feature_value(&feature.value)?;
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_cell(&feature.feature_id),
            window,
            escape_cell(&value)
        ));
    }
    out.push('\n');
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
        "Write a short natural-language summary the user can skim. \
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

fn escape_cell(raw: &str) -> String {
    raw.replace('|', "\\|").replace('\n', " ")
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

        // Feature table order: FocusScore@0, FocusScore@900, StressIndex@100
        let focus_pos = a.markdown.find("| FocusScore | 0–900 |").expect("focus row");
        let focus2_pos = a
            .markdown
            .find("| FocusScore | 900–1800 |")
            .expect("focus2 row");
        let stress_pos = a
            .markdown
            .find("| StressIndex | 100–200 |")
            .expect("stress row");
        assert!(focus_pos < focus2_pos);
        assert!(focus2_pos < stress_pos);

        // Insight order by UUID
        let first = a.markdown.find("### First").expect("first insight");
        let second = a.markdown.find("### Second").expect("second insight");
        assert!(first < second);

        assert!(a.markdown.contains("| FocusScore | 0–900 | 72.0000 |"));
        assert!(a.llm_prompt.contains("interpret"));
        assert!(a.llm_prompt.contains("---\n# BioFocus report"));
    }

    #[test]
    fn features_only_minimal_insights_section() {
        let features = vec![feature("FatigueIndex", 0, 60, 40.0)];
        let doc = build_report(&features, &[]).expect("features-only");
        assert!(doc.markdown.contains("| FatigueIndex |"));
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
