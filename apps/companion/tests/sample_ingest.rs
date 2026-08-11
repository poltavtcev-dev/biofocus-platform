//! Companion → ingest: sample heart_rate path + 401 / network handling.

use std::time::Duration;

use companion::{
    sample_active_energy_observation, sample_heart_rate_observation,
    sample_oxygen_saturation_observation, sample_sleep_interval_observation,
    sample_step_count_observation, CompanionClient, CompanionError, APPLE_HEALTH_PROVIDER_ID,
    HEART_RATE_DATA_TYPE,
};
use bio_spec::validate_observation_payload;
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

#[test]
fn adr018_scripted_samples_validate() {
    let steps = sample_step_count_observation(2400, Some(3600)).expect("steps");
    validate_observation_payload(&steps).expect("step_count");
    let energy = sample_active_energy_observation(185.5).expect("energy");
    validate_observation_payload(&energy).expect("active_energy");
    let sleep = sample_sleep_interval_observation(100, 200, Some("asleep")).expect("sleep");
    validate_observation_payload(&sleep).expect("sleep_interval");
    let sleep_no_stage = sample_sleep_interval_observation(100, 200, None).expect("sleep2");
    validate_observation_payload(&sleep_no_stage).expect("sleep without stage");
    let spo2 = sample_oxygen_saturation_observation(97.0).expect("spo2");
    validate_observation_payload(&spo2).expect("oxygen_saturation");
}

#[test]
fn adr018_rejects_bad_spo2_and_sleep() {
    assert!(sample_oxygen_saturation_observation(140.0).is_ok()); // construction ok
    let bad = sample_oxygen_saturation_observation(140.0).expect("built");
    // Observation builds; ingest validator rejects out-of-range
    assert!(validate_observation_payload(&bad).is_err());
    let bad_sleep = sample_sleep_interval_observation(200, 100, None).expect("built");
    assert!(validate_observation_payload(&bad_sleep).is_err());
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
