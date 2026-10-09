//! Local ingest + collector lifecycle owned by the desktop host.
//!
//! Startup: open default DB → [`IngestConfig::load`] → Observation channel →
//! persist worker → Axum serve (default loopback; LAN opt-in via env) →
//! active window collector (same `tx`).
//! Opt-in input aggregates when `BIOFOCUS_INPUT_AGGREGATES=1`.
//! Opt-in local Calendar (ICS) when `BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS`.
//! Opt-in Browser categories when `BIOFOCUS_BROWSER_CATEGORIES=1`.
//! Opt-in Now Playing ambient when `BIOFOCUS_NOW_PLAYING=1`.
//! Opt-in Git activity when `BIOFOCUS_GIT_ACTIVITY=1`.
//! Opt-in ambient light when `BIOFOCUS_AMBIENT_LIGHT=1`.
//! Opt-in notification events when `BIOFOCUS_NOTIFICATION_EVENTS=1`.
//! Shutdown stops collectors, then accept loop, then joins the worker.

use std::sync::Mutex;
use std::time::Duration;

use ingest::{IngestConfig, IngestState, serve_with_shutdown, spawn_persist_worker};
use macos_collector::{
    ActiveWindowPlugin, AmbientLightPlugin, BrowserCategoryPlugin, CalendarPlugin,
    GitActivityPlugin, KeystrokeAggregatePlugin, NotificationPlugin, NowPlayingPlugin,
    ambient_light_enabled, browser_categories_enabled, calendar_enabled,
    calendar_ics_path_from_env, git_activity_enabled, input_aggregates_enabled,
    notification_events_enabled, now_playing_enabled,
};
use plugin_sdk::BioFocusPlugin;
use runtime::{DEFAULT_OBSERVATION_BUFFER, observation_channel};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// What the running ingest listener actually is (vs. what prefs say for next launch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngestRunState {
    /// Startup has not reached the bind step yet.
    NotStarted,
    /// Listener requested on this host (loopback or LAN).
    Running { bind_host: std::net::Ipv4Addr },
    /// Ingest did not start; short UI-safe reason (no paths / secrets).
    Failed { reason: String },
}

static RUN_STATE: Mutex<IngestRunState> = Mutex::new(IngestRunState::NotStarted);

fn set_run_state(state: IngestRunState) {
    if let Ok(mut guard) = RUN_STATE.lock() {
        *guard = state;
    }
}

/// Current ingest run state (for pairing / status IPC).
pub fn ingest_run_state() -> IngestRunState {
    RUN_STATE
        .lock()
        .map(|g| g.clone())
        .unwrap_or(IngestRunState::NotStarted)
}

/// Managed handle so the Tauri exit path can stop accept + persist worker.
pub struct IngestHost {
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    server_done: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
    collector: Mutex<Option<Arc<ActiveWindowPlugin>>>,
    input_collector: Mutex<Option<Arc<KeystrokeAggregatePlugin>>>,
    calendar_collector: Mutex<Option<Arc<CalendarPlugin>>>,
    browser_collector: Mutex<Option<Arc<BrowserCategoryPlugin>>>,
    now_playing_collector: Mutex<Option<Arc<NowPlayingPlugin>>>,
    git_activity_collector: Mutex<Option<Arc<GitActivityPlugin>>>,
    ambient_light_collector: Mutex<Option<Arc<AmbientLightPlugin>>>,
    notification_collector: Mutex<Option<Arc<NotificationPlugin>>>,
}

