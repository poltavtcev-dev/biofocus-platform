//! Companion → ingest: sample heart_rate path + 401 / network handling.

use std::time::Duration;

use companion::{
    sample_heart_rate_observation, CompanionClient, CompanionError, APPLE_HEALTH_PROVIDER_ID,
    HEART_RATE_DATA_TYPE,
};
use ingest::{bind_loopback, ingest_router, IngestState};
use runtime::observation_channel;

const TOKEN: &str = "companion-test-token";

#[test]
fn sample_heart_rate_matches_contract() {
    let obs = sample_heart_rate_observation(74.0, Some("Apple Watch Series 9")).expect("obs");
    assert_eq!(obs.provider_id, APPLE_HEALTH_PROVIDER_ID);
    assert_eq!(obs.data_type, HEART_RATE_DATA_TYPE);
    assert_eq!(obs.payload["bpm"], 74.0);
    assert_eq!(obs.payload["source"], "Apple Watch Series 9");
    assert!(obs.payload.get("rr_intervals").is_none());
}

#[tokio::test]
async fn posts_sample_heart_rate_to_loopback_ingest() {
    let (tx, mut rx) = observation_channel(8).expect("channel");
    let state = IngestState::new(TOKEN, tx);
    let app = ingest_router(state);

    let (listener, addr) = bind_loopback(0).await.expect("bind");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    tokio::time::sleep(Duration::from_millis(20)).await;

    let base = format!("http://{addr}");
    let client = CompanionClient::new(&base, TOKEN).expect("client");
    let resp = client
        .post_sample_heart_rate(76.0, Some("test-source"))
        .await
        .expect("post");
    assert_eq!(resp.count, 1);

    let obs = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("obs");
    assert_eq!(obs.data_type, HEART_RATE_DATA_TYPE);
    assert_eq!(obs.payload["bpm"], 76.0);

    server.abort();
}

#[tokio::test]
async fn wrong_token_is_unauthorized_not_silent() {
    let (tx, _rx) = observation_channel(4).expect("channel");
    let state = IngestState::new(TOKEN, tx);
    let app = ingest_router(state);

    let (listener, addr) = bind_loopback(0).await.expect("bind");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    tokio::time::sleep(Duration::from_millis(20)).await;

    let base = format!("http://{addr}");
    let client = CompanionClient::new(&base, "wrong-token").expect("client");
    let err = client
        .post_sample_heart_rate(70.0, None)
        .await
        .expect_err("must fail");
    assert!(
        matches!(err, CompanionError::Unauthorized),
        "expected Unauthorized, got {err:?}"
    );

    server.abort();
}

#[tokio::test]
async fn connection_refused_surfaces_as_network_error() {
    let (listener, addr) = bind_loopback(0).await.expect("bind");
    let port = addr.port();
    drop(listener);

    let base = format!("http://127.0.0.1:{port}");
    let client = CompanionClient::new(&base, TOKEN).expect("client");
    let err = client
        .post_sample_heart_rate(72.0, None)
        .await
        .expect_err("must fail");
    assert!(
        matches!(err, CompanionError::Network(_)),
        "expected Network, got {err:?}"
    );
}
