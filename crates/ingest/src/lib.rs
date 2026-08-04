//! Local HTTP ingest for BioFocus (`POST /v1/ingest`).
//!
//! Binds **only** to loopback (`127.0.0.1`). Accepts a JSON array of
//! [`bio_spec::Observation`], authenticates via Bearer pairing token, and
//! enqueues into a bounded Observation channel. Persistence of Observations
//! is out of scope until P2-E1-T3 (`docs/SPRINT_ROADMAP.md`).

#![cfg_attr(not(test), forbid(unsafe_code))]

mod auth;
mod config;
mod error;
mod routes;
mod server;
mod token;

pub use config::{
    IngestConfig, DEFAULT_INGEST_PORT, DEFAULT_SKELETON_TOKEN, DEFAULT_TEST_TOKEN, INGEST_BIND_HOST,
    INGEST_TOKEN_ENV,
};
pub use error::{IngestError, IngestResult};
pub use routes::{ingest_router, IngestResponse, IngestState};
pub use server::{bind_loopback, serve_listener, serve_with_shutdown};
pub use token::{
    default_pairing_token_path, generate_pairing_token, load_or_create_pairing_token,
    resolve_ingest_token, BIOFOCUS_DIR, BIOFOCUS_HOME_ENV, PAIRING_TOKEN_FILE,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "ingest";
