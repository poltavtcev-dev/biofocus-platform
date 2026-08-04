//! Observation persist worker: bounded channel → [`ObservationRepository`].

use bio_spec::Observation;
use runtime::ObservationReceiver;
use storage::{Database, ObservationRepository, StorageError};
use tracing::{error, info};

/// Spawns a dedicated OS thread that drains the Observation ingress channel and
/// appends each item via [`ObservationRepository::insert`].
///
/// **Idle:** blocks on [`ObservationReceiver::blocking_recv`] (no busy-spin).
/// Stops when all senders are dropped and the channel is drained.
///
/// Duplicate primary keys surface as [`StorageError::DuplicateObservation`]
/// (logged; no overwrite). Other insert failures are logged and skipped.
#[must_use]
pub fn spawn_persist_worker(
    mut rx: ObservationReceiver,
    db: Database,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        info!("observation persist worker started");
        while let Some(observation) = rx.blocking_recv() {
            persist_one(&db, &observation);
        }
        info!("observation persist worker stopped");
    })
}

fn persist_one(db: &Database, observation: &Observation) {
    let repo = ObservationRepository::new(db);
    match repo.insert(observation) {
        Ok(()) => {}
        Err(StorageError::DuplicateObservation { id }) => {
            error!(
                %id,
                "duplicate Observation primary key; insert rejected (no overwrite)"
            );
        }
        Err(err) => {
            error!(
                error = %err,
                id = %observation.id,
                "failed to persist Observation"
            );
        }
    }
}
