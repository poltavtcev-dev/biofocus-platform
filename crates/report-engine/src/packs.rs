//! Named/versioned in-process prompt packs (ADR-011 / P11-E2).
//!
//! Packs format already-computed Features / Insights / Recommendations into
//! offline [`ReportDocument`] output. They do **not** compute Features,
//! Recommendations, or Evidence, and they open neither SQLite nor network.

use bio_spec::{Feature, Insight, Recommendation};

use crate::builder::{
    format_evidence, render_features_section, render_insights_section, wrap_markdown_for_llm,
    ReportDocument,
};
use crate::error::{ReportEngineError, ReportResult};

/// Default calm coaching pack id (`biofocus.default`).
pub const DEFAULT_PROMPT_PACK_ID: &str = "biofocus.default";

/// Default pack version string (`1`).
pub const DEFAULT_PROMPT_PACK_VERSION: &str = "1";

/// Identity of a registered in-process prompt pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptPackRef {
    /// Stable pack id (e.g. [`DEFAULT_PROMPT_PACK_ID`]).
    pub id: &'static str,
    /// Pack version (e.g. [`DEFAULT_PROMPT_PACK_VERSION`]).
    pub version: &'static str,
}

const DEFAULT_PACK: PromptPackRef = PromptPackRef {
    id: DEFAULT_PROMPT_PACK_ID,
    version: DEFAULT_PROMPT_PACK_VERSION,
};

/// In-process registry of shipped packs (v1: default only).
pub fn list_prompt_packs() -> &'static [PromptPackRef] {
    &[DEFAULT_PACK]
}

/// Returns the default pack identity (`biofocus.default` @ `1`).
pub fn default_prompt_pack() -> PromptPackRef {
    DEFAULT_PACK
}

/// Builds an offline report using a named/versioned prompt pack.
///
/// - Unknown id/version → [`ReportEngineError::UnknownPromptPack`].
/// - Empty / partial Evidence → soft empty sections, still `Ok`.
/// - No Feature / Recommendation math; values are rendered as given.
/// - No network and no SQLite.
pub fn build_report_with_pack(
    pack_id: &str,
    version: &str,
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
) -> ReportResult<ReportDocument> {
    match (pack_id, version) {
        (DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION) => {
            build_default_pack(features, insights, recommendations)
        }
        _ => Err(ReportEngineError::UnknownPromptPack {
            id: pack_id.to_owned(),
            version: version.to_owned(),
        }),
    }
}

fn build_default_pack(
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
) -> ReportResult<ReportDocument> {
    let markdown = render_default_markdown(features, insights, recommendations)?;
    let llm_prompt = render_default_llm_prompt(&markdown);
    Ok(ReportDocument {
        markdown,
        llm_prompt,
    })
}

fn render_default_markdown(
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
) -> ReportResult<String> {
    let mut out = String::new();
    out.push_str("# BioFocus report\n\n");
    out.push_str(
        "_Offline summary from Features, Insights, and Recommendations. \
         Not a medical assessment._\n\n",
    );

    if features.is_empty() && insights.is_empty() && recommendations.is_empty() {
        out.push_str("## Summary\n\n");
        out.push_str("Nothing to summarize for this period yet.\n");
        return Ok(out);
    }

    render_features_section(&mut out, features)?;
    render_insights_section(&mut out, insights);
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    render_recommendations_section(&mut out, recommendations);
    Ok(out)
}

fn render_recommendations_section(out: &mut String, recommendations: &[Recommendation]) {
    out.push_str("## Recommendations\n\n");
    if recommendations.is_empty() {
        out.push_str("_No Recommendations for this period._\n");
        return;
    }

    let mut sorted: Vec<&Recommendation> = recommendations.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    for rec in sorted {
        out.push_str(&format!("### {}\n\n", rec.title));
        out.push_str(&format!("{}\n\n", rec.suggestion));
        out.push_str(&format!("- **Category:** {}\n", rec.category));
        out.push_str(&format!(
            "- **Evidence:** {}\n",
            format_evidence(&rec.evidence_list)
        ));
        out.push('\n');
    }
}

