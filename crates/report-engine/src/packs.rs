//! Named/versioned in-process prompt packs (ADR-011 / P11-E2).
//!
//! Packs format already-computed Features / Insights / Recommendations into
//! offline [`ReportDocument`] output. They do **not** compute Features,
//! Recommendations, or Evidence, and they open neither SQLite nor network.
//!
//! Optional ADR-024 health context (P23-E2) may be injected as **user-declared**
//! interpret-only framing — never as diagnoses or Feature math.

use bio_spec::{Feature, Insight, Recommendation};

use crate::builder::{
    format_evidence, render_features_section, render_insights_section, wrap_markdown_for_llm,
    ReportDocument,
};
use crate::error::{ReportEngineError, ReportResult};
use crate::health_context::{load_health_context, HealthContext};

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
/// Loads opt-in `~/.biofocus/health-context.toml` when present (ADR-024) and
/// injects it as user-declared context. Prefer
/// [`build_report_with_pack_and_health`] in tests to pass an explicit context.
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
    build_report_with_pack_and_health(
        pack_id,
        version,
        features,
        insights,
        recommendations,
        load_health_context().as_ref(),
    )
}

/// Builds a pack report with an explicit health-context override (tests / hosts).
///
/// `health = None` → no health section (same as missing/empty config file).
pub fn build_report_with_pack_and_health(
    pack_id: &str,
    version: &str,
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
    health: Option<&HealthContext>,
) -> ReportResult<ReportDocument> {
    match (pack_id, version) {
        (DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION) => {
            build_default_pack(features, insights, recommendations, health)
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
    health: Option<&HealthContext>,
) -> ReportResult<ReportDocument> {
    let markdown = render_default_markdown(features, insights, recommendations, health)?;
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
    health: Option<&HealthContext>,
) -> ReportResult<String> {
    let mut out = String::new();
    out.push_str("# BioFocus report\n\n");
    out.push_str(
        "_Offline summary from Features, Insights, and Recommendations. \
         Not a medical assessment._\n\n",
    );

    render_health_context_section(&mut out, health);

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

fn render_health_context_section(out: &mut String, health: Option<&HealthContext>) {
    let Some(ctx) = health.filter(|c| !c.is_empty()) else {
        return;
    };
    out.push_str("## User-declared context\n\n");
    out.push_str(
        "_Optional personal notes the user already knows. \
         Not a diagnosis. Not inferred from biometrics._\n\n",
    );
    if !ctx.conditions.is_empty() {
        out.push_str("- **Declared labels:** ");
        out.push_str(&ctx.conditions.join(", "));
        out.push('\n');
    }
    if !ctx.note.is_empty() {
        out.push_str("- **Note:** ");
        out.push_str(&ctx.note);
        out.push('\n');
    }
    out.push('\n');
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
         If a \"User-declared context\" section is present, treat it as \
         optional framing the user already knows — do not diagnose from it \
         or invent additional conditions. \
         Keep a calm, non-evaluative tone. \
         Do not recompute Features — interpret the given values only. \
         Do not invent or rewrite Recommendations as new advice.\n\n",
    );
    wrap_markdown_for_llm(&mut out, markdown);
    out.push_str(
        "Write the summary in Russian, in a few short paragraphs. \
         Use the clock times from the schedule. \
         Say which sources are present and which are missing. \
         Do not invent watch, phone, or computer data. \
         Prefer gentle observations over advice. \
         This is optional personal interpretation, not medical advice.\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use bio_spec::{Feature, FeatureValue, TimeWindow, UnixTimestamp};
    use uuid::Uuid;

    use super::*;
    use crate::health_context::HealthContext;

    fn tw() -> TimeWindow {
        TimeWindow {
            start: UnixTimestamp::from_secs(0),
            end: UnixTimestamp::from_secs(900),
        }
    }

    fn sample_feature() -> Feature {
        Feature {
            feature_id: "FocusScore".into(),
            time_window: tw(),
            value: FeatureValue::Scalar(72.0),
            provenance: vec![Uuid::nil()],
            confidence: bio_spec::Confidence::saturating_from(1.0),
            factors: vec![],
        }
    }

    #[test]
    fn unknown_pack_errors() {
        let err = build_report_with_pack_and_health("missing.pack", "9", &[], &[], &[], None)
            .expect_err("unknown");
        assert!(matches!(
            err,
            ReportEngineError::UnknownPromptPack { .. }
        ));
    }

    #[test]
    fn health_none_keeps_prior_empty_shape() {
        let doc = build_report_with_pack_and_health(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[],
            &[],
            &[],
            None,
        )
        .expect("ok");
        assert!(doc.markdown.contains("Nothing to summarize"));
        assert!(!doc.markdown.contains("User-declared context"));
        assert!(doc.llm_prompt.contains("Do not invent diagnoses"));
    }

    #[test]
    fn health_context_injected_into_markdown_and_prompt() {
        let health = HealthContext {
            conditions: vec!["sleep_sensitive".into()],
            note: "I already know late nights hit me".into(),
        };
        let doc = build_report_with_pack_and_health(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[sample_feature()],
            &[],
            &[],
            Some(&health),
        )
        .expect("ok");
        assert!(doc.markdown.contains("User-declared context"));
        assert!(doc.markdown.contains("sleep_sensitive"));
        assert!(doc.markdown.contains("I already know late nights hit me"));
        assert!(doc.markdown.contains("Not a diagnosis"));
        assert!(doc.llm_prompt.contains("User-declared context"));
        assert!(doc.llm_prompt.contains("do not diagnose"));
        assert!(!doc.markdown.to_lowercase().contains("you have"));
    }

    #[test]
    fn does_not_branch_on_health_for_feature_values() {
        let health = HealthContext {
            conditions: vec!["migraine_prone".into()],
            note: String::new(),
        };
        let features = [sample_feature()];
        let with_h = build_report_with_pack_and_health(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &features,
            &[],
            &[],
            Some(&health),
        )
        .expect("ok");
        let without = build_report_with_pack_and_health(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &features,
            &[],
            &[],
            None,
        )
        .expect("ok");
        assert!(with_h.markdown.contains("72"));
        assert!(without.markdown.contains("72"));
    }
}
