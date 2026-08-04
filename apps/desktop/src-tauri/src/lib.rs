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
//! ## Local ingest HTTP (Phase 2)
//!
//! On startup the host opens the default DB, loads [`ingest::IngestConfig`],
//! spawns the persist worker, and serves loopback ingest (`127.0.0.1:8787`).
//! Companion/debug use `GET /v1/status`; the shell UI still uses IPC `get_status`.
//!
//! See also `docs/09-api.md`.

#![forbid(unsafe_code)]

mod ingest_host;

use std::path::Path;

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

/// Probes a DB path (create + WAL + migrate-on-open). Soft-fail via `Err`.
fn probe_database_at(path: &Path) -> Result<(), String> {
    let _db = storage::Database::open(path).map_err(|err| err.to_string())?;
    Ok(())
}

fn probe_default_database() -> Result<(), String> {
    let path = storage::default_db_path().map_err(|err| err.to_string())?;
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_status, core_ping])
        .build(tauri::generate_context!())?;

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            ingest_host::stop_ingest_host(app_handle);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
}
