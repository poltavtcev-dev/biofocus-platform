//! Feature Worker lifecycle owned by the desktop host (P3-E1-T4).
//!
//! Startup: open default DB (separate WAL connection) → cursor at tip →
//! [`runtime::spawn_feature_worker`] with SQLite source + noop Feature hook.
//! Shutdown stops the worker with the app (idle freeze after stop).

use std::sync::Mutex;

use runtime::{
    feature_source_error, spawn_feature_worker, FeatureWorkerConfig, FeatureWorkerHandle,
    NoopFeatureHook, ObservationSource, DEFAULT_FEATURE_BATCH_LIMIT,
};
use storage::{Database, ObservationRepository};
use tauri::{AppHandle, Manager, Runtime};
use tracing::{error, info, warn};

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
    fn poll_new(&mut self) -> Result<Vec<bio_spec::Observation>, runtime::RuntimeError> {
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

    let source = match SqliteObservationSource::open_at_tip(db) {
        Ok(source) => source,
        Err(err) => {
            error!(error = %err, "feature worker source init failed");
            return;
        }
    };

    let handle = spawn_feature_worker(
        source,
        NoopFeatureHook,
        FeatureWorkerConfig::default(),
    );
    info!("feature worker armed (poll ≥1s when idle; pipeline → noop feature hook)");

    app.manage(FeatureHost {
        worker: Mutex::new(Some(handle)),
    });
}

/// Stops the Feature Worker if it was started (no-op when manage state absent).
pub fn stop_feature_host<R: Runtime>(app: &AppHandle<R>) {
    if let Some(host) = app.try_state::<FeatureHost>() {
        host.shutdown();
    } else {
        warn!("feature host state absent at stop (worker never started)");
    }
}
