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
//!       "provenance": ["…uuid…"]
//!     }
//!   ],
//!   "signals": []
//! }
//! ```
//!
//! ## `get_insights` (P4-E2-T3)
//!
//! Evaluates product Insight rules against the latest cached Feature snapshot
//! (same in-memory path as `get_feature_snapshot`). Host registers
//! `knowledge_engine::register_insights_v1` once at startup. Idle / no match /
//! evaluate soft-fail → `{ "insights": [] }`. No SQLite, no LLM.
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
//! ## `generate_report` (P4-E3-T3)
//!
//! Explicit user action only — never call on app / Dashboard open. Builds an
//! offline report from the cached Feature snapshot + evaluate-on-read Insights
//! via `report_engine::build_report`. When `BIOFOCUS_LOCAL_LLM` is enabled,
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
//! ## Local ingest HTTP (Phase 2)
//!
//! On startup the host opens the default DB, loads [`ingest::IngestConfig`],
//! spawns the persist worker, and serves loopback ingest (`127.0.0.1:8787`).
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
mod feature_host;
mod ingest_host;

use std::path::Path;

use bio_spec::{EvidenceRef, Insight};
use feature_engine::{AlertLevel, Feature, FeatureSnapshot, FeatureValue, Signal};
use knowledge_engine::{register_insights_v1, KnowledgeEngine};
use qrcode::render::svg;
use qrcode::QrCode;
use report_engine::{
    build_report, interpret_report, LocalLlmConfig, ReportDocument, ReportEngineError,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use thiserror::Error;
use tracing::warn;

use crate::alert_state::{AlertState, SnapshotState};

/// Process-lifetime Knowledge Engine with v1 product rules (P4-E2-T3).
struct InsightsEngineState {
    engine: KnowledgeEngine,
}

impl InsightsEngineState {
    /// Registers [`register_insights_v1`]; soft-fails to an empty engine.
    fn new() -> Self {
        let mut engine = KnowledgeEngine::new();
        if let Err(err) = register_insights_v1(&mut engine) {
            warn!(
                error = %err,
                "insights: register_insights_v1 failed; get_insights returns []"
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

/// One Feature in the IPC snapshot (camelCase; provenance = Observation ids only).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FeatureDto {
    feature_id: String,
    time_window: TimeWindowDto,
    value: FeatureValueDto,
    provenance: Vec<String>,
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
            provenance: feature
                .provenance
                .iter()
                .map(|id| id.to_string())
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
    FeatureSnapshotDto {
        features: snapshot.features.iter().map(FeatureDto::from).collect(),
        signals: snapshot.signals.iter().map(SignalDto::from).collect(),
    }
}

/// Evidence ref on the IPC wire (`feature` | `signal` + id string).
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
            evidence_list: insight.evidence_list.iter().map(EvidenceRefDto::from).collect(),
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

fn evaluate_insights_list(engine: &KnowledgeEngine, snapshot: &FeatureSnapshot) -> Vec<Insight> {
    match engine.evaluate(&snapshot.features, &snapshot.signals) {
        Ok(insights) => insights,
        Err(err) => {
            warn!(error = %err, "insights: evaluate failed; returning empty list");
            Vec::new()
        }
    }
}

fn evaluate_insights(engine: &KnowledgeEngine, snapshot: &FeatureSnapshot) -> InsightsDto {
    insights_to_dto(&evaluate_insights_list(engine, snapshot))
}

/// IPC payload for [`generate_report`] (P4-E3-T3).
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
        ReportEngineError::LocalLlmTimeout { .. } => {
            "Local AI did not respond in time.".into()
        }
        ReportEngineError::LocalLlmHttp { .. } => {
            "Could not reach the local AI endpoint.".into()
        }
        ReportEngineError::LocalLlmResponse { .. } => {
            "Local AI returned an unusable response.".into()
        }
        ReportEngineError::LocalLlmDisabled => {
            "Local AI is optional and currently off.".into()
        }
        ReportEngineError::BuildFailed { .. } => "Could not build the report.".into(),
    }
}

async fn assemble_report_dto(
    features: &[Feature],
    insights: &[Insight],
    config: &LocalLlmConfig,
) -> Result<ReportDto, String> {
    let doc = build_report(features, insights).map_err(|err| err.to_string())?;

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

/// IPC payload for [`get_pairing_token`]. Local secret only — no cloud, no paths.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct PairingTokenInfo {
    /// Bearer token value (64 hex chars when from file generation).
    token: String,
    /// Loopback ingest base URL companions should use on the same Mac / Simulator.
    ingest_base_url: String,
    /// `true` when `BIOFOCUS_INGEST_TOKEN` overrides the on-disk file.
    from_env: bool,
    /// SVG markup for a QR encoding the token (phone camera → paste / scan).
    qr_svg: String,
}

/// Maps token resolve errors to short UI-safe strings (no filesystem paths).
fn pairing_error_message(err: ingest::IngestError) -> String {
    match err {
        ingest::IngestError::HomeDirUnavailable => {
            "Could not locate local pairing data.".into()
        }
        ingest::IngestError::TokenIo { .. } => "Could not read pairing token.".into(),
        ingest::IngestError::EmptyTokenFile { .. } => "Pairing token is empty.".into(),
        ingest::IngestError::TokenEntropy(_) => "Could not create pairing token.".into(),
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

fn build_pairing_info(token: String, from_env: bool) -> Result<PairingTokenInfo, String> {
    let qr_svg = render_token_qr_svg(&token)?;
    let ingest_base_url = format!(
        "http://{}:{}",
        ingest::INGEST_BIND_HOST,
        ingest::DEFAULT_INGEST_PORT
    );
    Ok(PairingTokenInfo {
        token,
        ingest_base_url,
        from_env,
        qr_svg,
    })
}

fn resolve_pairing_info() -> Result<PairingTokenInfo, String> {
    let from_env = std::env::var(ingest::INGEST_TOKEN_ENV)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let token = ingest::resolve_ingest_token().map_err(pairing_error_message)?;
    build_pairing_info(token, from_env)
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
#[tauri::command]
fn get_feature_snapshot(app: AppHandle) -> FeatureSnapshotDto {
    snapshot_to_dto(&current_feature_snapshot(&app))
}

/// Insights from v1 product rules over the cached Feature snapshot (P4-E2-T3).
///
/// Evaluate-on-read — no separate Insights cache / SQLite. Soft-fails to `[]`.
#[tauri::command]
fn get_insights(app: AppHandle) -> InsightsDto {
    let snapshot = current_feature_snapshot(&app);
    let Some(state) = app.try_state::<InsightsEngineState>() else {
        return InsightsDto {
            insights: Vec::new(),
        };
    };
    evaluate_insights(&state.engine, &snapshot)
}

/// Offline report (+ optional local LLM) for Dashboard — **explicit invoke only**
/// (P4-E3-T3). Never auto-called on app / Dashboard open.
#[tauri::command]
async fn generate_report(app: AppHandle) -> Result<ReportDto, String> {
    let snapshot = current_feature_snapshot(&app);
    let insights = match app.try_state::<InsightsEngineState>() {
        Some(state) => evaluate_insights_list(&state.engine, &snapshot),
        None => Vec::new(),
    };
    let config = LocalLlmConfig::from_env();
    assemble_report_dto(&snapshot.features, &insights, &config).await
}

/// Shows the Dashboard window (P4-E1-T2). Soft-fail if the window is missing.
#[tauri::command]
fn open_dashboard(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("dashboard") else {
        return Err("Dashboard window is not available.".into());
    };
    window
        .unminimize()
        .map_err(|err| err.to_string())?;
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_feature_snapshot,
            get_insights,
            generate_report,
            open_dashboard,
            core_ping,
            get_pairing_token
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
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("window");
        let snap = FeatureSnapshot {
            features: vec![Feature {
                feature_id: "FocusScore".into(),
                time_window: window,
                value: FeatureValue::Scalar(72.5),
                provenance: vec![Uuid::from_u128(1)],
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
        assert_eq!(
            s.get("timestampStart").and_then(|v| v.as_i64()),
            Some(900)
        );

        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains(".biofocus"));
        assert!(!raw.contains("rmssd"));
    }

    #[test]
    fn empty_snapshot_insights_dto_is_idle() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let dto = evaluate_insights(&engine, &FeatureSnapshot::empty());
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
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("window");
        let snap = FeatureSnapshot {
            features: vec![Feature {
                feature_id: "ContextSwitchRate".into(),
                time_window: window,
                value: FeatureValue::Scalar(2.5),
                provenance: vec![Uuid::from_u128(2)],
            }],
            signals: vec![Signal {
                id: Uuid::from_u128(9),
                signal_type: "High_Stress".into(),
                timestamp_start: UnixTimestamp::from_secs(900),
                timestamp_end: UnixTimestamp::from_secs(1260),
                severity: Severity::High,
            }],
        };
        let dto = evaluate_insights(&engine, &snap);
        assert!(dto.insights.is_empty());
    }

    #[test]
    fn registered_engine_emits_insights_with_evidence_refs() {
        let mut engine = KnowledgeEngine::new();
        register_insights_v1(&mut engine).expect("register");
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("window");
        let signal_id = Uuid::from_u128(9);
        let snap = FeatureSnapshot {
            features: vec![
                Feature {
                    feature_id: "ContextSwitchRate".into(),
                    time_window: window.clone(),
                    value: FeatureValue::Scalar(2.5),
                    provenance: vec![Uuid::from_u128(2)],
                },
                Feature {
                    feature_id: "StressIndex".into(),
                    time_window: window,
                    value: FeatureValue::Scalar(80.0),
                    provenance: vec![Uuid::from_u128(3)],
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
        let dto = evaluate_insights(&engine, &snap);
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

    #[tokio::test]
    async fn assemble_report_disabled_has_no_interpretation() {
        let dto = assemble_report_dto(&[], &[], &LocalLlmConfig::disabled())
            .await
            .expect("offline report");
        assert_eq!(dto.llm_status, "disabled");
        assert!(dto.interpretation.is_none());
        assert!(dto.llm_error.is_none());
        assert!(dto.markdown.contains("BioFocus"));
        assert!(!dto.llm_prompt.is_empty());

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
        let window =
            TimeWindow::try_new(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(1000))
                .expect("window");
        let features = vec![Feature {
            feature_id: "FocusScore".into(),
            time_window: window,
            value: FeatureValue::Scalar(72.5),
            provenance: vec![Uuid::from_u128(1)],
        }];
        let dto = assemble_report_dto(&features, &[], &LocalLlmConfig::disabled())
            .await
            .expect("report");
        assert_eq!(dto.llm_status, "disabled");
        assert!(dto.markdown.contains("FocusScore"));
        assert!(dto.llm_prompt.contains("FocusScore"));
        let raw = serde_json::to_string(&dto).expect("string");
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains(".biofocus"));
    }

    #[test]
    fn pairing_info_json_has_no_paths() {
        let info = build_pairing_info("abc123deadbeef".into(), false).expect("qr");
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
        assert_eq!(obj.get("fromEnv").and_then(|v| v.as_bool()), Some(false));
        let qr = obj.get("qrSvg").and_then(|v| v.as_str()).expect("qrSvg");
        assert!(qr.contains("<svg"), "expected SVG markup");
        assert!(!obj.contains_key("path"));
        assert!(!obj.contains_key("home"));
        assert!(!serde_json::to_string(&info).unwrap().contains(".biofocus"));
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
        let _guard = ENV_LOCK.lock().expect("env lock");
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
