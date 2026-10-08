//! BioFocus desktop shell — Tauri host over workspace Core crates.
//!
//! # Internal IPC (Phase 1)
//!
//! Frontend talks to Core **only** via Tauri `invoke`. Do not open SQLite from UI.
//! Menubar status labels/states are owned by the frontend (P1-E3-T2).
//!
//! ## `get_status`
//!
//! Returns app version + DB probe result (`ok` | `error`) + Menubar
//! `alertLevel` (`green` | `yellow` | `red`). Never includes raw Observation
//! or biometric payloads. Kept lean — Feature series live in
//! [`get_feature_snapshot`].
//!
//! Example (camelCase JSON):
//! ```json
//! { "version": "0.1.0", "dbStatus": "ok", "alertLevel": "green" }
//! { "version": "0.1.0", "dbStatus": "error", "dbError": "…", "alertLevel": "green" }
//! ```
//!
//! ## `get_feature_snapshot`
//!
//! Returns the latest cached Feature snapshot (+ optional Signals) from the
//! Feature Worker. Pure read of in-memory cache — no busy-loop, no SQLite from
//! the command path. Empty when idle / no evidence yet.
//!
//! Example (camelCase JSON):
//! ```json
//! {
//!   "features": [
//!     {
//!       "featureId": "FocusScore",
//!       "timeWindow": { "start": 100, "end": 1000 },
//!       "value": 72.5,
//!       "provenance": ["…uuid…"],
//!       "confidence": 1.0,
//!       "factors": [
//!         { "id": "typing", "label": "Typing activity", "share": 0.4 },
//!         { "id": "stability", "label": "App stability", "share": 0.35 },
//!         { "id": "hrv", "label": "Heart-rate variability", "share": 0.25 }
//!       ]
//!     }
//!   ],
//!   "signals": []
//! }
//! ```
//!
//! ## `get_insights` (P4-E2-T3 / P8-E2-T1)
//!
//! Evaluates product Insight rules against the latest cached Feature snapshot
//! (same in-memory path as `get_feature_snapshot`). Host registers
//! `knowledge_engine::register_insights_v1` once at startup. Pattern Discovery
//! baseline rules (ADR-008) may recompute a bounded FocusScore series from local
//! Observations on read (Core only — UI ↛ SQLite); optional in-process memo.
//! Idle / thin history / no match / soft-fail → omit pattern Insight / `{ "insights": [] }`.
//! No Feature-history table, no always-on worker, no LLM.
//!
//! Example (camelCase JSON):
//! ```json
//! {
//!   "insights": [
//!     {
//!       "id": "…uuid…",
//!       "title": "Sustained stress pattern",
//!       "description": "…",
//!       "category": "stress",
//!       "evidenceList": [{ "kind": "signal", "id": "…uuid…" }],
//!       "actionRecommendation": "…"
//!     }
//!   ]
//! }
//! ```
//!
//! ## `get_recommendations` (P9-E3-T1 / ADR-009)
//!
//! Evaluate-on-read Recommendations **after** Insights on the same Feature
//! snapshot (+ pattern baseline inputs). Host registers
//! `register_recommendations_v1` alongside Insights at startup. Soft-fail /
//! idle / no match → `{ "recommendations": [] }`. No Recommendation SQLite,
//! no busy-loop, no LLM inventing actions. UI ↛ SQLite.
//!
//! Example (camelCase JSON):
//! ```json
//! {
//!   "recommendations": [
//!     {
//!       "id": "…uuid…",
//!       "title": "A gentler pace may help",
//!       "suggestion": "If it fits your schedule…",
//!       "category": "pace",
//!       "evidenceList": [
//!         { "kind": "feature", "id": "FocusScore" },
//!         { "kind": "insight", "id": "…uuid…" }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## `get_local_llm_status` (P11-E3-T1)
//!
//! Calm local-AI provider status from host env (`BIOFOCUS_LOCAL_LLM*`):
//! `disabled` / `ready` / `error`. Reflects config only — **no** HTTP probe,
//! no secrets/tokens. Safe to read on Dashboard open; never invokes interpret.
//!
//! ## `generate_report` (P4-E3-T3 / P11-E3-T1)
//!
//! Explicit user action only — never call on app / Dashboard open. Builds an
//! offline report from the cached Feature snapshot + evaluate-on-read Insights
//! + Recommendations (+ last 8h of non-retracted Life Events) via
//! `report_engine::build_report_with_pack_and_life_events`
//! (`biofocus.default` @ `1`). When `BIOFOCUS_LOCAL_LLM` is enabled,
//! optionally runs `interpret_report` (local HTTP). When disabled, returns
//! deterministic markdown/prompt with `llmStatus: "disabled"` and **no**
//! network. Soft-fails LLM errors into typed status so markdown still returns.
//!
//! Example (camelCase JSON):
//! ```json
//! {
//!   "markdown": "# BioFocus report\n…",
//!   "llmPrompt": "…",
//!   "llmStatus": "disabled"
//! }
//! ```
//!
//! ## `open_dashboard` (P4-E1-T2)
//!
//! Shows the preconfigured `dashboard` webview (hide-on-close). Menubar shell
//! invokes this; no SQLite.
//!
//! ## `core_ping`
//!
//! Legacy scaffold probe (T1). UI prefers `get_status`; kept as fallback.
//!
//! ## `get_pairing_token`
//!
//! Returns the local pairing Bearer token (+ QR SVG) for companion share.
//! Never returns filesystem paths. UI must not read `~/.biofocus` itself.
//!
//! ## `log_life_event` / `list_recent_life_events` (P6-E2-T1)
//!
//! Menubar quick-log for v1 Life Event Observations (`data_type: "life_event"`,
//! `payload.kind` ∈ coffee / walk / lunch / workout). Host validates via
//! `bio_spec`, appends through [`storage::ObservationRepository`] (same store
//! as ingest — no parallel table). UI never opens SQLite.
//!
//! `retract_life_event` / `restore_life_event` / `retime_life_event` are
//! append-only: removal writes a `life_event_retraction` marker that storage
//! readers honour; nothing is deleted. `list_life_events_between` feeds chart
//! markers. `timestamp` = happened at, payload `logged_at` = when tapped.
//!
//! ## `get_git_watched_roots` / `set_git_watched_roots` (P14-E3-T1 / ADR-014)
//!
//! Menubar editor for personal Git watched folders. Host reads/writes
//! `git-watched-roots.toml` only (`BIOFOCUS_HOME` for tests). Paths stay in
//! config — never copied into Observation payloads or default logs. UI ↛ SQLite.
//!
//! ## Local ingest HTTP (Phase 2)
//!
//! On startup the host opens the default DB, loads [`ingest::IngestConfig`],
//! spawns the persist worker, and serves ingest (default `127.0.0.1:8787`;
//! LAN opt-in via `BIOFOCUS_INGEST_LAN` / bind-host — ADR-005).
//! Companion/debug use `GET /v1/status`; the shell UI still uses IPC `get_status`.
//!
//! ## Feature Worker (Phase 3 / 4)
//!
//! A Core Feature Worker polls new Observations from SQLite, runs pipeline
//! quality stages (accept → dedupe → normalize), then catalog Feature Engine +
//! [`feature_engine::map_alert_level`] into shared [`alert_state::AlertState`]
//! and caches [`feature_engine::FeatureSnapshot`] in
//! [`alert_state::SnapshotState`] for IPC. Started/stopped with the desktop process.
//!
//! See also `docs/09-api.md`.

#![cfg_attr(not(test), forbid(unsafe_code))]

