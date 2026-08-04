//! Canonical domain types and serialization contracts for BioFocus.
//!
//! Ubiquitous language (`docs/16-glossary.md`):
//! - [`Observation`] — immutable biometric/context fact
//! - [`Signal`] — transient change / anomaly
//! - [`Feature`] — windowed metric with provenance
//! - [`Insight`] — analytical conclusion with evidence
//!
//! Identifiers intended for new Observations use **UUIDv7**.
//! Timestamps are **Unix seconds UTC** (`i64`).

#![forbid(unsafe_code)]

mod error;
mod feature;
mod insight;
mod observation;
mod signal;
mod time;

pub use error::{SpecError, SpecResult};
pub use feature::{Feature, FeatureId, FeatureValue, Provenance};
pub use insight::{EvidenceRef, Insight, InsightId};
pub use observation::{Confidence, DataType, Observation, ObservationId, ProviderId};
pub use signal::{Severity, Signal, SignalId, SignalType};
pub use time::{TimeWindow, UnixTimestamp};

/// Crate identity used by dependents and IPC status payloads.
pub const CRATE_NAME: &str = "bio-spec";