impl IngestHost {
    /// Signals graceful HTTP shutdown, waits briefly for the server task, joins worker.
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.notification_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("notification_event collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "notification_event collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.ambient_light_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("ambient_light collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "ambient_light collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.git_activity_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("git_activity collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "git_activity collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.now_playing_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("now_playing collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "now_playing collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.browser_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("browser category collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "browser category collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.calendar_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("calendar collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "calendar collector stop failed"),
                }
            }
        }

        if let Ok(mut guard) = self.input_collector.lock() {
            if let Some(plugin) = guard.take() {
                match tauri::async_runtime::block_on(plugin.stop_stream()) {
                    Ok(()) => info!("keystroke aggregate collector stopped"),
                    Err(plugin_sdk::PluginError::NotRunning) => {}
                    Err(err) => warn!(error = %err, "keystroke aggregate collector stop failed"),
                }
            }
        }

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

/// Starts ingest + persist if config/DB allow. Soft-fails (logs, no panic).
///
/// Default bind is loopback; set `BIOFOCUS_INGEST_LAN=1` (or bind-host override)
/// for LAN-reachable dogfood. Bearer pairing token still required.
///
/// Menubar UI continues to use IPC `get_status`; HTTP `/v1/status` is for companion/debug.
pub fn start_ingest_host<R: Runtime>(app: &AppHandle<R>) {
    let config = match IngestConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!(error = %err, "IngestConfig::load failed; ingest HTTP not started");
            let reason = match err {
                ingest::IngestError::InsecureToken { reason } => {
                    format!("Phone sync is off: {reason}.")
                }
                ingest::IngestError::InvalidBindHost { value } => {
                    format!("Phone sync is off: invalid BIOFOCUS_INGEST_BIND_HOST ({value}).")
                }
                _ => "Phone sync is off: could not load ingest settings.".to_owned(),
            };
            set_run_state(IngestRunState::Failed { reason });
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
    let bind_host = config.bind_host;
    let lan_bind = config.is_lan_bind();
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
        .with_db_path(db_path)
        .with_bind(config.bind_host, config.port);
    ingest::install_live_token(state.token.clone());
    ingest::publish_companion_status(state.companion_status.clone());

    set_run_state(IngestRunState::Running { bind_host });
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (done_tx, done_rx) = std::sync::mpsc::channel();

    tauri::async_runtime::spawn(async move {
        match serve_with_shutdown(config, state, shutdown_rx).await {
            Ok(addr) => {
                info!(%addr, "ingest HTTP stopped after graceful shutdown");
            }
            Err(err) => {
                error!(error = %err, port, "ingest HTTP serve failed");
                let reason = match err {
                    ingest::IngestError::Bind { .. } => format!(
                        "Синхронизация с телефоном выключена: порт {port} уже занят (не запущен ли ещё один BioFocus?)."
                    ),
                    _ => "Phone sync stopped unexpectedly. Restart BioFocus.".to_owned(),
                };
                set_run_state(IngestRunState::Failed { reason });
            }
        }
        let _ = done_tx.send(());
    });

    if lan_bind {
        info!(
            %port,
            %bind_host,
            "ingest HTTP starting with LAN opt-in bind (idle on accept; Bearer required)"
        );
    } else {
        info!(
            %port,
            %bind_host,
            "ingest HTTP starting on loopback (idle on accept)"
        );
    }

    let collector = Arc::new(ActiveWindowPlugin::system_default());
    match tauri::async_runtime::block_on(collector.start_stream(collector_tx.clone())) {
        Ok(()) => info!(
            plugin = collector.id(),
            "active window collector armed (poll ≥1s, emit on change)"
        ),
        Err(err) => {
            warn!(error = %err, "active window collector failed to start");
        }
    }

    let input_collector = if input_aggregates_enabled() {
        let plugin = Arc::new(KeystrokeAggregatePlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "keystroke aggregate collector armed (opt-in, aggregates only)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "keystroke aggregate collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::ENABLE_ENV,
            "keystroke aggregate collector off (set env=1 to enable; requires Accessibility)"
        );
        None
    };

    let calendar_collector = if calendar_enabled() {
        match calendar_ics_path_from_env() {
            Some(path) => {
                let plugin = Arc::new(CalendarPlugin::from_ics_path(path));
                match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
                    Ok(()) => {
                        info!(
                            plugin = plugin.id(),
                            "calendar collector armed (opt-in local ICS; rare poll ≥60s)"
                        );
                        Some(plugin)
                    }
                    Err(err) => {
                        warn!(error = %err, "calendar collector failed to start");
                        None
                    }
                }
            }
            None => {
                warn!(
                    flag = macos_collector::CALENDAR_ENABLE_ENV,
                    path_env = macos_collector::CALENDAR_ICS_ENV,
                    "calendar collector enabled but ICS path missing; not started"
                );
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::CALENDAR_ENABLE_ENV,
            "calendar collector off (set env=1 + BIOFOCUS_CALENDAR_ICS=/path.ics)"
        );
        None
    };

    let browser_collector = if browser_categories_enabled() {
        let plugin = Arc::new(BrowserCategoryPlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "browser category collector armed (opt-in; coarse labels only; poll ≥5s)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "browser category collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::BROWSER_ENABLE_ENV,
            "browser category collector off (set BIOFOCUS_BROWSER_CATEGORIES=1 to enable)"
        );
        None
    };

    let now_playing_collector = if now_playing_enabled() {
        let plugin = Arc::new(NowPlayingPlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "now_playing collector armed (opt-in; coarse media_kind + is_playing; poll ≥5s)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "now_playing collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::NOW_PLAYING_ENABLE_ENV,
            "now_playing collector off (set BIOFOCUS_NOW_PLAYING=1 to enable)"
        );
        None
    };

    let git_activity_collector = if git_activity_enabled() {
        let plugin = Arc::new(GitActivityPlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "git_activity collector armed (opt-in; allowlisted roots; coarse activity_kind only; poll ≥5s)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "git_activity collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::GIT_ACTIVITY_ENABLE_ENV,
            "git_activity collector off (set BIOFOCUS_GIT_ACTIVITY=1 to enable)"
        );
        None
    };

    let ambient_light_collector = if ambient_light_enabled() {
        let plugin = Arc::new(AmbientLightPlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx.clone())) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "ambient_light collector armed (opt-in; coarse light_kind + optional level; poll ≥5s)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "ambient_light collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::AMBIENT_LIGHT_ENABLE_ENV,
            "ambient_light collector off (set BIOFOCUS_AMBIENT_LIGHT=1 to enable)"
        );
        None
    };

    let notification_collector = if notification_events_enabled() {
        let plugin = Arc::new(NotificationPlugin::system_default());
        match tauri::async_runtime::block_on(plugin.start_stream(collector_tx)) {
            Ok(()) => {
                info!(
                    plugin = plugin.id(),
                    "notification_event collector armed (opt-in; coarse count + labels; poll ≥5s)"
                );
                Some(plugin)
            }
            Err(err) => {
                warn!(error = %err, "notification_event collector failed to start");
                None
            }
        }
    } else {
        info!(
            flag = macos_collector::NOTIFICATION_ENABLE_ENV,
            "notification_event collector off (set BIOFOCUS_NOTIFICATION_EVENTS=1 to enable)"
        );
        None
    };

    app.manage(IngestHost {
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
        server_done: Mutex::new(Some(done_rx)),
        worker: Mutex::new(Some(worker)),
        collector: Mutex::new(Some(collector)),
        input_collector: Mutex::new(input_collector),
        calendar_collector: Mutex::new(calendar_collector),
        browser_collector: Mutex::new(browser_collector),
        now_playing_collector: Mutex::new(now_playing_collector),
        git_activity_collector: Mutex::new(git_activity_collector),
        ambient_light_collector: Mutex::new(ambient_light_collector),
        notification_collector: Mutex::new(notification_collector),
    });
}

/// Stops ingest if it was started (no-op when manage state absent).
pub fn stop_ingest_host<R: Runtime>(app: &AppHandle<R>) {
    if let Some(host) = app.try_state::<IngestHost>() {
        host.shutdown();
    }
}