mod alert_state;
mod data_sources_ipc;
mod feature_host;
mod git_watched_roots_ipc;
mod ingest_host;
mod life_event_ipc;
mod pattern_host;
mod series_host;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{EvidenceRef, Insight, Recommendation};
use feature_engine::{AlertLevel, Feature, FeatureSnapshot, FeatureValue, Signal};
use knowledge_engine::{
    KnowledgeEngine, PatternInputs, register_insights_v1, register_recommendations_v1,
};
use qrcode::QrCode;
use qrcode::render::svg;
use report_engine::{
    DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION, LocalLlmConfig, ReportDocument,
    ReportEngineError, ReportLifeEvent, build_report_with_pack_and_life_events, interpret_report,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use thiserror::Error;
use tracing::warn;

use crate::alert_state::{AlertState, SnapshotState};
use crate::pattern_host::{BaselineMemoState, load_focus_baseline_series};
use crate::series_host::{
    FeatureSeriesResult, SeriesMemoState, latest_features_per_id, load_feature_series,
};

/// Process-lifetime Knowledge Engine with v1 Insight + Recommendation rules.
struct InsightsEngineState {
    engine: KnowledgeEngine,
}

impl InsightsEngineState {
    /// Registers Insights + Recommendations v1; soft-fails per registry.
    fn new() -> Self {
        let mut engine = KnowledgeEngine::new();
        if let Err(err) = register_insights_v1(&mut engine) {
            warn!(
                error = %err,
                "insights: register_insights_v1 failed; get_insights returns []"
            );
        }
        if let Err(err) = register_recommendations_v1(&mut engine) {
            warn!(
                error = %err,
                "recommendations: register_recommendations_v1 failed; get_recommendations returns []"
            );
        }
        Self { engine }
    }
}

/// Errors from the desktop host bootstrap (no panics on the production path).
#[derive(Debug, Error)]
pub enum DesktopError {
    #[error("runtime: {0}")]
    Runtime(#[from] runtime::RuntimeError),
    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("{0}")]
    Message(&'static str),
}

pub type DesktopResult<T> = Result<T, DesktopError>;

/// IPC payload for [`get_status`]. No Observation / biometric fields.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CoreStatus {
    /// Desktop package version (`CARGO_PKG_VERSION`).
    version: String,
    /// `"ok"` after successful open+migrate probe; otherwise `"error"`.
    db_status: String,
    /// Short storage error when `db_status == "error"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    db_error: Option<String>,
    /// Menubar traffic-light level (`green` / `yellow` / `red`). Independent of
    /// Idle/Ready/Error (`db_status`); defaults to green without Feature evidence.
    alert_level: String,
}

/// IPC wire value for a Feature (scalar or JSON object).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
enum FeatureValueDto {
    Scalar(f64),
    Object(JsonValue),
}

impl From<&FeatureValue> for FeatureValueDto {
    fn from(value: &FeatureValue) -> Self {
        match value {
            FeatureValue::Scalar(v) => Self::Scalar(*v),
            FeatureValue::Object(v) => Self::Object(v.clone()),
        }
    }
}

/// One calm contribution factor on the IPC wire (P7-E2).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ExplanationFactorDto {
    id: String,
    label: String,
    share: f64,
}

impl From<&bio_spec::ExplanationFactor> for ExplanationFactorDto {
    fn from(factor: &bio_spec::ExplanationFactor) -> Self {
        Self {
            id: factor.id.clone(),
            label: factor.label.clone(),
            share: factor.share,
        }
    }
}

/// One Feature in the IPC snapshot (camelCase; provenance = Observation ids only).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FeatureDto {
    feature_id: String,
    time_window: TimeWindowDto,
    value: FeatureValueDto,
    provenance: Vec<String>,
    /// Derived Feature confidence `[0.0, 1.0]` (ADR-007). Data quality, not clinical.
    confidence: f64,
    /// Calm “why this value” factors (P7-E2). Omitted when empty (catalog may not emit yet).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    factors: Vec<ExplanationFactorDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TimeWindowDto {
    start: i64,
    end: i64,
}

/// One Signal in the IPC snapshot (no Observation payloads).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SignalDto {
    id: String,
    #[serde(rename = "type")]
    signal_type: String,
    timestamp_start: i64,
    timestamp_end: i64,
    severity: String,
}

/// IPC payload for [`get_feature_snapshot`].
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FeatureSnapshotDto {
    features: Vec<FeatureDto>,
    signals: Vec<SignalDto>,
}

impl From<&Feature> for FeatureDto {
    fn from(feature: &Feature) -> Self {
        Self {
            feature_id: feature.feature_id.clone(),
            time_window: TimeWindowDto {
                start: feature.time_window.start.as_secs(),
                end: feature.time_window.end.as_secs(),
            },
            value: FeatureValueDto::from(&feature.value),
            provenance: feature.provenance.iter().map(|id| id.to_string()).collect(),
            confidence: feature.confidence.get(),
            factors: feature
                .factors
                .iter()
                .map(ExplanationFactorDto::from)
                .collect(),
        }
    }
}

impl From<&Signal> for SignalDto {
    fn from(signal: &Signal) -> Self {
        Self {
            id: signal.id.to_string(),
            signal_type: signal.signal_type.clone(),
            timestamp_start: signal.timestamp_start.as_secs(),
            timestamp_end: signal.timestamp_end.as_secs(),
            severity: severity_wire(signal.severity),
        }
    }
}

fn severity_wire(severity: bio_spec::Severity) -> String {
    match severity {
        bio_spec::Severity::Low => "low".into(),
        bio_spec::Severity::Medium => "medium".into(),
        bio_spec::Severity::High => "high".into(),
        bio_spec::Severity::Critical => "critical".into(),
    }
}

fn snapshot_to_dto(snapshot: &FeatureSnapshot) -> FeatureSnapshotDto {
    // Snapshot surfaces stay latest-oriented (ADR-018); chart uses series IPC.
    let latest = latest_features_per_id(&snapshot.features);
    FeatureSnapshotDto {
        features: latest.iter().map(FeatureDto::from).collect(),
        signals: snapshot.signals.iter().map(SignalDto::from).collect(),
    }
}

/// IPC payload for [`get_feature_series`] (ADR-018 / P17-E3).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FeatureSeriesDto {
    range: String,
    step_secs: i64,
    window: TimeWindowDto,
    features: Vec<FeatureDto>,
}

fn series_to_dto(series: &FeatureSeriesResult) -> FeatureSeriesDto {
    FeatureSeriesDto {
        range: series.range.clone(),
        step_secs: series.step_secs,
        window: TimeWindowDto {
            start: series.window_start,
            end: series.window_end,
        },
        features: series.features.iter().map(FeatureDto::from).collect(),
    }
}

/// Evidence ref on the IPC wire (`feature` | `signal` | `insight` | `observation` + id string).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct EvidenceRefDto {
    kind: String,
    id: String,
}

impl From<&EvidenceRef> for EvidenceRefDto {
    fn from(evidence: &EvidenceRef) -> Self {
        match evidence {
            EvidenceRef::Feature(id) => Self {
                kind: "feature".into(),
                id: id.clone(),
            },
            EvidenceRef::Signal(id) => Self {
                kind: "signal".into(),
                id: id.to_string(),
            },
            EvidenceRef::Insight(id) => Self {
                kind: "insight".into(),
                id: id.to_string(),
            },
            EvidenceRef::Observation(id) => Self {
                kind: "observation".into(),
                id: id.to_string(),
            },
        }
    }
}

/// One Insight in the IPC list (camelCase; evidence ids only).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct InsightDto {
    id: String,
    title: String,
    description: String,
    category: String,
    evidence_list: Vec<EvidenceRefDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_recommendation: Option<String>,
}

impl From<&Insight> for InsightDto {
    fn from(insight: &Insight) -> Self {
        Self {
            id: insight.id.to_string(),
            title: insight.title.clone(),
            description: insight.description.clone(),
            category: insight.category.clone(),
            evidence_list: insight
                .evidence_list
                .iter()
                .map(EvidenceRefDto::from)
                .collect(),
            action_recommendation: insight.action_recommendation.clone(),
        }
    }
}

/// IPC payload for [`get_insights`].
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct InsightsDto {
    insights: Vec<InsightDto>,
}

fn insights_to_dto(insights: &[Insight]) -> InsightsDto {
    InsightsDto {
        insights: insights.iter().map(InsightDto::from).collect(),
    }
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn pattern_inputs_for_insights(app: &AppHandle, snapshot: &FeatureSnapshot) -> PatternInputs {
    let reference_ts = snapshot
        .features
        .iter()
        .filter(|f| f.feature_id == "FocusScore")
        .map(|f| f.time_window.end.as_secs())
        .max()
        .unwrap_or_else(unix_now_secs);
    let memo = app.try_state::<BaselineMemoState>();
    let series = load_focus_baseline_series(memo.as_deref(), reference_ts);
    let pattern = PatternInputs::with_baseline_series(series);

    // Life Events (retracted ones already hidden by storage) + recent stepped
    // Focus / CognitiveLoad series for the before/after rule. Only recompute the
    // 8h series when a comparable event exists.
    let now = unix_now_secs();
    let marks = life_event_ipc::life_event_marks_between(now - LIFE_EVENT_LOOKBACK_SECS, now);
    let comparable = marks
        .iter()
        .any(|m| knowledge_engine::LIFE_EVENT_EFFECT_KINDS.contains(&m.kind.as_str()));
    let recent = if comparable {
        let memo = app.try_state::<LifeEventSeriesMemo>();
        let ids = ["FocusScore".to_string(), "CognitiveLoad".to_string()];
        load_feature_series(memo.as_deref().map(|m| &m.0), "8h", Some(&ids), now).features
    } else {
        Vec::new()
    };
    pattern.with_life_events(marks, recent)
}

/// Non-retracted Life Events for the offline report (soft-fails to empty).
fn report_life_events(start: i64, end: i64) -> Vec<ReportLifeEvent> {
    life_event_ipc::list_life_events_between(start, end)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|dto| {
            Some(ReportLifeEvent {
                id: uuid::Uuid::parse_str(&dto.id).ok()?,
                kind: dto.kind,
                happened_at: dto.timestamp,
                logged_at: dto.logged_at,
            })
        })
        .collect()
}

/// Life Event lookback for insights + report (matches the 8h chart range).
const LIFE_EVENT_LOOKBACK_SECS: i64 = 8 * 3600;

/// Separate single-entry memo so the Life Event series does not evict the
/// chart's memo entry.
struct LifeEventSeriesMemo(SeriesMemoState);

