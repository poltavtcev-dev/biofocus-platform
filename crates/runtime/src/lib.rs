//! Core async runtime host for BioFocus (no Tauri dependency).
//!
//! Provides:
//! - multi-thread Tokio executor ([`CoreRuntime`])
//! - bounded Observation ingress channel ([`observation_channel`])
//! - tracing subscriber bootstrap ([`init_tracing`])
//!
//! HTTP ingest lives in the `ingest` crate (Phase 2). OS collectors are Phase 2+
//! (`docs/03-runtime.md`, `docs/15-engineering-principles.md`).

#![forbid(unsafe_code)]

mod channel;
mod error;
mod host;
mod tracing_init;

pub use channel::{
    observation_channel, ObservationReceiver, ObservationSender, DEFAULT_OBSERVATION_BUFFER,
};
pub use error::{RuntimeError, RuntimeResult};
pub use host::{CoreRuntime, RuntimeConfig};
pub use tracing_init::init_tracing;

pub use bio_spec::CRATE_NAME as SPEC_CRATE_NAME;

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "runtime";
