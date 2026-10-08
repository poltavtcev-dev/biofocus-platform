//! LAN ingest speaks TLS. A wrong certificate fingerprint fails the handshake.
//! Plain HTTP stays on loopback.

use std::net::Ipv4Addr;
use std::time::Duration;

use ingest::{
    INGEST_LAN_BIND_HOST, IngestConfig, IngestState, IngestTransport, bind_host, bind_loopback,
    generate_tls_identity, ingest_router, pinned_client_config, serve_listener_with_shutdown,
    serve_tls_listener, transport_for_bind,
};
use runtime::observation_channel;
use tokio::sync::oneshot;

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn loopback_serve_stays_plain_http() {
    assert_eq!(
        transport_for_bind(Ipv4Addr::LOCALHOST),
        IngestTransport::PlainHttp
    );
    let (tx, _rx) = observation_channel(4).expect("channel");
    let (listener, addr) = bind_loopback(0).await.expect("bind");
    let state = IngestState::new(TOKEN, tx);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        serve_listener_with_shutdown(listener, Ipv4Addr::LOCALHOST, state, shutdown_rx).await
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client");
    let response = client
        .get(format!("http://127.0.0.1:{}/v1/status", addr.port()))
        .send()
        .await
        .expect("plain http on loopback");
    assert!(response.status().is_success());

    let _ = shutdown_tx.send(());
    server.abort();
}

#[tokio::test]
async fn lan_handshake_accepts_the_pin_and_rejects_a_mismatch() {
    let identity = generate_tls_identity().expect("cert");
    let fingerprint = identity.fingerprint_hex.clone();
    let (tx, _rx) = observation_channel(4).expect("channel");
    let (listener, addr) = bind_host(INGEST_LAN_BIND_HOST, 0).await.expect("bind");
    let port = addr.port();
    let state = IngestState::new(TOKEN, tx).with_bind(INGEST_LAN_BIND_HOST, port);
    let app = ingest_router(state);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server =
        tokio::spawn(async move { serve_tls_listener(listener, identity, app, shutdown_rx).await });
    tokio::time::sleep(Duration::from_millis(80)).await;

    let plain = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("plain");
    let plain_result = plain
        .get(format!("http://127.0.0.1:{port}/v1/status"))
        .send()
        .await;
    assert!(
        plain_result.is_err(),
        "LAN must not answer plain HTTP, got {plain_result:?}"
    );

    let pinned = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .use_preconfigured_tls(pinned_client_config(&fingerprint).expect("pin config"))
        .build()
        .expect("pinned client");
    let ok = pinned
        .get(format!("https://127.0.0.1:{port}/v1/status"))
        .send()
        .await
        .expect("pinned handshake");
    assert!(ok.status().is_success());

    let wrong = "ab".repeat(32);
    let mismatch = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .use_preconfigured_tls(pinned_client_config(&wrong).expect("wrong pin"))
        .build()
        .expect("mismatch client");
    let denied = mismatch
        .get(format!("https://127.0.0.1:{port}/v1/status"))
        .send()
        .await;
    assert!(
        denied.is_err(),
        "a different fingerprint must fail the handshake, got {denied:?}"
    );

    let _ = shutdown_tx.send(());
    server.abort();
}

#[test]
fn lan_config_still_refuses_a_short_token() {
    let mut cfg = IngestConfig::with_token("short");
    cfg.bind_host = INGEST_LAN_BIND_HOST;
    assert!(cfg.validate().is_err());
}
