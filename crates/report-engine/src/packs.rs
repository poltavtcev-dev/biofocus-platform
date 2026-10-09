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

/// Like [`build_report_with_pack`] plus a "Life events" section (user-logged
/// coffee / walk / lunch / workout; retracted events must already be removed
/// by the host). Empty slice → identical output to [`build_report_with_pack`].
pub fn build_report_with_pack_and_life_events(
    pack_id: &str,
    version: &str,
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
    life_events: &[ReportLifeEvent],
) -> ReportResult<ReportDocument> {
    build_report_with_pack_full(
        pack_id,
        version,
        features,
        insights,
        recommendations,
        life_events,
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
    build_report_with_pack_full(pack_id, version, features, insights, recommendations, &[], health)
}

/// Full-control variant (tests / hosts): Life Events + explicit health context.
pub fn build_report_with_pack_full(
    pack_id: &str,
    version: &str,
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
    life_events: &[ReportLifeEvent],
    health: Option<&HealthContext>,
) -> ReportResult<ReportDocument> {
    match (pack_id, version) {
        (DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION) => {
            build_default_pack(features, insights, recommendations, life_events, health)
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
    life_events: &[ReportLifeEvent],
    health: Option<&HealthContext>,
) -> ReportResult<ReportDocument> {
    let markdown =
        render_default_markdown(features, insights, recommendations, life_events, health)?;
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
    life_events: &[ReportLifeEvent],
    health: Option<&HealthContext>,
) -> ReportResult<String> {
    let mut out = String::new();
    out.push_str("# Отчёт BioFocus\n\n");
    out.push_str(
        "_Локальное резюме по метрикам, наблюдениям и подсказкам. \
         Это не медицинская оценка._\n\n",
    );

    render_health_context_section(&mut out, health);

    if features.is_empty()
        && insights.is_empty()
        && recommendations.is_empty()
        && life_events.is_empty()
    {
        out.push_str("## Кратко\n\n");
        out.push_str("За этот период пока нечего обобщать.\n");
        return Ok(out);
    }

    render_features_section(&mut out, features)?;
    render_insights_section(&mut out, insights);
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    render_life_events_section(&mut out, life_events);
    render_recommendations_section(&mut out, recommendations);
    Ok(out)
}

/// One user-logged Life Event for the report (host-filtered, non-retracted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportLifeEvent {
    /// Observation id (cited by Insights as `observation:<id>`).
    pub id: bio_spec::ObservationId,
    /// v1 kind.
    pub kind: String,
    /// When it happened (Unix secs, UTC).
    pub happened_at: i64,
    /// When the user logged it (Unix secs, UTC).
    pub logged_at: i64,
}

fn render_life_events_section(out: &mut String, life_events: &[ReportLifeEvent]) {
    if life_events.is_empty() {
        return;
    }
    out.push_str("## События дня\n\n");
    out.push_str(
        "_Записал пользователь. Убранные события не входят. \
         Время — секунды Unix (UTC); «записано» отличается, если время сдвинули._\n\n",
    );
    let mut sorted: Vec<&ReportLifeEvent> = life_events.iter().collect();
    sorted.sort_by(|a, b| (a.happened_at, a.id).cmp(&(b.happened_at, b.id)));
    out.push_str("| Событие | Было (UTC, с) | Записано (UTC, с) | Id |\n");
    out.push_str("| :--- | :--- | :--- | :--- |\n");
    for ev in sorted {
        out.push_str(&format!(
            "| {} | {} | {} | observation:{} |\n",
            ev.kind.replace('|', "\\|"),
            ev.happened_at,
            ev.logged_at,
            ev.id
        ));
    }
    out.push('\n');
}

fn render_health_context_section(out: &mut String, health: Option<&HealthContext>) {
    let Some(ctx) = health.filter(|c| !c.is_empty()) else {
        return;
    };
    out.push_str("## Ваши пометки\n\n");
    out.push_str(
        "_Необязательные личные пометки, которые пользователь уже знает. \
         Это не диагноз и не вывод из биометрии._\n\n",
    );
    if !ctx.conditions.is_empty() {
        out.push_str("- **Пометки:** ");
        out.push_str(&ctx.conditions.join(", "));
        out.push('\n');
    }
    if !ctx.note.is_empty() {
        out.push_str("- **Заметка:** ");
        out.push_str(&ctx.note);
        out.push('\n');
    }
    out.push('\n');
}

fn render_recommendations_section(out: &mut String, recommendations: &[Recommendation]) {
    out.push_str("## Подсказки\n\n");
    if recommendations.is_empty() {
        out.push_str("_За этот период подсказок нет._\n");
        return;
    }

    let mut sorted: Vec<&Recommendation> = recommendations.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));

    for rec in sorted {
        out.push_str(&format!("### {}\n\n", rec.title));
        out.push_str(&format!("{}\n\n", rec.suggestion));
        out.push_str(&format!("- **Категория:** {}\n", rec.category));
        out.push_str(&format!(
            "- **Основание:** {}\n",
            format_evidence(&rec.evidence_list)
        ));
        out.push('\n');
    }
}

