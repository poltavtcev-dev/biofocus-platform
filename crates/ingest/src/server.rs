//! Loopback TCP bind and Axum serve loop (idle on accept — no spin).

use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::config::{IngestConfig, INGEST_BIND_HOST};
use crate::error::{IngestError, IngestResult};
use crate::routes::{ingest_router, IngestState};
use runtime::ObservationSender;

/// Binds a TCP listener to `127.0.0.1:<port>` only.
///
/// Port `0` requests an ephemeral free port (useful in tests).
pub async fn bind_loopback(port: u16) -> IngestResult<(TcpListener, SocketAddr)> {
    let addr = SocketAddr::V4(SocketAddrV4::new(INGEST_BIND_HOST, port));
    let listener = TcpListener::bind(addr).await.map_err(|source| IngestError::Bind {
        addr: addr.to_string(),
        source,
    })?;
    let local = listener
        .local_addr()
        .map_err(|source| IngestError::Bind {
            addr: addr.to_string(),
            source,
        })?;
    Ok((listener, local))
}

/// Serves ingest on an already-bound loopback listener until the process ends.
pub async fn serve_listener(
    listener: TcpListener,
    config: IngestConfig,
    tx: ObservationSender,
) -> IngestResult<()> {
    let state = IngestState {
        token: Arc::from(config.token),
        tx,
    };
    let app = ingest_router(state);
    axum::serve(listener, app)
        .await
        .map_err(IngestError::Serve)
}

/// Binds loopback and serves until `shutdown` receives a value.
///
/// Returns the bound [`SocketAddr`] (port may differ when `config.port == 0`).
pub async fn serve_with_shutdown(
    config: IngestConfig,
    tx: ObservationSender,
    shutdown: oneshot::Receiver<()>,
) -> IngestResult<SocketAddr> {
    let (listener, addr) = bind_loopback(config.port).await?;
    let state = IngestState {
        token: Arc::from(config.token),
        tx,
    };
    let app = ingest_router(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(IngestError::Serve)?;

    Ok(addr)
}
