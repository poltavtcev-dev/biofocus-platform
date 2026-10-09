//! Bounded Observation ingress channel (no unbounded buffers).

use bio_spec::Observation;
use tokio::sync::mpsc;

use crate::{RuntimeError, RuntimeResult};

/// Default bounded buffer for Observation ingress (`docs/15-engineering-principles.md`).
pub const DEFAULT_OBSERVATION_BUFFER: usize = 1024;

/// Sender half of the Observation ingress channel.
pub type ObservationSender = mpsc::Sender<Observation>;

/// Receiver half of the Observation ingress channel.
pub type ObservationReceiver = mpsc::Receiver<Observation>;

/// Creates a **bounded** Observation channel.
///
/// Capacity must be `> 0`. Prefer [`DEFAULT_OBSERVATION_BUFFER`] unless a caller
/// has a measured reason to change it.
pub fn observation_channel(
    capacity: usize,
) -> RuntimeResult<(ObservationSender, ObservationReceiver)> {
    if capacity == 0 {
        return Err(RuntimeError::InvalidChannelCapacity);
    }
    Ok(mpsc::channel(capacity))
}
