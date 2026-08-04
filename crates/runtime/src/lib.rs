//! Core async runtime host for BioFocus (no Tauri dependency).
//!
//! Provides:
//! - multi-thread Tokio executor ([`CoreRuntime`])
//! - bounded Observation ingress channel ([`observation_channel`])
//! - tracing subscriber bootstrap ([`init_tracing`])
//! - idle-safe Feature Worker ([`spawn_feature_worker`]) — pipeline wire (P3-E1-T4)
//!
//! HTTP ingest lives in the `ingest` crate (Phase 2). OS collectors are Phase 2+
//! (`docs/03-runtime.md`, `docs/15-engineering-principles.md`).

#![forbid(unsafe_code)]

mod channel;
mod error;
mod feature_worker;
mod host;
mod tracing_init;

pub use channel::{
    observation_channel, ObservationReceiver, ObservationSender, DEFAULT_OBSERVATION_BUFFER,
};
pub use error::{RuntimeError, RuntimeResult};
pub use feature_worker::{
    feature_source_error, spawn_feature_worker, validate_feature_worker_config, FeatureHook,
    FeatureWorkerConfig, FeatureWorkerHandle, NoopFeatureHook, ObservationSource,
    DEFAULT_FEATURE_BATCH_LIMIT, DEFAULT_FEATURE_POLL_INTERVAL,
};
pub use host::{CoreRuntime, RuntimeConfig};
pub use tracing_init::init_tracing;

pub use bio_spec::{Observation, CRATE_NAME as SPEC_CRATE_NAME};

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "runtime";
