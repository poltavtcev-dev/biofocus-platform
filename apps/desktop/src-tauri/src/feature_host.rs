//! Feature Worker lifecycle owned by the desktop host (P3-E1-T4 / P3-E3-T2).
//!
//! Startup: open default DB (separate WAL connection) → cursor at tip →
//! [`runtime::spawn_feature_worker`] with SQLite source + catalog alert hook.
//! Shutdown stops the worker with the app (idle freeze after stop).

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{
    UnixTimestamp, DATA_TYPE_HRV, DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_RESPIRATORY_RATE,
    DATA_TYPE_RESTING_HEART_RATE, DATA_TYPE_SLEEPING_WRIST_TEMPERATURE, DATA_TYPE_SLEEP_INTERVAL,
};
use pipeline::SourcePriority;
use runtime::{
    feature_source_error, spawn_feature_worker, FeatureWorkerConfig, FeatureWorkerHandle,
    Observation, ObservationSource, DEFAULT_FEATURE_BATCH_LIMIT,
};
use storage::{Database, ObservationRepository};
use tauri::{AppHandle, Manager, Runtime};
use tracing::{error, info, warn};

use crate::alert_state::{AlertState, CatalogAlertHook, SnapshotState};

/// Managed handle so Tauri exit can stop the Feature Worker.
pub struct FeatureHost {
    worker: Mutex<Option<FeatureWorkerHandle>>,
}

impl FeatureHost {
    /// Stops the worker thread (no further source polls after join).
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.worker.lock() {
            if let Some(handle) = guard.take() {
                handle.stop();
                info!("feature worker stopped with desktop host");
            }
        }
    }
}

/// SQLite-backed Observation source for the Feature Worker.
///
/// Starts at the current max `(created_at, id)` tip so historical backlog is
/// not replayed on every app launch. Polls
/// [`ObservationRepository::list_after_created_cursor`] — empty → worker sleeps.
struct SqliteObservationSource {
    db: Database,
    after_created_at: i64,
    after_id: String,
    batch_limit: usize,
}

impl SqliteObservationSource {
    fn open_at_tip(db: Database) -> Result<Self, runtime::RuntimeError> {
        let repo = ObservationRepository::new(&db);
        let (after_created_at, after_id) = match repo.max_created_cursor() {
            Ok(Some((ts, id))) => (ts, id),
            Ok(None) => (0, String::new()),
            Err(err) => {
                return Err(feature_source_error(err.public_message()));
            }
        };
        Ok(Self {
            db,
            after_created_at,
            after_id,
            batch_limit: DEFAULT_FEATURE_BATCH_LIMIT,
        })
    }
}

impl ObservationSource for SqliteObservationSource {
    fn poll_new(&mut self) -> Result<Vec<Observation>, runtime::RuntimeError> {
        let repo = ObservationRepository::new(&self.db);
        let page = repo
            .list_after_created_cursor(self.after_created_at, &self.after_id, self.batch_limit)
            .map_err(|err| feature_source_error(err.public_message()))?;

        if let Some(last) = page.last() {
            self.after_created_at = last.created_at;
            self.after_id = last.observation.id.to_string();
        }

        Ok(page.into_iter().map(|row| row.observation).collect())
    }
}

/// Starts the Feature Worker if the default DB opens. Soft-fails (logs, no panic).
pub fn start_feature_host<R: Runtime>(app: &AppHandle<R>) {
    let db_path = match storage::default_db_path() {
        Ok(path) => path,
        Err(err) => {
            error!(
                error = %err.public_message(),
                "default DB path unavailable; feature worker not started"
            );
            return;
        }
    };

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(err) => {
            error!(
                error = %err.public_message(),
                "failed to open DB for feature worker; not started"
            );
            return;
        }
    };

    let history = load_vital_history(&db);
    let source = match SqliteObservationSource::open_at_tip(db) {
        Ok(source) => source,
        Err(err) => {
            error!(error = %err, "feature worker source init failed");
            return;
        }
    };

    let alert_state = AlertState::new();
    let snapshot_state = SnapshotState::new();
    let mut hook = CatalogAlertHook::new(alert_state.share(), snapshot_state.share());
    hook.seed_vitals(history);
    let handle = spawn_feature_worker(source, hook, FeatureWorkerConfig::default());
    info!("feature worker armed (poll ≥1s when idle; pipeline → catalog alert hook)");

    app.manage(alert_state);
    app.manage(snapshot_state);
    app.manage(FeatureHost {
        worker: Mutex::new(Some(handle)),
    });
}

const VITAL_HISTORY_SECS: i64 = 28 * 24 * 60 * 60;

fn load_vital_history(db: &Database) -> Vec<Observation> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    if now <= 0 {
        return Vec::new();
    }
    let start = UnixTimestamp::from_secs(now.saturating_sub(VITAL_HISTORY_SECS));
    let end = UnixTimestamp::from_secs(now);
    let repo = ObservationRepository::new(db);
    let types = [
        DATA_TYPE_RESTING_HEART_RATE,
        DATA_TYPE_HRV,
        DATA_TYPE_SLEEP_INTERVAL,
        DATA_TYPE_OXYGEN_SATURATION,
        DATA_TYPE_RESPIRATORY_RATE,
        DATA_TYPE_SLEEPING_WRIST_TEMPERATURE,
    ];
    let mut all = Vec::new();
    for data_type in types {
        match repo.list_by_data_type_in_range(data_type, start, end) {
            Ok(mut rows) => all.append(&mut rows),
            Err(err) => warn!(
                error = %err.public_message(),
                data_type,
                "vital history load skipped"
            ),
        }
    }
    pipeline::select_sources(all, &SourcePriority::load_installed())
}

/// Stops the Feature Worker if it was started (no-op when manage state absent).
pub fn stop_feature_host<R: Runtime>(app: &AppHandle<R>) {
    if let Some(host) = app.try_state::<FeatureHost>() {
        host.shutdown();
    } else {
        warn!("feature host state absent at stop (worker never started)");
    }
}