fn render_default_llm_prompt(markdown: &str) -> String {
    let mut out = String::new();
    out.push_str(
        "You are interpreting a BioFocus local wellness summary. \
         Use only the facts in the report below. \
         Do not invent metrics, Evidence, Insights, Recommendations, or actions. \
         Do not invent diagnoses or clinical claims. \
         Keep a calm, non-evaluative tone. \
         Do not recompute Features — interpret the given values only. \
         Do not invent or rewrite Recommendations as new advice.\n\n",
    );
    wrap_markdown_for_llm(&mut out, markdown);
    out.push_str(
        "Write a short natural-language summary the user can skim. \
         Prefer gentle observations over advice. \
         This is optional personal interpretation, not medical advice.\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        Confidence, EvidenceRef, Feature, FeatureValue, Insight, Recommendation, TimeWindow,
        UnixTimestamp,
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
            action_recommendation: None,
        }
    }

    fn recommendation(id: Uuid, title: &str) -> Recommendation {
        Recommendation {
            id,
            title: title.into(),
            suggestion: "If it fits, a short pause may help.".into(),
            category: "pace".into(),
            evidence_list: vec![EvidenceRef::Feature("FocusScore".into())],
        }
    }

    #[test]
    fn default_pack_is_registered() {
        let packs = list_prompt_packs();
        assert!(packs.iter().any(|p| {
            p.id == DEFAULT_PROMPT_PACK_ID && p.version == DEFAULT_PROMPT_PACK_VERSION
        }));
        let d = default_prompt_pack();
        assert_eq!(d.id, DEFAULT_PROMPT_PACK_ID);
        assert_eq!(d.version, DEFAULT_PROMPT_PACK_VERSION);
    }

    #[test]
    fn unknown_pack_returns_typed_error() {
        let err = build_report_with_pack("missing.pack", "9", &[], &[], &[])
            .expect_err("unknown pack");
        assert_eq!(
            err,
            ReportEngineError::UnknownPromptPack {
                id: "missing.pack".into(),
                version: "9".into(),
            }
        );
    }

    #[test]
    fn wrong_version_of_default_is_unknown() {
        let err = build_report_with_pack(DEFAULT_PROMPT_PACK_ID, "999", &[], &[], &[])
            .expect_err("wrong version");
        assert!(matches!(err, ReportEngineError::UnknownPromptPack { .. }));
    }

    #[test]
    fn empty_inputs_soft_ok_with_forbid_instructions() {
        let doc = build_report_with_pack(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[],
            &[],
            &[],
        )
        .expect("empty pack build");
        assert!(doc.markdown.contains("Nothing to summarize"));
        assert!(!doc.markdown.contains("| Feature |"));
        assert!(doc.llm_prompt.contains("Do not invent metrics, Evidence, Insights, Recommendations, or actions"));
        assert!(doc.llm_prompt.contains(&doc.markdown));
    }

    #[test]
    fn partial_evidence_soft_empty_sections() {
        let features = vec![feature("FocusScore", 0, 900, 70.0)];
        let doc = build_report_with_pack(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &features,
            &[],
            &[],
        )
        .expect("partial build");
        assert!(doc.markdown.contains("| FocusScore |"));
        assert!(doc.markdown.contains("_No Insights matched"));
        assert!(doc.markdown.contains("_No Recommendations for this period._"));
        assert!(doc.llm_prompt.contains("Do not invent"));
    }

    #[test]
    fn recommendations_only_soft_feature_insight_sections() {
        let id = Uuid::parse_str("01900000-0000-7000-8000-0000000000aa").expect("uuid");
        let recs = vec![recommendation(id, "Gentler pace")];
        let doc = build_report_with_pack(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[],
            &[],
            &recs,
        )
        .expect("recs-only");
        assert!(doc.markdown.contains("_No Features in this period._"));
        assert!(doc.markdown.contains("_No Insights matched"));
        assert!(doc.markdown.contains("### Gentler pace"));
        assert!(doc.markdown.contains("If it fits, a short pause may help."));
    }

    #[test]
    fn full_inputs_stable_and_sorted() {
        let i1 = Uuid::parse_str("01900000-0000-7000-8000-000000000001").expect("uuid");
        let i2 = Uuid::parse_str("01900000-0000-7000-8000-000000000002").expect("uuid");
        let r1 = Uuid::parse_str("01900000-0000-7000-8000-000000000011").expect("uuid");
        let r2 = Uuid::parse_str("01900000-0000-7000-8000-000000000012").expect("uuid");

        let features = vec![
            feature("StressIndex", 100, 200, 50.0),
            feature("FocusScore", 0, 900, 72.0),
        ];
        let insights = vec![insight(i2, "Second"), insight(i1, "First")];
        let recs = vec![
            recommendation(r2, "Rec B"),
            recommendation(r1, "Rec A"),
        ];

        let a = build_report_with_pack(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &features,
            &insights,
            &recs,
        )
        .expect("build a");
        let b = build_report_with_pack(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &features,
            &insights,
            &recs,
        )
        .expect("build b");
        assert_eq!(a, b);

        let focus = a.markdown.find("| FocusScore |").expect("focus");
        let stress = a.markdown.find("| StressIndex |").expect("stress");
        assert!(focus < stress);

        let first = a.markdown.find("### First").expect("first");
        let second = a.markdown.find("### Second").expect("second");
        assert!(first < second);

        let rec_a = a.markdown.find("### Rec A").expect("rec a");
        let rec_b = a.markdown.find("### Rec B").expect("rec b");
        assert!(rec_a < rec_b);

        assert!(a.llm_prompt.contains("Do not invent metrics, Evidence, Insights, Recommendations, or actions"));
        assert!(a.llm_prompt.contains("not medical advice"));
    }
}
