//! Capability plugin SDK for device and context providers.
//!
//! Platform collectors implement [`BioFocusPlugin`] and emit
//! [`bio_spec::Observation`] into the Core Observation channel.

#![forbid(unsafe_code)]

use std::future::Future;

use runtime::ObservationSender;
use thiserror::Error;

/// Crate identity used by dependents.
pub const CRATE_NAME: &str = "plugin-sdk";

/// Declared capability of a plugin (which Observation kinds it can produce).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// Human-readable capability name (e.g. `active_window`).
    pub name: String,
    /// `data_type` values this capability may emit.
    pub data_types: Vec<String>,
}

/// Errors from plugin lifecycle / streaming.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Stream is already running.
    #[error("plugin stream already started")]
    AlreadyStarted,
    /// Stream is not running.
    #[error("plugin stream is not running")]
    NotRunning,
    /// Underlying collector / OS probe failure.
    #[error(transparent)]
    Collector(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Local-first provider that streams Observations into Core.
///
/// Implementations must honour idle DoD: no busy-loop while waiting for OS
/// events or poll ticks; `stop_stream` must end background work promptly.
pub trait BioFocusPlugin: Send + Sync {
    /// Stable plugin id (e.g. `com.biofocus.macos.context`).
    fn id(&self) -> &str;

    /// Display name for logs / diagnostics.
    fn name(&self) -> &str;

    /// Capabilities this plugin advertises.
    fn capabilities(&self) -> Vec<Capability>;

    /// Starts emitting Observations on `tx`. Returns when the stream is armed
    /// (background work may continue until [`Self::stop_stream`]).
    fn start_stream(
        &self,
        tx: ObservationSender,
    ) -> impl Future<Output = Result<(), PluginError>> + Send;

    /// Stops the stream and joins background work (no orphan spin).
    fn stop_stream(&self) -> impl Future<Output = Result<(), PluginError>> + Send;
}