fn evaluate_insights_list(
    engine: &KnowledgeEngine,
    snapshot: &FeatureSnapshot,
    pattern: &PatternInputs,
) -> Vec<Insight> {
    match engine.evaluate_with_pattern(&snapshot.features, &snapshot.signals, pattern) {
        Ok(insights) => insights,
        Err(err) => {
            warn!(error = %err, "insights: evaluate failed; returning empty list");
            Vec::new()
        }
    }
}

fn evaluate_insights(
    engine: &KnowledgeEngine,
    snapshot: &FeatureSnapshot,
    pattern: &PatternInputs,
) -> InsightsDto {
    insights_to_dto(&evaluate_insights_list(engine, snapshot, pattern))
}

/// One Recommendation in the IPC list (camelCase; evidence ids only).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RecommendationDto {
    id: String,
    title: String,
    suggestion: String,
    category: String,
    evidence_list: Vec<EvidenceRefDto>,
}

impl From<&Recommendation> for RecommendationDto {
    fn from(recommendation: &Recommendation) -> Self {
        Self {
            id: recommendation.id.to_string(),
            title: recommendation.title.clone(),
            suggestion: recommendation.suggestion.clone(),
            category: recommendation.category.clone(),
            evidence_list: recommendation
                .evidence_list
                .iter()
                .map(EvidenceRefDto::from)
                .collect(),
        }
    }
}

/// IPC payload for [`get_recommendations`].
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RecommendationsDto {
    recommendations: Vec<RecommendationDto>,
}

fn recommendations_to_dto(recommendations: &[Recommendation]) -> RecommendationsDto {
    RecommendationsDto {
        recommendations: recommendations
            .iter()
            .map(RecommendationDto::from)
            .collect(),
    }
}

fn evaluate_recommendations_list(
    engine: &KnowledgeEngine,
    snapshot: &FeatureSnapshot,
    insights: &[Insight],
) -> Vec<Recommendation> {
    match engine.evaluate_recommendations(&snapshot.features, &snapshot.signals, insights) {
        Ok(recommendations) => recommendations,
        Err(err) => {
            warn!(
                error = %err,
                "recommendations: evaluate failed; returning empty list"
            );
            Vec::new()
        }
    }
}

fn evaluate_recommendations_dto(
    engine: &KnowledgeEngine,
    snapshot: &FeatureSnapshot,
    insights: &[Insight],
) -> RecommendationsDto {
    recommendations_to_dto(&evaluate_recommendations_list(engine, snapshot, insights))
}

/// IPC payload for [`get_local_llm_status`] (P11-E3-T1).
///
/// Config-only status — no HTTP probe, no tokens, no filesystem paths.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct LocalLlmProviderStatusDto {
    /// `"disabled"` | `"ready"` | `"error"`.
    status: String,
    /// Calm one-liner for Dashboard.
    detail: String,
    /// Model id when enabled and usable (not a secret).
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    /// Default pack used by [`generate_report`].
    pack_id: String,
    pack_version: String,
}

fn local_llm_provider_status(config: &LocalLlmConfig) -> LocalLlmProviderStatusDto {
    let pack_id = DEFAULT_PROMPT_PACK_ID.to_owned();
    let pack_version = DEFAULT_PROMPT_PACK_VERSION.to_owned();
    if !config.enabled {
        return LocalLlmProviderStatusDto {
            status: "disabled".into(),
            detail: "Local AI is optional and currently off.".into(),
            model: None,
            pack_id,
            pack_version,
        };
    }

    let url_ok = config.base_url.starts_with("http://") || config.base_url.starts_with("https://");
    let model_ok = !config.model.trim().is_empty();
    if url_ok && model_ok {
        LocalLlmProviderStatusDto {
            status: "ready".into(),
            detail: "Local AI is configured. Interpretation runs only when you generate a report."
                .into(),
            model: Some(config.model.clone()),
            pack_id,
            pack_version,
        }
    } else {
        LocalLlmProviderStatusDto {
            status: "error".into(),
            detail: "Local AI is enabled but the endpoint config looks unusable.".into(),
            model: None,
            pack_id,
            pack_version,
        }
    }
}

/// IPC payload for [`generate_report`] (P4-E3-T3 / P11-E3-T1).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ReportDto {
    markdown: String,
    llm_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    interpretation: Option<String>,
    /// `"disabled"` | `"ok"` | `"error"` | `"timeout"`.
    llm_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    llm_error: Option<String>,
}

fn report_dto_offline(doc: ReportDocument) -> ReportDto {
    ReportDto {
        markdown: doc.markdown,
        llm_prompt: doc.llm_prompt,
        interpretation: None,
        llm_status: "disabled".into(),
        llm_error: None,
    }
}

fn calm_llm_error(err: &ReportEngineError) -> String {
    match err {
        ReportEngineError::LocalLlmTimeout { .. } => "Local AI did not respond in time.".into(),
        ReportEngineError::LocalLlmHttp { .. } => "Could not reach the local AI endpoint.".into(),
        ReportEngineError::LocalLlmResponse { .. } => {
            "Local AI returned an unusable response.".into()
        }
        ReportEngineError::LocalLlmDisabled => "Local AI is optional and currently off.".into(),
        ReportEngineError::BuildFailed { .. } => "Could not build the report.".into(),
        ReportEngineError::UnknownPromptPack { .. } => "That report pack is not available.".into(),
    }
}

async fn assemble_report_dto(
    features: &[Feature],
    insights: &[Insight],
    recommendations: &[Recommendation],
    life_events: &[ReportLifeEvent],
    config: &LocalLlmConfig,
) -> Result<ReportDto, String> {
    let doc = build_report_with_pack_and_life_events(
        DEFAULT_PROMPT_PACK_ID,
        DEFAULT_PROMPT_PACK_VERSION,
        features,
        insights,
        recommendations,
        life_events,
    )
    .map_err(|err| err.to_string())?;

    if !config.enabled {
        // No network when off — explicit offline path for the Dashboard.
        return Ok(report_dto_offline(doc));
    }

    match interpret_report(&doc, config).await {
        Ok(text) => Ok(ReportDto {
            markdown: doc.markdown,
            llm_prompt: doc.llm_prompt,
            interpretation: Some(text),
            llm_status: "ok".into(),
            llm_error: None,
        }),
        Err(ReportEngineError::LocalLlmDisabled) => Ok(report_dto_offline(doc)),
        Err(err @ ReportEngineError::LocalLlmTimeout { .. }) => Ok(ReportDto {
            markdown: doc.markdown,
            llm_prompt: doc.llm_prompt,
            interpretation: None,
            llm_status: "timeout".into(),
            llm_error: Some(calm_llm_error(&err)),
        }),
        Err(err) => Ok(ReportDto {
            markdown: doc.markdown,
            llm_prompt: doc.llm_prompt,
            interpretation: None,
            llm_status: "error".into(),
            llm_error: Some(calm_llm_error(&err)),
        }),
    }
}

/// Placeholder IPC payload kept for T2 fallback until consumers drop it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CorePing {
    status: &'static str,
    runtime: &'static str,
    storage: &'static str,
    db_file: &'static str,
    schema_version: u32,
}

/// IPC payload for ingest LAN preference (P28-E1-T1 / ADR-029).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct IngestLanPreferenceDto {
    /// User persisted opt-in via `~/.biofocus/ingest_lan_enabled`.
    persisted: bool,
    /// Env overrides persisted file for this process.
    from_env: bool,
    /// Current running bind is LAN (may lag persisted until restart).
    effective_lan: bool,
    /// After `set_ingest_lan_preference`, host must restart ingest bind.
    needs_restart: bool,
}

fn build_ingest_lan_preference(needs_restart: bool) -> Result<IngestLanPreferenceDto, String> {
    let from_env = ingest::lan_preference_overridden_by_env();
    let persisted = ingest::read_persisted_lan_enabled();
    let configured_lan = ingest::resolve_bind_host()
        .map(|host| !host.is_loopback())
        .map_err(|_| "Could not resolve ingest bind host.".to_string())?;
    // Prefer what is actually listening; fall back to config before startup.
    let effective_lan = match ingest_host::ingest_run_state() {
        ingest_host::IngestRunState::Running { bind_host } => !bind_host.is_loopback(),
        _ => configured_lan,
    };
    let needs_restart = needs_restart || configured_lan != effective_lan;
    Ok(IngestLanPreferenceDto {
        persisted,
        from_env,
        effective_lan,
        needs_restart,
    })
}

/// Reads persisted LAN opt-in and effective bind mode (no SQLite).
#[tauri::command]
fn get_ingest_lan_preference() -> Result<IngestLanPreferenceDto, String> {
    build_ingest_lan_preference(false)
}

