//! Local HTTP ingest for BioFocus (`POST /v1/ingest`).
//!
//! Default bind is loopback (`127.0.0.1`). LAN-reachable bind is **opt-in**
//! (`BIOFOCUS_INGEST_LAN=1` and/or `BIOFOCUS_INGEST_BIND_HOST`). Accepts a JSON
//! array of [`bio_spec::Observation`], authenticates via Bearer pairing token,
//! enqueues into a bounded Observation channel, and (via [`spawn_persist_worker`])
//! appends to SQLite through [`storage::ObservationRepository`].

#![cfg_attr(not(test), forbid(unsafe_code))]

mod advertise;
mod auth;
mod config;
mod error;
mod ingest_prefs;
mod persist;
mod routes;
mod server;
mod status;
mod test_lock;
mod token;

pub use advertise::{AdvertiseInfo, BindMode, http_base_url};
pub use config::{
    DEFAULT_INGEST_PORT, INGEST_BIND_HOST, INGEST_BIND_HOST_ENV, INGEST_LAN_BIND_HOST,
    INGEST_LAN_ENV, INGEST_TOKEN_ENV, IngestConfig, MIN_LAN_TOKEN_LEN, resolve_bind_host,
};
pub use error::{IngestError, IngestResult};
pub use ingest_prefs::lan_preference_overridden_by_env;
pub use ingest_prefs::{
    INGEST_LAN_PREFS_FILE, ingest_lan_prefs_path, read_persisted_lan_enabled,
    write_persisted_lan_enabled,
};
pub use persist::spawn_persist_worker;
pub use routes::{DbProbe, IngestResponse, IngestState, QueuePressureBody, ingest_router};
pub use server::{bind_host, bind_loopback, serve_listener, serve_with_shutdown};
pub use status::{StatusResponse, probe_db_at};
pub use token::{
    BIOFOCUS_DIR, BIOFOCUS_HOME_ENV, PAIRING_TOKEN_FILE, default_pairing_token_path,
    generate_pairing_token, load_or_create_pairing_token, resolve_ingest_token,
};

/// Crate identity used by dependents and status payloads.
pub const CRATE_NAME: &str = "ingest";
