//! TCP bind and Axum serve loop (idle on accept — no spin).
//!
//! Default bind is loopback; LAN bind is opt-in via [`IngestConfig`] env knobs.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::config::{IngestConfig, INGEST_BIND_HOST};
use crate::error::{IngestError, IngestResult};
use crate::routes::{ingest_router, IngestState};

/// Binds a TCP listener to `host:port`.
///
/// Port `0` requests an ephemeral free port (useful in tests).
pub async fn bind_host(host: Ipv4Addr, port: u16) -> IngestResult<(TcpListener, SocketAddr)> {
    let addr = SocketAddr::V4(SocketAddrV4::new(host, port));
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

/// Binds a TCP listener to `127.0.0.1:<port>` only.
///
/// Port `0` requests an ephemeral free port (useful in tests).
pub async fn bind_loopback(port: u16) -> IngestResult<(TcpListener, SocketAddr)> {
    bind_host(INGEST_BIND_HOST, port).await
}

/// Serves ingest on an already-bound listener until the process ends.
pub async fn serve_listener(listener: TcpListener, state: IngestState) -> IngestResult<()> {
    let app = ingest_router(state);
    axum::serve(listener, app)
        .await
        .map_err(IngestError::Serve)
}

/// Binds per [`IngestConfig::bind_host`] and serves until `shutdown` receives a value.
///
/// Returns the bound [`SocketAddr`] (port may differ when `config.port == 0`).
/// Idle: tokio accept — no busy-spin.
pub async fn serve_with_shutdown(
    config: IngestConfig,
    state: IngestState,
    shutdown: oneshot::Receiver<()>,
) -> IngestResult<SocketAddr> {
    let (listener, addr) = bind_host(config.bind_host, config.port).await?;
    let app = ingest_router(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(IngestError::Serve)?;

    Ok(addr)
}
