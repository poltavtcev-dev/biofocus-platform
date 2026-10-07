//! Local SQLite persistence for BioFocus (WAL mode).
//!
//! Phase 1 progress:
//! - P1-E2-T1: open database + WAL / NORMAL pragmas ([`Database`])
//! - P1-E2-T2: migrate `observations` on open ([`Database::migrate`])
//! - P1-E2-T3: [`ObservationRepository`]
//! - P1-E2-T4: expanded storage integration tests

#![forbid(unsafe_code)]

mod clock;
mod db;
mod error;
mod migrate;
mod observation_repo;
mod paths;

pub use db::Database;
pub use error::{StorageError, StorageResult};
pub use migrate::SCHEMA_VERSION;
pub use observation_repo::{
    ObservationCreated, ObservationRepository, ObservationTypeSummary,
};
pub use paths::{default_db_path, BIOFOCUS_DATA_DIR, DEFAULT_DB_FILE_NAME};

pub use bio_spec::CRATE_NAME as SPEC_CRATE_NAME;

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "storage";