/// Persists LAN opt-in for next launch (ADR-005 — still explicit opt-in).
#[tauri::command]
fn set_ingest_lan_preference(enabled: bool) -> Result<IngestLanPreferenceDto, String> {
    if ingest::lan_preference_overridden_by_env() {
        return Err("LAN bind is controlled by environment variables for this launch.".into());
    }
    ingest::write_persisted_lan_enabled(enabled).map_err(|err| match err {
        ingest::IngestError::HomeDirUnavailable => "Could not locate local BioFocus config.".into(),
        ingest::IngestError::TokenIo { .. } => "Could not save LAN preference.".into(),
        other => other.to_string(),
    })?;
    build_ingest_lan_preference(true)
}

/// IPC payload for [`get_pairing_token`]. Local secret only — no cloud, no paths.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PairingTokenInfo {
    /// Bearer token value (64 hex chars when from file generation).
    token: String,
    /// Primary ingest base URL (loopback or LAN hint). Companions / Simulator.
    ingest_base_url: String,
    /// `"loopback"` or `"lan"` — matches HTTP `/v1/status` `bind_mode`.
    bind_mode: String,
    /// Usable base URL hints (primary first). Empty only if LAN discovery failed.
    base_url_hints: Vec<String>,
    /// `true` when `BIOFOCUS_INGEST_TOKEN` overrides the on-disk file.
    from_env: bool,
    /// SVG markup for a QR encoding URL, token, and certificate fingerprint.
    qr_svg: String,
    /// SHA-256 of the LAN certificate, when this listener speaks TLS. Absent on loopback.
    cert_fingerprint: Option<String>,
    /// Settings (env / saved toggle) ask for LAN on the next launch.
    lan_configured: bool,
    /// Saved settings differ from the running listener → restart BioFocus.
    restart_required: bool,
    /// Ingest listener is up (false if it failed or has not started).
    ingest_running: bool,
    /// Short UI-safe reason when ingest is not running.
    ingest_error: Option<String>,
}

/// Running listener vs. configured bind (pure; unit-tested).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PairingRuntime {
    running_bind: Option<std::net::Ipv4Addr>,
    configured_bind: std::net::Ipv4Addr,
    ingest_error: Option<String>,
}

/// Maps token resolve errors to short UI-safe strings (no filesystem paths).
fn pairing_error_message(err: ingest::IngestError) -> String {
    match err {
        ingest::IngestError::HomeDirUnavailable => "Could not locate local pairing data.".into(),
        ingest::IngestError::TokenIo { .. } => "Could not read pairing token.".into(),
        ingest::IngestError::EmptyTokenFile { .. } => "Pairing token is empty.".into(),
        ingest::IngestError::TokenEntropy(_) => "Could not create pairing token.".into(),
        ingest::IngestError::TokenFromEnv => {
            "Token is set by the environment for this launch.".into()
        }
        ingest::IngestError::Tls(_) => "Could not prepare the LAN certificate.".into(),
        _ => "Could not load pairing token.".into(),
    }
}

fn render_token_qr_svg(token: &str) -> Result<String, String> {
    let code = QrCode::new(token.as_bytes()).map_err(|err| err.to_string())?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(168, 168)
        .dark_color(svg::Color("#1d1d1f"))
        .light_color(svg::Color("#f5f5f7"))
        .build())
}

fn bind_mode_label(mode: ingest::BindMode) -> &'static str {
    match mode {
        ingest::BindMode::Loopback => "loopback",
        ingest::BindMode::Lan => "lan",
    }
}

#[cfg(test)]
fn build_pairing_info(
    token: String,
    from_env: bool,
    advertise: &ingest::AdvertiseInfo,
) -> Result<PairingTokenInfo, String> {
    let runtime = PairingRuntime {
        running_bind: None,
        configured_bind: if advertise.bind_mode == ingest::BindMode::Lan {
            ingest::INGEST_LAN_BIND_HOST
        } else {
            ingest::INGEST_BIND_HOST
        },
        ingest_error: None,
    };
    build_pairing_info_with(token, from_env, advertise, &runtime, None)
}

fn build_pairing_info_with(
    token: String,
    from_env: bool,
    advertise: &ingest::AdvertiseInfo,
    runtime: &PairingRuntime,
    cert_fingerprint: Option<String>,
) -> Result<PairingTokenInfo, String> {
    // Prefer primary LAN/loopback hint; fall back to loopback so Simulator path stays usable.
    let ingest_base_url = advertise
        .primary_base_url()
        .map(str::to_owned)
        .unwrap_or_else(|| {
            ingest::http_base_url(ingest::INGEST_BIND_HOST, ingest::DEFAULT_INGEST_PORT)
        });
    let qr_svg = render_token_qr_svg(&ingest::pairing_qr_payload(
        &ingest_base_url,
        &token,
        cert_fingerprint.as_deref(),
    ))?;
    Ok(PairingTokenInfo {
        token,
        ingest_base_url,
        bind_mode: bind_mode_label(advertise.bind_mode).to_owned(),
        base_url_hints: advertise.base_url_hints.clone(),
        from_env,
        qr_svg,
        cert_fingerprint,
        lan_configured: !runtime.configured_bind.is_loopback(),
        restart_required: runtime
            .running_bind
            .is_some_and(|running| running.is_loopback() != runtime.configured_bind.is_loopback()),
        ingest_running: runtime.ingest_error.is_none(),
        ingest_error: runtime.ingest_error.clone(),
    })
}

fn resolve_configured_bind() -> Result<std::net::Ipv4Addr, String> {
    ingest::resolve_bind_host().map_err(|err| match err {
        ingest::IngestError::InvalidBindHost { value } => {
            format!("Invalid ingest bind host: {value}")
        }
        _ => "Could not resolve ingest bind host.".into(),
    })
}

/// Advertise hints from the **configured** bind (tests).
#[cfg(test)]
fn resolve_pairing_advertise() -> Result<ingest::AdvertiseInfo, String> {
    Ok(ingest::AdvertiseInfo::for_bind(
        resolve_configured_bind()?,
        ingest::DEFAULT_INGEST_PORT,
    ))
}

/// Builds pairing info from the listener that is actually running. LAN address
/// discovery runs fresh on every call, so the UI "Reload" re-detects it.
fn resolve_pairing_info() -> Result<PairingTokenInfo, String> {
    let from_env = std::env::var(ingest::INGEST_TOKEN_ENV)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let token = ingest::resolve_ingest_token().map_err(pairing_error_message)?;
    let configured_bind = resolve_configured_bind()?;
    let (running_bind, ingest_error) = match ingest_host::ingest_run_state() {
        ingest_host::IngestRunState::Running { bind_host } => (Some(bind_host), None),
        ingest_host::IngestRunState::Failed { reason } => (None, Some(reason)),
        ingest_host::IngestRunState::NotStarted => (None, None),
    };
    let advertise_bind = running_bind.unwrap_or(configured_bind);
    let advertise = ingest::AdvertiseInfo::for_bind(advertise_bind, ingest::DEFAULT_INGEST_PORT);
    let cert_fingerprint = if advertise_bind.is_loopback() {
        None
    } else {
        Some(
            ingest::load_or_create_tls_identity()
                .map_err(pairing_error_message)?
                .fingerprint_hex,
        )
    };
    let runtime = PairingRuntime {
        running_bind,
        configured_bind,
        ingest_error,
    };
    build_pairing_info_with(token, from_env, &advertise, &runtime, cert_fingerprint)
}

/// Probes a DB path (create + WAL + migrate-on-open). Soft-fail via `Err`.
/// Error strings are path-free ([`storage::StorageError::public_message`]).
fn probe_database_at(path: &Path) -> Result<(), String> {
    let _db = storage::Database::open(path).map_err(|err| err.public_message())?;
    Ok(())
}

fn probe_default_database() -> Result<(), String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    probe_database_at(&path)
}

fn build_status(probe: Result<(), String>, alert: AlertLevel) -> CoreStatus {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let alert_level = alert.as_str().to_owned();
    match probe {
        Ok(()) => CoreStatus {
            version,
            db_status: "ok".into(),
            db_error: None,
            alert_level,
        },
        Err(message) => CoreStatus {
            version,
            db_status: "error".into(),
            db_error: Some(message),
            alert_level,
        },
    }
}

fn current_alert_level(app: &AppHandle) -> AlertLevel {
    app.try_state::<AlertState>()
        .map(|state| state.current())
        .unwrap_or(AlertLevel::Green)
}

fn current_feature_snapshot(app: &AppHandle) -> FeatureSnapshot {
    app.try_state::<SnapshotState>()
        .map(|state| state.current())
        .unwrap_or_else(FeatureSnapshot::empty)
}

/// Core status for Menubar/UI. Soft-fails DB problems into `dbStatus: "error"`.
#[tauri::command]
fn get_status(app: AppHandle) -> CoreStatus {
    build_status(probe_default_database(), current_alert_level(&app))
}

/// Latest cached Feature snapshot for Dashboard (P4-E1-T1). Pure cache read.
///
/// Features are collapsed to **latest per `featureId`** (ADR-018). Chart series
/// use [`get_feature_series`].
#[tauri::command]
fn get_feature_snapshot(app: AppHandle) -> FeatureSnapshotDto {
    snapshot_to_dto(&current_feature_snapshot(&app))
}

