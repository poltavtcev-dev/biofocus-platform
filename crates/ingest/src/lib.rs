//! Local HTTP ingest for BioFocus (`POST /v1/ingest`).
//!
//! Default bind is loopback (`127.0.0.1`). LAN-reachable bind is **opt-in**
//! (`BIOFOCUS_INGEST_LAN=1` and/or `BIOFOCUS_INGEST_BIND_HOST`). Accepts a JSON
//! array of [`bio_spec::Observation`], authenticates via Bearer pairing token,
//! enqueues into a bounded Observation channel, and (via [`spawn_persist_worker`])
//! appends to SQLite through [`storage::ObservationRepository`].

#![cfg_attr(not(test), forbid(unsafe_code))]

mod auth;
mod config;
mod error;
mod persist;
mod routes;
mod server;
mod status;
mod token;

pub use config::{
    resolve_bind_host, IngestConfig, DEFAULT_INGEST_PORT, DEFAULT_SKELETON_TOKEN, DEFAULT_TEST_TOKEN,
    INGEST_BIND_HOST, INGEST_BIND_HOST_ENV, INGEST_LAN_BIND_HOST, INGEST_LAN_ENV, INGEST_TOKEN_ENV,
};
pub use error::{IngestError, IngestResult};
pub use persist::spawn_persist_worker;
pub use routes::{ingest_router, DbProbe, IngestResponse, IngestState, QueuePressureBody};
pub use server::{bind_host, bind_loopback, serve_listener, serve_with_shutdown};
pub use status::{probe_db_at, StatusResponse};
pub use token::{
    default_pairing_token_path, generate_pairing_token, load_or_create_pairing_token,
    resolve_ingest_token, BIOFOCUS_DIR, BIOFOCUS_HOME_ENV, PAIRING_TOKEN_FILE,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "ingest";
