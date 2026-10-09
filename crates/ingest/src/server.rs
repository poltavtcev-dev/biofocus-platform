//! TCP bind and Axum serve loop (idle on accept — no spin).
//!
//! Default bind is loopback; LAN bind is opt-in via [`IngestConfig`] env knobs.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use axum_server::tls_rustls::RustlsConfig;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::config::{INGEST_BIND_HOST, IngestConfig};
use crate::error::{IngestError, IngestResult};
use crate::routes::{IngestState, ingest_router};
use crate::tls::{self, IngestTransport};

/// Binds a TCP listener to `host:port`.
///
/// Port `0` requests an ephemeral free port (useful in tests).
pub async fn bind_host(host: Ipv4Addr, port: u16) -> IngestResult<(TcpListener, SocketAddr)> {
    let addr = SocketAddr::V4(SocketAddrV4::new(host, port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| IngestError::Bind {
            addr: addr.to_string(),
            source,
        })?;
    let local = listener.local_addr().map_err(|source| IngestError::Bind {
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
    axum::serve(listener, app).await.map_err(IngestError::Serve)
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
    // Never expose an insecure token on the LAN, whatever path built the config.
    config.validate()?;
    let (listener, addr) = bind_host(config.bind_host, config.port).await?;
    serve_listener_with_shutdown(listener, config.bind_host, state, shutdown).await?;
    Ok(addr)
}

/// Serves `listener` until `shutdown`. Non-loopback binds speak TLS only.
pub async fn serve_listener_with_shutdown(
    listener: TcpListener,
    bind_host: Ipv4Addr,
    state: IngestState,
    shutdown: oneshot::Receiver<()>,
) -> IngestResult<()> {
    let port = listener.local_addr().map_err(|source| IngestError::Bind {
        addr: bind_host.to_string(),
        source,
    })?;
    let state = state.with_bind(bind_host, port.port());
    let app = ingest_router(state);
    match tls::transport_for_bind(bind_host) {
        IngestTransport::PlainHttp => {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown.await;
                })
                .await
                .map_err(IngestError::Serve)?;
        }
        IngestTransport::Tls => {
            let identity = tls::load_or_create_tls_identity()?;
            serve_tls_listener(listener, identity, app, shutdown).await?;
        }
    }
    Ok(())
}

/// Serves TLS on an already-bound listener using `identity` (no disk read).
pub async fn serve_tls_listener(
    listener: TcpListener,
    identity: tls::TlsIdentity,
    app: axum::Router,
    shutdown: oneshot::Receiver<()>,
) -> IngestResult<()> {
    tls::install_ring();
    let rustls = RustlsConfig::from_pem(
        identity.cert_pem.into_bytes(),
        identity.key_pem.into_bytes(),
    )
    .await
    .map_err(|err| IngestError::Tls(err.to_string()))?;
    let std_listener = listener.into_std().map_err(|source| IngestError::Bind {
        addr: "tls".to_owned(),
        source,
    })?;
    std_listener
        .set_nonblocking(true)
        .map_err(|source| IngestError::Bind {
            addr: "tls".to_owned(),
            source,
        })?;
    let handle = axum_server::Handle::new();
    let shutdown_handle = handle.clone();
    tokio::spawn(async move {
        let _ = shutdown.await;
        shutdown_handle.shutdown();
    });
    axum_server::from_tcp_rustls(std_listener, rustls)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .map_err(IngestError::Serve)?;
    Ok(())
}