/// Recompute-on-read Feature series for a Dashboard chart range (ADR-018 / P17-E3).
///
/// Host loads Observations for the span and runs FeatureEngine with the range
/// step. Soft-fails to empty `features`. No Feature-history SQLite; UI ↛ DB.
#[tauri::command]
fn get_feature_series(
    app: AppHandle,
    range: String,
    feature_ids: Option<Vec<String>>,
) -> FeatureSeriesDto {
    let memo = app.try_state::<SeriesMemoState>();
    let series = load_feature_series(
        memo.as_deref(),
        &range,
        feature_ids.as_deref(),
        unix_now_secs(),
    );
    series_to_dto(&series)
}

/// Insights from v1 product rules over the cached Feature snapshot (P4-E2-T3).
///
/// Evaluate-on-read — pattern baseline may recompute from Observations (ADR-008).
/// Soft-fails to `[]`. UI never opens SQLite.
#[tauri::command]
fn get_insights(app: AppHandle) -> InsightsDto {
    let snapshot = current_feature_snapshot(&app);
    let Some(state) = app.try_state::<InsightsEngineState>() else {
        return InsightsDto {
            insights: Vec::new(),
        };
    };
    let pattern = pattern_inputs_for_insights(&app, &snapshot);
    evaluate_insights(&state.engine, &snapshot, &pattern)
}

/// Recommendations from v1 rules after Insights (P9-E3-T1 / ADR-009).
///
/// Evaluate-on-read on the same Feature snapshot + pattern inputs as Insights.
/// Soft-fails to `[]`. UI never opens SQLite; no Recommendation persistence.
#[tauri::command]
fn get_recommendations(app: AppHandle) -> RecommendationsDto {
    let snapshot = current_feature_snapshot(&app);
    let Some(state) = app.try_state::<InsightsEngineState>() else {
        return RecommendationsDto {
            recommendations: Vec::new(),
        };
    };
    let pattern = pattern_inputs_for_insights(&app, &snapshot);
    let insights = evaluate_insights_list(&state.engine, &snapshot, &pattern);
    evaluate_recommendations_dto(&state.engine, &snapshot, &insights)
}

/// Calm local-AI provider status from host env (P11-E3-T1).
///
/// Config-only — no HTTP probe, no interpret, no SQLite. Safe on Dashboard open.
#[tauri::command]
fn get_local_llm_status() -> LocalLlmProviderStatusDto {
    local_llm_provider_status(&LocalLlmConfig::from_env())
}

/// Offline report (+ optional local LLM) for Dashboard — **explicit invoke only**
/// (P4-E3-T3 / P11-E3-T1). Never auto-called on app / Dashboard open.
#[tauri::command]
async fn generate_report(app: AppHandle) -> Result<ReportDto, String> {
    let snapshot = current_feature_snapshot(&app);
    let pattern = pattern_inputs_for_insights(&app, &snapshot);
    let (insights, recommendations) = match app.try_state::<InsightsEngineState>() {
        Some(state) => {
            let insights = evaluate_insights_list(&state.engine, &snapshot, &pattern);
            let recommendations =
                evaluate_recommendations_list(&state.engine, &snapshot, &insights);
            (insights, recommendations)
        }
        None => (Vec::new(), Vec::new()),
    };
    let config = LocalLlmConfig::from_env();
    let now = unix_now_secs();
    let life_events = report_life_events(now - LIFE_EVENT_LOOKBACK_SECS, now);
    assemble_report_dto(
        &snapshot.features,
        &insights,
        &recommendations,
        &life_events,
        &config,
    )
    .await
}

/// Shows the Dashboard window (P4-E1-T2). Soft-fail if the window is missing.
#[tauri::command]
fn open_dashboard(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("dashboard") else {
        return Err("Dashboard window is not available.".into());
    };
    window.unminimize().map_err(|err| err.to_string())?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    Ok(())
}

/// Trivial Core link check (T1). Does not open SQLite or expose Observation rows.
#[tauri::command]
fn core_ping() -> Result<CorePing, String> {
    let (_tx, _rx) = runtime::observation_channel(1).map_err(|err| err.to_string())?;
    Ok(CorePing {
        status: "ok",
        runtime: runtime::CRATE_NAME,
        storage: storage::CRATE_NAME,
        db_file: storage::DEFAULT_DB_FILE_NAME,
        schema_version: storage::SCHEMA_VERSION,
    })
}

/// Local pairing secret for companion share (copy / QR). No filesystem paths.
#[tauri::command]
fn get_pairing_token() -> Result<PairingTokenInfo, String> {
    resolve_pairing_info()
}

/// Replaces the on-disk pairing token and the live bearer. The phone must scan again.
#[tauri::command]
fn rotate_pairing_token() -> Result<PairingTokenInfo, String> {
    ingest::rotate_live_pairing_token().map_err(pairing_error_message)?;
    resolve_pairing_info()
}

/// Logs a v1 Life Event Observation from the Menubar (P6-E2-T1 / ADR-006).
///
/// Persists via [`storage::ObservationRepository`] — UI ↛ SQLite. Calm errors.
///
/// `happened_at` (Unix secs, optional) back-dates the event up to 24 h; payload
/// keeps `logged_at` = now.
#[tauri::command]
fn log_life_event(
    kind: String,
    happened_at: Option<i64>,
) -> Result<life_event_ipc::LifeEventDto, String> {
    life_event_ipc::log_life_event(&kind, happened_at)
}

/// Removes a Life Event by appending a retraction marker (append-only; nothing deleted).
#[tauri::command]
fn retract_life_event(id: String) -> Result<life_event_ipc::LifeEventRetractionDto, String> {
    life_event_ipc::retract_life_event(&id)
}

/// Undo a removal (appends a copy with the same happened-at / logged-at).
#[tauri::command]
fn restore_life_event(id: String) -> Result<life_event_ipc::LifeEventDto, String> {
    life_event_ipc::restore_life_event(&id)
}

/// Change when a Life Event happened (re-timed copy + retraction of the old row).
#[tauri::command]
fn retime_life_event(id: String, happened_at: i64) -> Result<life_event_ipc::LifeEventDto, String> {
    life_event_ipc::retime_life_event(&id, happened_at)
}

/// Non-retracted Life Events with happened-at in `[start, end]` (chart markers).
#[tauri::command]
fn list_life_events_between(
    start: i64,
    end: i64,
) -> Result<Vec<life_event_ipc::LifeEventDto>, String> {
    life_event_ipc::list_life_events_between(start, end)
}

/// Recent Life Events for Menubar confirmation (newest first). Soft-fails to Err string.
#[tauri::command]
fn list_recent_life_events(
    limit: Option<u32>,
) -> Result<Vec<life_event_ipc::LifeEventDto>, String> {
    life_event_ipc::list_recent_life_events(limit)
}

/// Wearable sources for the Dashboard tab. No raw payloads.
#[tauri::command]
fn get_data_sources() -> data_sources_ipc::DataSourcesDto {
    data_sources_ipc::get_data_sources()
}

/// Saves source priority order (`source-priority.toml`). Returns the refreshed list.
#[tauri::command]
fn set_source_priority(order: Vec<String>) -> Result<data_sources_ipc::DataSourcesDto, String> {
    data_sources_ipc::set_source_priority(order)
}

/// Load personal Git watched folders from the ADR-014 config file (P14-E3-T1).
#[tauri::command]
fn get_git_watched_roots() -> Result<git_watched_roots_ipc::GitWatchedRootsDto, String> {
    git_watched_roots_ipc::get_git_watched_roots()
}

/// Save personal Git watched folders to the ADR-014 config file (P14-E3-T1).
#[tauri::command]
fn set_git_watched_roots(
    roots: Vec<String>,
) -> Result<git_watched_roots_ipc::GitWatchedRootsDto, String> {
    git_watched_roots_ipc::set_git_watched_roots(roots)
}