fn render_default_llm_prompt(markdown: &str) -> String {
    let mut out = String::new();
    out.push_str(
        "Ты пересказываешь локальное резюме BioFocus. \
         Используй только факты из отчёта ниже. \
         Не выдумывай метрики, основания, наблюдения, подсказки и действия. \
         Не ставь диагнозы и не делай медицинских выводов. \
         Если есть раздел «Ваши пометки», это необязательный контекст, \
         который пользователь уже знает: не ставь диагноз по нему \
         и не добавляй новые состояния. \
         Тон спокойный, без оценки человека. \
         Не пересчитывай метрики — только поясни уже данные значения. \
         Не выдумывай подсказки и не переписывай их как новый совет.\n\n",
    );
    wrap_markdown_for_llm(&mut out, markdown);
    out.push_str(
        "Напиши короткое резюме на русском, несколькими короткими абзацами. \
         Используй время с часов из расписания. \
         Скажи, какие источники есть и каких нет. \
         Не выдумывай данные часов, телефона или компьютера. \
         Лучше спокойные наблюдения, чем советы. \
         Это необязательное личное пояснение, не медицинский совет.\n",
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
        assert!(doc.markdown.contains("нечего обобщать"));
        assert!(!doc.markdown.contains("User-declared context"));
        assert!(doc.llm_prompt.contains("Не ставь диагнозы"));
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
        assert!(doc.markdown.contains("Ваши пометки"));
        assert!(doc.markdown.contains("sleep_sensitive"));
        assert!(doc.markdown.contains("I already know late nights hit me"));
        assert!(doc.markdown.contains("не диагноз"));
        assert!(doc.llm_prompt.contains("Ваши пометки"));
        assert!(doc.llm_prompt.contains("не ставь диагноз"));
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

    #[test]
    fn life_events_section_lists_events_sorted_with_both_timestamps() {
        let events = vec![
            ReportLifeEvent {
                id: Uuid::from_u128(2),
                kind: "walk".into(),
                happened_at: 2_000,
                logged_at: 2_900,
            },
            ReportLifeEvent {
                id: Uuid::from_u128(1),
                kind: "coffee".into(),
                happened_at: 1_000,
                logged_at: 1_000,
            },
        ];
        let doc = build_report_with_pack_full(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[sample_feature()],
            &[],
            &[],
            &events,
            None,
        )
        .expect("build");
        let md = &doc.markdown;
        assert!(md.contains("## События дня"));
        let coffee = md.find("| coffee | 1000 | 1000 |").expect("coffee row");
        let walk = md.find("| walk | 2000 | 2900 |").expect("walk row");
        assert!(coffee < walk, "sorted by happened-at");
        assert!(md.find("## События дня").unwrap() < md.find("## Подсказки").unwrap());
        assert!(doc.llm_prompt.contains("## События дня"));
    }

    #[test]
    fn no_life_events_keeps_report_unchanged() {
        let a = build_report_with_pack_and_health(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[sample_feature()],
            &[],
            &[],
            None,
        )
        .expect("a");
        let b = build_report_with_pack_full(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[sample_feature()],
            &[],
            &[],
            &[],
            None,
        )
        .expect("b");
        assert_eq!(a, b);
        assert!(!a.markdown.contains("Life events"));
    }

    #[test]
    fn life_events_alone_are_not_an_empty_summary() {
        let doc = build_report_with_pack_full(
            DEFAULT_PROMPT_PACK_ID,
            DEFAULT_PROMPT_PACK_VERSION,
            &[],
            &[],
            &[],
            &[ReportLifeEvent {
                id: Uuid::from_u128(3),
                kind: "lunch".into(),
                happened_at: 5,
                logged_at: 5,
            }],
            None,
        )
        .expect("build");
        assert!(!doc.markdown.contains("нечего обобщать"));
        assert!(doc.markdown.contains("| lunch | 5 | 5 |"));
    }
}
