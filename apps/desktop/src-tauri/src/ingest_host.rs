//! Local ingest + collector lifecycle owned by the desktop host.
//!
//! Startup: open default DB → [`IngestConfig::load`] → Observation channel →
//! persist worker → loopback Axum serve → active window collector (same `tx`).
//! Shutdown stops collector, then accept loop, then joins the worker.

use std::sync::Mutex;
use std::time::Duration;

use ingest::{serve_with_shutdown, spawn_persist_worker, IngestConfig, IngestState};
use macos_collector::ActiveWindowPlugin;
use plugin_sdk::BioFocusPlugin;
use runtime::{observation_channel, DEFAULT_OBSERVATION_BUFFER};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// Managed handle so the Tauri exit path can stop accept + persist worker.
pub struct IngestHost {
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    server_done: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
    collector: Mutex<Option<Arc<ActiveWindowPlugin>>>,
}

impl IngestHost {
    /// Signals graceful HTTP shutdown, waits briefly for the server task, joins worker.
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("active window collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "active window collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.shutdown_tx.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }

        if let Ok(mut guard) = self.server_done.lock() {
            if let Some(rx) = guard.take() {
                match rx.recv_timeout(Duration::from_secs(3)) {
                    Ok(()) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        warn!("ingest server did not stop within shutdown timeout");
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
                }
            }
        }

        if let Ok(mut guard) = self.worker.lock() {
            if let Some(handle) = guard.take() {
                if let Err(err) = handle.join() {
                    error!(?err, "ingest persist worker join failed");
                }
            }
        }
    }
}

/// Starts loopback ingest + persist if config/DB allow. Soft-fails (logs, no panic).
///
/// Menubar UI continues to use IPC `get_status`; HTTP `/v1/status` is for companion/debug.
pub fn start_ingest_host<R: Runtime>(app: &AppHandle<R>) {
    let config = match IngestConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!(error = %err, "IngestConfig::load failed; ingest HTTP not started");
            return;
        }
    };

    let db_path = match storage::default_db_path() {
        Ok(path) => path,
        Err(err) => {
            error!(error = %err, "default DB path unavailable; ingest HTTP not started");
            return;
        }
    };

    let (tx, mut rx) = match observation_channel(DEFAULT_OBSERVATION_BUFFER) {
        Ok(pair) => pair,
        Err(err) => {
            error!(error = %err, "observation channel create failed; ingest HTTP not started");
            return;
        }
    };

    let port = config.port;
    let collector_tx = tx.clone();

    let worker = match storage::Database::open(&db_path) {
        Ok(db) => spawn_persist_worker(rx, db),
        Err(err) => {
            // Soft-fail: still expose `/v1/status` with db error; drain via blocking_recv
            // (idle, no spin) so POST does not enqueue into a channel with no consumer forever.
            error!(
                error = %err,
                "failed to open default DB for persist; starting discard drain"
            );
            std::thread::spawn(move || {
                while let Some(observation) = rx.blocking_recv() {
                    warn!(
                        id = %observation.id,
                        "dropping Observation; persist DB unavailable at host start"
                    );
                }
            })
        }
    };

    let state = IngestState::new(config.token.clone(), tx)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_db_path(db_path);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel();

    tauri::async_runtime::spawn(async move {
        match serve_with_shutdown(config, state, shutdown_rx).await {
            Ok(addr) => {
                info!(%addr, "ingest HTTP stopped after graceful shutdown");
            }
            Err(err) => {
                error!(error = %err, port, "ingest HTTP serve failed");
            }
        }
        let _ = done_tx.send(());
    });

    info!(%port, host = "127.0.0.1", "ingest HTTP starting on loopback (idle on accept)");

    let collector = Arc::new(ActiveWindowPlugin::system_default());
    match tauri::async_runtime::block_on(collector.start_stream(collector_tx)) {
        Ok(()) => info!(
            plugin = collector.id(),
            "active window collector armed (poll ≥1s, emit on change)"
        ),
        Err(err) => {
            warn!(error = %err, "active window collector failed to start");
        }
    }

    app.manage(IngestHost {
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
        server_done: Mutex::new(Some(done_rx)),
        worker: Mutex::new(Some(worker)),
        collector: Mutex::new(Some(collector)),
    });
}

/// Stops ingest if it was started (no-op when manage state absent).
pub fn stop_ingest_host<R: Runtime>(app: &AppHandle<R>) {
    if let Some(host) = app.try_state::<IngestHost>() {
        host.shutdown();
    }
}