/// Starts the Tauri event loop (window + tray shell + local ingest).
pub fn run() -> DesktopResult<()> {
    let _ = runtime::init_tracing(Some("info"));

    let app = tauri::Builder::default()
        .setup(|app| {
            let icon = app
                .default_window_icon()
                .ok_or(DesktopError::Message(
                    "default window icon missing; tray cannot be created",
                ))?
                .clone();

            // Id `main` is read by the frontend to update the tooltip after IPC status.
            TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .tooltip("BioFocus — Idle")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Dashboard closes hide the window so reopen stays cheap (P4-E1-T2).
            if let Some(dashboard) = app.get_webview_window("dashboard") {
                let hide_target = dashboard.clone();
                dashboard.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide_target.hide();
                    }
                });
            }

            ingest_host::start_ingest_host(app.handle());
            feature_host::start_feature_host(app.handle());
            app.manage(InsightsEngineState::new());
            app.manage(BaselineMemoState::new());
            app.manage(SeriesMemoState::new());
            app.manage(LifeEventSeriesMemo(SeriesMemoState::new()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_feature_snapshot,
            get_feature_series,
            get_insights,
            get_recommendations,
            get_local_llm_status,
            generate_report,
            open_dashboard,
            core_ping,
            get_pairing_token,
            rotate_pairing_token,
            get_ingest_lan_preference,
            set_ingest_lan_preference,
            log_life_event,
            list_recent_life_events,
            retract_life_event,
            restore_life_event,
            retime_life_event,
            list_life_events_between,
            get_git_watched_roots,
            set_git_watched_roots,
            get_data_sources,
            set_source_priority
        ])
        .build(tauri::generate_context!())?;

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            feature_host::stop_feature_host(app_handle);
            ingest_host::stop_ingest_host(app_handle);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use bio_spec::{Severity, TimeWindow, UnixTimestamp};
    use feature_engine::FeatureValue;
    use uuid::Uuid;

    use super::FeatureSeriesResult;

    /// Serializes env-mutating pairing tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn build_status_ok_has_no_db_error() {
        let status = build_status(Ok(()), AlertLevel::Green);
        assert_eq!(status.db_status, "ok");
        assert_eq!(status.db_error, None);
        assert_eq!(status.alert_level, "green");
        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn build_status_error_keeps_reason_and_alert() {
        let status = build_status(Err("disk full".into()), AlertLevel::Yellow);
        assert_eq!(status.db_status, "error");
        assert_eq!(status.db_error.as_deref(), Some("disk full"));
        assert_eq!(status.alert_level, "yellow");
    }

    #[test]
    fn build_status_alert_independent_of_db_ok() {
        let status = build_status(Ok(()), AlertLevel::Red);
        assert_eq!(status.db_status, "ok");
        assert_eq!(status.alert_level, "red");
    }

    #[test]
    fn storage_public_message_hides_paths_in_db_error() {
        use std::path::PathBuf;

        let err = storage::StorageError::CreateDir {
            path: PathBuf::from("/Users/secret/.biofocus/data"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let status = build_status(Err(err.public_message()), AlertLevel::Green);
        let msg = status.db_error.as_deref().expect("dbError");
        assert_eq!(msg, "Could not create local data directory.");
        assert!(!msg.contains('/'));
        assert!(!msg.contains(".biofocus"));
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(!json.contains("/Users"));
        assert!(!json.contains(".biofocus"));
    }

    #[test]
    fn probe_temp_database_succeeds() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "biofocus-get-status-{}-{}.db",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_file(&path);

        let result = probe_database_at(&path);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));

        assert!(result.is_ok(), "probe failed: {result:?}");
    }

    #[test]
    fn status_json_has_alert_level_without_observation_fields() {
        let status = build_status(Ok(()), AlertLevel::Yellow);
        let json = serde_json::to_value(&status).expect("serialize status");
        let obj = json.as_object().expect("object");
        assert!(obj.contains_key("version"));
        assert!(obj.contains_key("dbStatus"));
        assert_eq!(
            obj.get("alertLevel").and_then(|v| v.as_str()),
            Some("yellow")
        );
        assert!(!obj.contains_key("observations"));
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("hrv"));
        assert!(!obj.contains_key("stressIndex"));
        assert!(!obj.contains_key("features"));
    }

    #[test]
    fn empty_feature_snapshot_dto_is_idle() {
        let dto = snapshot_to_dto(&FeatureSnapshot::empty());
        assert!(dto.features.is_empty());
        assert!(dto.signals.is_empty());
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(
            obj.get("features")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        assert_eq!(
            obj.get("signals")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("path"));
    }

    #[test]
    fn non_empty_feature_snapshot_dto_has_provenance_no_biometrics() {
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let snap = FeatureSnapshot {
            features: vec![Feature {
                feature_id: "FocusScore".into(),
                time_window: window,
                value: FeatureValue::Scalar(72.5),
                provenance: vec![Uuid::from_u128(1)],
                confidence: bio_spec::Confidence::ONE,
                factors: Vec::new(),
            }],
            signals: vec![Signal {
                id: Uuid::from_u128(9),
                signal_type: "High_Stress".into(),
                timestamp_start: UnixTimestamp::from_secs(900),
                timestamp_end: UnixTimestamp::from_secs(1260),
                severity: Severity::High,
            }],
        };
        let dto = snapshot_to_dto(&snap);
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");

        let features = obj
            .get("features")
            .and_then(|v| v.as_array())
            .expect("features");
        assert_eq!(features.len(), 1);
        let f = features[0].as_object().expect("feature obj");
        assert_eq!(
            f.get("featureId").and_then(|v| v.as_str()),
            Some("FocusScore")
        );
        assert_eq!(f.get("value").and_then(|v| v.as_f64()), Some(72.5));
        assert_eq!(f.get("confidence").and_then(|v| v.as_f64()), Some(1.0));
        // Empty factors omitted from IPC JSON (skip_serializing_if).
        assert!(!f.contains_key("factors"));
        let tw = f.get("timeWindow").and_then(|v| v.as_object()).expect("tw");
        assert_eq!(tw.get("start").and_then(|v| v.as_i64()), Some(100));
        assert_eq!(tw.get("end").and_then(|v| v.as_i64()), Some(1000));
        let prov = f
            .get("provenance")
            .and_then(|v| v.as_array())
            .expect("prov");
        assert_eq!(prov.len(), 1);
        assert!(!f.contains_key("payload"));
        assert!(!f.contains_key("rmssd_ms"));

        let signals = obj
            .get("signals")
            .and_then(|v| v.as_array())
            .expect("signals");
        assert_eq!(signals.len(), 1);
        let s = signals[0].as_object().expect("signal obj");
        assert_eq!(s.get("type").and_then(|v| v.as_str()), Some("High_Stress"));
        assert_eq!(s.get("severity").and_then(|v| v.as_str()), Some("high"));
        assert_eq!(s.get("timestampStart").and_then(|v| v.as_i64()), Some(900));

        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains(".biofocus"));
        assert!(!raw.contains("rmssd"));
    }

    #[test]
    fn feature_snapshot_dto_exposes_factors_when_present() {
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let snap = FeatureSnapshot {
            features: vec![Feature {
                feature_id: "FocusScore".into(),
                time_window: window,
                value: FeatureValue::Scalar(72.5),
                provenance: vec![Uuid::from_u128(1)],
                confidence: bio_spec::Confidence::ONE,
                factors: vec![bio_spec::ExplanationFactor {
                    id: "typing".into(),
                    label: "Typing activity".into(),
                    share: 0.4,
                }],
            }],
            signals: vec![],
        };
        let dto = snapshot_to_dto(&snap);
        let json = serde_json::to_value(&dto).expect("serialize");
        let f = json["features"][0].as_object().expect("feature");
        let factors = f
            .get("factors")
            .and_then(|v| v.as_array())
            .expect("factors");
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0]["id"].as_str(), Some("typing"));
        assert_eq!(factors[0]["label"].as_str(), Some("Typing activity"));
        assert_eq!(factors[0]["share"].as_f64(), Some(0.4));
        assert!(!serde_json::to_string(&dto).expect("s").contains("payload"));
    }

    #[test]
    fn snapshot_dto_collapses_to_latest_per_feature_id() {
        let older = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let newer = TimeWindow::try_new(
            UnixTimestamp::from_secs(1100),
            UnixTimestamp::from_secs(2000),
        )
        .expect("window");
        let snap = FeatureSnapshot {
            features: vec![
                Feature {
                    feature_id: "FocusScore".into(),
                    time_window: older,
                    value: FeatureValue::Scalar(10.0),
                    provenance: vec![Uuid::from_u128(1)],
                    confidence: bio_spec::Confidence::ONE,
                    factors: Vec::new(),
                },
                Feature {
                    feature_id: "FocusScore".into(),
                    time_window: newer,
                    value: FeatureValue::Scalar(90.0),
                    provenance: vec![Uuid::from_u128(2)],
                    confidence: bio_spec::Confidence::ONE,
                    factors: Vec::new(),
                },
            ],
            signals: vec![],
        };
        let dto = snapshot_to_dto(&snap);
        assert_eq!(dto.features.len(), 1);
        assert_eq!(dto.features[0].feature_id, "FocusScore");
        assert_eq!(dto.features[0].time_window.end, 2000);
    }

    #[test]
    fn empty_feature_series_dto_is_calm() {
        let series = FeatureSeriesResult::empty("1d", 900, 1000, 2000);
        let dto = series_to_dto(&series);
        assert!(dto.features.is_empty());
        assert_eq!(dto.range, "1d");
        assert_eq!(dto.step_secs, 900);
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.get("range").and_then(|v| v.as_str()), Some("1d"));
        assert_eq!(obj.get("stepSecs").and_then(|v| v.as_i64()), Some(900));
        assert_eq!(
            obj.get("features")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        assert!(!serde_json::to_string(&dto).expect("s").contains("payload"));
        assert!(!serde_json::to_string(&dto).expect("s").contains("rmssd"));
    }

    #[test]
    fn non_empty_feature_series_dto_has_wire_shape_no_biometrics() {
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let series = FeatureSeriesResult {
            range: "1h".into(),
            step_secs: 60,
            window_start: 0,
            window_end: 3600,
            features: vec![Feature {
                feature_id: "ActivityBalance".into(),
                time_window: window,
                value: FeatureValue::Scalar(55.0),
                provenance: vec![Uuid::from_u128(7)],
                confidence: bio_spec::Confidence::ONE,
                factors: Vec::new(),
            }],
        };
        let dto = series_to_dto(&series);
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.get("range").and_then(|v| v.as_str()), Some("1h"));
        assert_eq!(obj.get("stepSecs").and_then(|v| v.as_i64()), Some(60));
        let tw = obj
            .get("window")
            .and_then(|v| v.as_object())
            .expect("window");
        assert_eq!(tw.get("start").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(tw.get("end").and_then(|v| v.as_i64()), Some(3600));
        let features = obj
            .get("features")
            .and_then(|v| v.as_array())
            .expect("features");
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get("featureId").and_then(|v| v.as_str()),
            Some("ActivityBalance")
        );
        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("rmssd"));
        assert!(!raw.contains("/Users"));
    }

    #[test]
    fn empty_snapshot_insights_dto_is_idle() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let dto = evaluate_insights(&engine, &FeatureSnapshot::empty(), &PatternInputs::empty());
        assert!(dto.insights.is_empty());
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(
            obj.get("insights")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("path"));
    }

    #[test]
    fn unregistered_engine_insights_are_empty() {
        let engine = KnowledgeEngine::new();
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let snap = FeatureSnapshot {
            features: vec![Feature {
                feature_id: "ContextSwitchRate".into(),
                time_window: window,
                value: FeatureValue::Scalar(2.5),
                provenance: vec![Uuid::from_u128(2)],
                confidence: bio_spec::Confidence::ONE,
                factors: Vec::new(),
            }],
            signals: vec![Signal {
                id: Uuid::from_u128(9),
                signal_type: "High_Stress".into(),
                timestamp_start: UnixTimestamp::from_secs(900),
                timestamp_end: UnixTimestamp::from_secs(1260),
                severity: Severity::High,
            }],
        };
        let dto = evaluate_insights(&engine, &snap, &PatternInputs::empty());
        assert!(dto.insights.is_empty());
    }

    #[test]
    fn registered_engine_emits_insights_with_evidence_refs() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let signal_id = Uuid::from_u128(9);
        let snap = FeatureSnapshot {
            features: vec![
                Feature {
                    feature_id: "ContextSwitchRate".into(),
                    time_window: window.clone(),
                    value: FeatureValue::Scalar(2.5),
                    provenance: vec![Uuid::from_u128(2)],
                    confidence: bio_spec::Confidence::ONE,
                    factors: Vec::new(),
                },
                Feature {
                    feature_id: "StressIndex".into(),
                    time_window: window,
                    value: FeatureValue::Scalar(80.0),
                    provenance: vec![Uuid::from_u128(3)],
                    confidence: bio_spec::Confidence::ONE,
                    factors: Vec::new(),
                },
            ],
            signals: vec![Signal {
                id: signal_id,
                signal_type: "High_Stress".into(),
                timestamp_start: UnixTimestamp::from_secs(900),
                timestamp_end: UnixTimestamp::from_secs(1260),
                severity: Severity::High,
            }],
        };
        let dto = evaluate_insights(&engine, &snap, &PatternInputs::empty());
        assert!(
            dto.insights.len() >= 2,
            "expected stress + context insights, got {}",
            dto.insights.len()
        );

        let json = serde_json::to_value(&dto).expect("serialize");
        let insights = json
            .get("insights")
            .and_then(|v| v.as_array())
            .expect("insights");
        for item in insights {
            let obj = item.as_object().expect("insight obj");
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("title"));
            assert!(obj.contains_key("description"));
            assert!(obj.contains_key("category"));
            let evidence = obj
                .get("evidenceList")
                .and_then(|v| v.as_array())
                .expect("evidenceList");
            assert!(!evidence.is_empty());
            for ev in evidence {
                let e = ev.as_object().expect("evidence obj");
                let kind = e.get("kind").and_then(|v| v.as_str()).expect("kind");
                assert!(kind == "feature" || kind == "signal", "kind={kind}");
                assert!(e.get("id").and_then(|v| v.as_str()).is_some());
            }
            assert!(!obj.contains_key("payload"));
            assert!(!obj.contains_key("observations"));
        }

        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains(".biofocus"));
        assert!(!raw.contains("rmssd"));
    }

    #[test]
    fn registered_engine_emits_pace_recommendation_with_insight_evidence() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("insights");
        register_recommendations_v1(&mut engine).expect("recommendations");

        let focus = |end: i64, value: f64| {
            let start = end.saturating_sub(900);
            let window = TimeWindow::try_new(
                UnixTimestamp::from_secs(start),
                UnixTimestamp::from_secs(end),
            )
            .expect("window");
            Feature {
                feature_id: "FocusScore".into(),
                time_window: window,
                value: FeatureValue::Scalar(value),
                provenance: vec![],
                confidence: bio_spec::Confidence::ONE,
                factors: Vec::new(),
            }
        };

        let snap = FeatureSnapshot {
            features: vec![focus(10_000, 40.0)],
            signals: vec![],
        };
        let pattern = PatternInputs::with_baseline_series(vec![
            focus(1_000, 70.0),
            focus(2_000, 72.0),
            focus(3_000, 68.0),
        ]);
        let insights = evaluate_insights_list(&engine, &snap, &pattern);
        let pattern_insight = insights
            .iter()
            .find(|i| i.category == "pattern")
            .expect("pattern insight");
        assert!(pattern_insight.description.contains("lower"));

        let dto = evaluate_recommendations_dto(&engine, &snap, &insights);
        assert_eq!(dto.recommendations.len(), 1);
        assert_eq!(dto.recommendations[0].category, "pace");

        let json = serde_json::to_value(&dto).expect("serialize");
        let list = json
            .get("recommendations")
            .and_then(|v| v.as_array())
            .expect("recommendations");
        let obj = list[0].as_object().expect("rec obj");
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("suggestion"));
        let evidence = obj
            .get("evidenceList")
            .and_then(|v| v.as_array())
            .expect("evidenceList");
        let kinds: Vec<&str> = evidence
            .iter()
            .filter_map(|e| e.get("kind").and_then(|k| k.as_str()))
            .collect();
        assert!(kinds.contains(&"feature"));
        assert!(kinds.contains(&"insight"));
        let pattern_id = pattern_insight.id.to_string();
        assert!(
            evidence.iter().any(|e| {
                e.get("kind").and_then(|k| k.as_str()) == Some("insight")
                    && e.get("id").and_then(|i| i.as_str()) == Some(pattern_id.as_str())
            }),
            "evidence should cite pattern Insight id"
        );
        assert!(!serde_json::to_string(&dto).expect("s").contains("payload"));
    }

    #[tokio::test]
    async fn assemble_report_disabled_has_no_interpretation() {
        let dto = assemble_report_dto(&[], &[], &[], &[], &LocalLlmConfig::disabled())
            .await
            .expect("offline report");
        assert_eq!(dto.llm_status, "disabled");
        assert!(dto.interpretation.is_none());
        assert!(dto.llm_error.is_none());
        assert!(dto.markdown.contains("BioFocus"));
        assert!(!dto.llm_prompt.is_empty());
        // Empty Evidence → calm minimal default-pack summary (not an error).
        assert!(dto.markdown.contains("Nothing to summarize"));
        assert!(dto.llm_prompt.contains("Do not invent"));

        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert!(obj.contains_key("markdown"));
        assert!(obj.contains_key("llmPrompt"));
        assert_eq!(
            obj.get("llmStatus").and_then(|v| v.as_str()),
            Some("disabled")
        );
        assert!(!obj.contains_key("interpretation"));
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("path"));
    }

    #[tokio::test]
    async fn assemble_report_with_features_keeps_deterministic_markdown() {
        let window = TimeWindow::try_new(
            UnixTimestamp::from_secs(100),
            UnixTimestamp::from_secs(1000),
        )
        .expect("window");
        let features = vec![Feature {
            feature_id: "FocusScore".into(),
            time_window: window,
            value: FeatureValue::Scalar(72.5),
            provenance: vec![Uuid::from_u128(1)],
            confidence: bio_spec::Confidence::ONE,
            factors: Vec::new(),
        }];
        let dto = assemble_report_dto(&features, &[], &[], &[], &LocalLlmConfig::disabled())
            .await
            .expect("report");
        assert_eq!(dto.llm_status, "disabled");
        assert!(dto.markdown.contains("FocusScore"));
        assert!(dto.llm_prompt.contains("FocusScore"));
        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains(".biofocus"));
    }

    #[tokio::test]
    async fn assemble_report_includes_life_events_section() {
        let ev = ReportLifeEvent {
            id: Uuid::from_u128(77),
            kind: "coffee".into(),
            happened_at: 1_700_000_000,
            logged_at: 1_700_000_900,
        };
        let dto = assemble_report_dto(&[], &[], &[], &[ev], &LocalLlmConfig::disabled())
            .await
            .expect("report");
        assert!(dto.markdown.contains("## Life events"));
        assert!(
            dto.markdown
                .contains("| coffee | 1700000000 | 1700000900 |")
        );
    }

    #[tokio::test]
    async fn assemble_report_includes_recommendations_section() {
        let rec = Recommendation {
            id: Uuid::from_u128(42),
            title: "A gentler pace may help".into(),
            suggestion: "If it fits your schedule, take a short break.".into(),
            category: "pace".into(),
            evidence_list: vec![EvidenceRef::Feature("FocusScore".into())],
        };
        let dto = assemble_report_dto(&[], &[], &[rec], &[], &LocalLlmConfig::disabled())
            .await
            .expect("report");
        assert!(dto.markdown.contains("## Recommendations"));
        assert!(dto.markdown.contains("A gentler pace may help"));
        assert!(dto.llm_prompt.contains("A gentler pace may help"));
        assert_eq!(dto.llm_status, "disabled");
    }

    #[test]
    fn local_llm_provider_status_disabled_by_default() {
        let dto = local_llm_provider_status(&LocalLlmConfig::disabled());
        assert_eq!(dto.status, "disabled");
        assert!(dto.model.is_none());
        assert_eq!(dto.pack_id, DEFAULT_PROMPT_PACK_ID);
        assert_eq!(dto.pack_version, DEFAULT_PROMPT_PACK_VERSION);
        let json = serde_json::to_value(&dto).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.get("status").and_then(|v| v.as_str()), Some("disabled"));
        assert_eq!(
            obj.get("packId").and_then(|v| v.as_str()),
            Some(DEFAULT_PROMPT_PACK_ID)
        );
        assert!(!serde_json::to_string(&dto).expect("s").contains("token"));
        assert!(!serde_json::to_string(&dto).expect("s").contains("Bearer"));
    }

    #[test]
    fn local_llm_provider_status_ready_when_enabled() {
        let cfg = LocalLlmConfig {
            enabled: true,
            base_url: "http://127.0.0.1:11434/v1".into(),
            model: "llama3.2".into(),
            timeout: std::time::Duration::from_secs(30),
            api_key: Some("must-not-leak".into()),
        };
        let dto = local_llm_provider_status(&cfg);
        assert_eq!(dto.status, "ready");
        assert_eq!(dto.model.as_deref(), Some("llama3.2"));
        let wire = serde_json::to_string(&dto).expect("s");
        assert!(!wire.contains("must-not-leak"));
        assert!(!wire.contains("api_key"));
        assert!(!wire.contains("Bearer"));
    }

    #[test]
    fn local_llm_provider_status_error_on_bad_url() {
        let cfg = LocalLlmConfig {
            enabled: true,
            base_url: "not-a-url".into(),
            model: "llama3.2".into(),
            timeout: std::time::Duration::from_secs(30),
            api_key: None,
        };
        let dto = local_llm_provider_status(&cfg);
        assert_eq!(dto.status, "error");
        assert!(dto.model.is_none());
    }

    #[test]
    fn pairing_info_json_has_no_paths() {
        let advertise =
            ingest::AdvertiseInfo::for_bind(ingest::INGEST_BIND_HOST, ingest::DEFAULT_INGEST_PORT);
        let info = build_pairing_info("abc123deadbeef".into(), false, &advertise).expect("qr");
        let json = serde_json::to_value(&info).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(
            obj.get("token").and_then(|v| v.as_str()),
            Some("abc123deadbeef")
        );
        assert_eq!(
            obj.get("ingestBaseUrl").and_then(|v| v.as_str()),
            Some("http://127.0.0.1:8787")
        );
        assert_eq!(
            obj.get("bindMode").and_then(|v| v.as_str()),
            Some("loopback")
        );
        let hints = obj
            .get("baseUrlHints")
            .and_then(|v| v.as_array())
            .expect("baseUrlHints");
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].as_str(), Some("http://127.0.0.1:8787"));
        assert_eq!(obj.get("fromEnv").and_then(|v| v.as_bool()), Some(false));
        let qr = obj.get("qrSvg").and_then(|v| v.as_str()).expect("qrSvg");
        assert!(qr.contains("<svg"), "expected SVG markup");
        assert!(!obj.contains_key("path"));
        assert!(!obj.contains_key("home"));
        let raw = serde_json::to_string(&info).unwrap();
        assert!(!raw.contains(".biofocus"));
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains("observation"));
    }

    #[test]
    fn pairing_info_lan_advertise_uses_primary_hint() {
        let advertise = ingest::AdvertiseInfo::for_bind_with(
            ingest::INGEST_LAN_BIND_HOST,
            ingest::DEFAULT_INGEST_PORT,
            || vec![std::net::Ipv4Addr::new(192, 168, 1, 40)],
        );
        let info = build_pairing_info("tok".into(), true, &advertise).expect("qr");
        assert_eq!(info.bind_mode, "lan");
        assert_eq!(info.ingest_base_url, "https://192.168.1.40:8787");
        assert_eq!(
            info.base_url_hints,
            vec!["https://192.168.1.40:8787".to_owned()]
        );
        assert!(info.from_env);
    }

    #[test]
    fn resolve_pairing_advertise_loopback_by_default() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Isolate from the developer's real ~/.biofocus (LAN opt-in file).
        let home = tempfile::tempdir().expect("tempdir");
        // SAFETY: serialized by ENV_LOCK; restore LAN knobs before unlock.
        unsafe {
            std::env::remove_var(ingest::INGEST_LAN_ENV);
            std::env::remove_var(ingest::INGEST_BIND_HOST_ENV);
            std::env::set_var("BIOFOCUS_HOME", home.path());
        }
        let advertise = resolve_pairing_advertise().expect("advertise");
        unsafe {
            std::env::remove_var("BIOFOCUS_HOME");
        }
        assert_eq!(advertise.bind_mode, ingest::BindMode::Loopback);
        assert_eq!(
            advertise.base_url_hints,
            vec!["http://127.0.0.1:8787".to_owned()]
        );
    }

    #[test]
    fn resolve_pairing_advertise_lan_flag_is_lan_mode() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe {
            std::env::set_var(ingest::INGEST_LAN_ENV, "1");
            std::env::remove_var(ingest::INGEST_BIND_HOST_ENV);
        }
        let advertise = resolve_pairing_advertise().expect("advertise");
        assert_eq!(advertise.bind_mode, ingest::BindMode::Lan);
        // Primary hint may be empty if OS discovery fails in CI; mode must still be lan.
        for url in &advertise.base_url_hints {
            assert!(url.starts_with("https://"), "hint={url}");
            assert!(url.contains(":8787"), "hint={url}");
            assert!(
                !url.contains("127.0.0.1"),
                "LAN hint should not be loopback"
            );
        }
        unsafe {
            std::env::remove_var(ingest::INGEST_LAN_ENV);
        }
    }

    #[test]
    fn pairing_flags_restart_when_lan_enabled_but_running_loopback() {
        let advertise =
            ingest::AdvertiseInfo::for_bind(ingest::INGEST_BIND_HOST, ingest::DEFAULT_INGEST_PORT);
        let runtime = PairingRuntime {
            running_bind: Some(ingest::INGEST_BIND_HOST),
            configured_bind: ingest::INGEST_LAN_BIND_HOST,
            ingest_error: None,
        };
        let info =
            build_pairing_info_with("tok".into(), false, &advertise, &runtime, None).expect("qr");
        assert_eq!(info.bind_mode, "loopback");
        assert!(info.lan_configured);
        assert!(info.restart_required);
        assert!(info.ingest_running);
    }

    #[test]
    fn pairing_reports_ingest_failure_reason() {
        let advertise =
            ingest::AdvertiseInfo::for_bind(ingest::INGEST_BIND_HOST, ingest::DEFAULT_INGEST_PORT);
        let runtime = PairingRuntime {
            running_bind: None,
            configured_bind: ingest::INGEST_BIND_HOST,
            ingest_error: Some("Phone sync is off: port 8787 is already in use.".into()),
        };
        let info =
            build_pairing_info_with("tok".into(), false, &advertise, &runtime, None).expect("qr");
        assert!(!info.ingest_running);
        assert!(!info.restart_required);
        assert!(info.ingest_error.unwrap().contains("8787"));
    }

    #[test]
    fn pairing_error_message_hides_paths() {
        let msg = pairing_error_message(ingest::IngestError::TokenIo {
            path: "/Users/secret/.biofocus/pairing_token".into(),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        });
        assert!(!msg.contains("/Users"));
        assert!(!msg.contains(".biofocus"));
        assert!(!msg.contains("pairing_token"));
    }

    #[test]
    fn resolve_pairing_info_from_env_override() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // SAFETY: serialized by ENV_LOCK; restored before unlock.
        unsafe {
            std::env::set_var(ingest::INGEST_TOKEN_ENV, "ux-test-pairing-token");
        }
        let info = resolve_pairing_info().expect("resolve");
        assert_eq!(info.token, "ux-test-pairing-token");
        assert!(info.from_env);
        assert!(info.qr_svg.contains("<svg"));
        unsafe {
            std::env::remove_var(ingest::INGEST_TOKEN_ENV);
        }
    }
}
