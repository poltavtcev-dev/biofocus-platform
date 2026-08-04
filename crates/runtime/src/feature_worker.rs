//! Idle-safe Feature Worker: poll new Observations → quality pipeline → hook.
//!
//! Phase 3 (P3-E1-T4): wires Core to `pipeline` stages. Full Feature DAG is a
//! stub hook until P3-E2. No busy-loop — sleeps / waits on stop when idle.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bio_spec::Observation;
use pipeline::{run_quality_pipeline, DedupeState, NormalizedBatch, PipelineError};
use tracing::{debug, error, info, warn};

use crate::{RuntimeError, RuntimeResult};

/// Default poll cadence when the source has nothing new (~1s; not a spin).
pub const DEFAULT_FEATURE_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Default page size when draining SQLite / mock sources.
pub const DEFAULT_FEATURE_BATCH_LIMIT: usize = 256;

/// Pulls Observation batches for the Feature Worker (storage, mock, etc.).
pub trait ObservationSource: Send {
    /// Returns newly available Observations (may be empty = idle).
    ///
    /// Implementations must not busy-spin; the worker sleeps when this is empty.
    fn poll_new(&mut self) -> Result<Vec<Observation>, RuntimeError>;
}

/// Hook after quality pipeline (Feature Engine stub until P3-E2).
pub trait FeatureHook: Send {
    /// Called with a successfully normalized batch (may be empty after skip/dedupe).
    fn on_normalized(&mut self, batch: &NormalizedBatch);
}

/// No-op Feature Engine hook (logs count at debug).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFeatureHook;

impl FeatureHook for NoopFeatureHook {
    fn on_normalized(&mut self, batch: &NormalizedBatch) {
        debug!(
            kept = batch.len(),
            skipped = batch.skipped_count(),
            "feature hook (noop): normalized batch"
        );
    }
}

/// Configuration for [`spawn_feature_worker`].
#[derive(Debug, Clone)]
pub struct FeatureWorkerConfig {
    /// Sleep duration when [`ObservationSource::poll_new`] returns empty.
    pub poll_interval: Duration,
}

impl Default for FeatureWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: DEFAULT_FEATURE_POLL_INTERVAL,
        }
    }
}

/// Handle to stop and join a Feature Worker thread.
pub struct FeatureWorkerHandle {
    stop_tx: Option<mpsc::Sender<()>>,
    join: Option<JoinHandle<()>>,
}

impl FeatureWorkerHandle {
    /// Signals the worker to exit and waits for the thread to finish.
    ///
    /// After return, the worker must not poll the source again (idle freeze).
    pub fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join.take() {
            if let Err(err) = handle.join() {
                error!(?err, "feature worker join failed");
            }
        }
    }
}

impl Drop for FeatureWorkerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

/// Spawns an OS thread that periodically polls `source`, runs
/// [`pipeline::run_quality_pipeline`], and invokes `hook`.
///
/// **Idle:** when there are no new Observations, blocks on
/// [`mpsc::Receiver::recv_timeout`] for `poll_interval` (or until stop).
/// **Stop:** [`FeatureWorkerHandle::stop`] ends the loop; subsequent source
/// polls must not occur.
#[must_use]
pub fn spawn_feature_worker<S, H>(
    mut source: S,
    mut hook: H,
    config: FeatureWorkerConfig,
) -> FeatureWorkerHandle
where
    S: ObservationSource + 'static,
    H: FeatureHook + 'static,
{
    let (stop_tx, stop_rx) = mpsc::channel();
    let interval = config.poll_interval;

    let join = thread::spawn(move || {
        info!(
            poll_ms = interval.as_millis() as u64,
            "feature worker started"
        );
        let mut dedupe = DedupeState::new();

        loop {
            match stop_rx.try_recv() {
                Ok(()) | Err(mpsc::TryRecvError::Disconnected) => break,
                Err(mpsc::TryRecvError::Empty) => {}
            }

            match source.poll_new() {
                Ok(batch) if batch.is_empty() => {
                    match stop_rx.recv_timeout(interval) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    }
                }
                Ok(batch) => {
                    if let Err(err) = process_batch(batch, &mut dedupe, &mut hook) {
                        error!(error = %err, "feature worker pipeline tick failed");
                    }
                }
                Err(err) => {
                    warn!(error = %err, "feature worker source poll failed");
                    match stop_rx.recv_timeout(interval) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    }
                }
            }
        }

        info!("feature worker stopped");
    });

    FeatureWorkerHandle {
        stop_tx: Some(stop_tx),
        join: Some(join),
    }
}

fn process_batch<H: FeatureHook>(
    batch: Vec<Observation>,
    dedupe: &mut DedupeState,
    hook: &mut H,
) -> Result<(), PipelineError> {
    let normalized = run_quality_pipeline(batch, dedupe)?;
    hook.on_normalized(&normalized);
    Ok(())
}

/// Maps a displayable source failure into [`RuntimeError`].
pub fn feature_source_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::FeatureWorkerSource(message.into())
}

/// Validates Feature Worker config (non-zero interval).
pub fn validate_feature_worker_config(config: &FeatureWorkerConfig) -> RuntimeResult<()> {
    if config.poll_interval.is_zero() {
        return Err(RuntimeError::InvalidFeaturePollInterval);
    }
    Ok(())
}
