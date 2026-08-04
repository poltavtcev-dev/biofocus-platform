//! Multi-thread Tokio runtime host.

use std::future::Future;

use tokio::runtime::{Builder, Handle, Runtime};

use crate::{RuntimeError, RuntimeResult};

/// Configuration for [`CoreRuntime`].
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Optional worker thread count. `None` uses Tokio's default (CPU count).
    pub worker_threads: Option<usize>,
    /// Value passed to `thread_name` for Tokio workers.
    pub thread_name: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,
            thread_name: "biofocus-worker".into(),
        }
    }
}

/// Owned multi-thread Tokio runtime for BioFocus Core.
///
/// Intentionally free of Tauri / HTTP ingest dependencies.
pub struct CoreRuntime {
    inner: Runtime,
}

impl CoreRuntime {
    /// Builds a multi-thread Tokio runtime from `config`.
    pub fn try_new(config: &RuntimeConfig) -> RuntimeResult<Self> {
        let mut builder = Builder::new_multi_thread();
        builder.enable_all().thread_name(config.thread_name.clone());

        if let Some(workers) = config.worker_threads {
            if workers == 0 {
                return Err(RuntimeError::RuntimeBuild(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "worker_threads must be greater than 0 when set",
                )));
            }
            builder.worker_threads(workers);
        }

        let inner = builder.build().map_err(RuntimeError::RuntimeBuild)?;
        Ok(Self { inner })
    }

    /// Returns a handle that can spawn tasks from non-runtime threads.
    #[must_use]
    pub fn handle(&self) -> Handle {
        self.inner.handle().clone()
    }

    /// Runs a future to completion on this runtime.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.inner.block_on(future)
    }
}
