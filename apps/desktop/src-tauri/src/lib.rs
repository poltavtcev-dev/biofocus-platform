//! BioFocus desktop shell — Tauri host over workspace Core crates.
//!
//! # Internal IPC (Phase 1)
//!
//! Frontend talks to Core **only** via Tauri `invoke`. Do not open SQLite from UI.
//! Menubar status labels/states are owned by the frontend (P1-E3-T2).
//!
//! ## `get_status`
//!
//! Returns app version + DB probe result (`ok` | `error`). Never includes
//! raw Observation or biometric payloads.
//!
//! Example (camelCase JSON):
//! ```json
//! { "version": "0.1.0", "dbStatus": "ok" }
//! { "version": "0.1.0", "dbStatus": "error", "dbError": "…" }
//! ```
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
//! ## Feature Worker (Phase 3)
//!
//! A Core Feature Worker polls new Observations from SQLite, runs pipeline
//! quality stages (accept → dedupe → normalize), and invokes a noop Feature
//! Engine hook until P3-E2. Started/stopped with the desktop process.
//!
//! See also `docs/09-api.md`.

#![cfg_attr(not(test), forbid(unsafe_code))]

mod feature_host;
mod ingest_host;

use std::path::Path;

use qrcode::render::svg;
use qrcode::QrCode;
use serde::Serialize;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent};
use thiserror::Error;

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

fn build_status(probe: Result<(), String>) -> CoreStatus {
    let version = env!("CARGO_PKG_VERSION").to_string();
    match probe {
        Ok(()) => CoreStatus {
            version,
            db_status: "ok".into(),
            db_error: None,
        },
        Err(message) => CoreStatus {
            version,
            db_status: "error".into(),
            db_error: Some(message),
        },
    }
}

/// Core status for Menubar/UI. Soft-fails DB problems into `dbStatus: "error"`.
#[tauri::command]
fn get_status() -> CoreStatus {
    build_status(probe_default_database())
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

            ingest_host::start_ingest_host(app.handle());
            feature_host::start_feature_host(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
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
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Serializes env-mutating pairing tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn build_status_ok_has_no_db_error() {
        let status = build_status(Ok(()));
        assert_eq!(status.db_status, "ok");
        assert_eq!(status.db_error, None);
        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn build_status_error_keeps_reason() {
        let status = build_status(Err("disk full".into()));
        assert_eq!(status.db_status, "error");
        assert_eq!(status.db_error.as_deref(), Some("disk full"));
    }

    #[test]
    fn storage_public_message_hides_paths_in_db_error() {
        use std::io;
        use std::path::PathBuf;

        let err = storage::StorageError::CreateDir {
            path: PathBuf::from("/Users/secret/.biofocus/data"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let status = build_status(Err(err.public_message()));
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
    fn status_json_has_no_observation_fields() {
        let status = build_status(Ok(()));
        let json = serde_json::to_value(&status).expect("serialize status");
        let obj = json.as_object().expect("object");
        assert!(obj.contains_key("version"));
        assert!(obj.contains_key("dbStatus"));
        assert!(!obj.contains_key("observations"));
        assert!(!obj.contains_key("payload"));
        assert!(!obj.contains_key("hrv"));
    }

    #[test]
    fn pairing_info_json_has_no_paths() {
        let info = build_pairing_info("abc123deadbeef".into(), false).expect("qr");
        let json = serde_json::to_value(&info).expect("serialize");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.get("token").and_then(|v| v.as_str()), Some("abc123deadbeef"));
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
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
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
