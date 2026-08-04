//! Integration tests: loopback bind, auth reject, 202 happy path, bad JSON.

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bio_spec::Observation;
use http::header::AUTHORIZATION;
use ingest::{
    bind_loopback, ingest_router, IngestConfig, IngestResponse, IngestState, QueuePressureBody,
    StatusResponse, INGEST_BIND_HOST,
};
use runtime::observation_channel;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

const TOKEN: &str = "test-ingest-token";

fn sample_observation_json() -> serde_json::Value {
    json!({
        "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
        "timestamp": 1721990400,
        "provider_id": "com.biofocus.applehealth",
        "data_type": "heart_rate",
        "payload": { "bpm": 74.0 },
        "confidence": 0.98
    })
}

fn test_state(capacity: usize) -> (IngestState, runtime::ObservationReceiver) {
    let (tx, rx) = observation_channel(capacity).expect("channel");
    (IngestState::new(TOKEN, tx), rx)
}

#[tokio::test]
async fn bind_loopback_is_localhost_only() {
    let (listener, addr) = bind_loopback(0).await.expect("bind ephemeral");
    assert_eq!(addr.ip(), std::net::IpAddr::V4(INGEST_BIND_HOST));
    assert_ne!(addr.port(), 0);
    drop(listener);
}

#[tokio::test]
async fn post_ingest_rejects_missing_token() {
    let (state, _rx) = test_state(8);
    let app = ingest_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_ingest_rejects_wrong_token() {
    let (state, _rx) = test_state(8);
    let app = ingest_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, "Bearer wrong-token")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_ingest_rejects_invalid_json() {
    let (state, _rx) = test_state(8);
    let app = ingest_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from("{not-json"))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_ingest_rejects_domain_validation_failure() {
    let (state, _rx) = test_state(8);
    let app = ingest_router(state);

    let bad = json!([{
        "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
        "timestamp": 1721990400,
        "provider_id": "com.biofocus.applehealth",
        "data_type": "heart_rate",
        "payload": {},
        "confidence": 1.5
    }]);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(bad.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_ingest_returns_202_and_enqueues() {
    let (state, mut rx) = test_state(8);
    let app = ingest_router(state);

    let body = json!([sample_observation_json()]);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: IngestResponse = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed, IngestResponse::queued(1));

    let obs = rx.try_recv().expect("enqueued observation");
    assert_eq!(
        obs.id,
        Uuid::parse_str("0190ecb5-7c2a-7123-8901-23456789abcd").expect("uuid")
    );
    assert_eq!(obs.data_type, "heart_rate");
}

#[tokio::test]
async fn post_ingest_mid_batch_queue_full_returns_503_with_counts() {
    let (state, mut rx) = test_state(1);
    let app = ingest_router(state);

    let a = json!({
        "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
        "timestamp": 1721990400,
        "provider_id": "com.biofocus.applehealth",
        "data_type": "heart_rate",
        "payload": { "bpm": 74.0 },
        "confidence": 0.98
    });
    let b = json!({
        "id": "0190ecb5-7c2a-7123-8901-23456789abce",
        "timestamp": 1721990401,
        "provider_id": "com.biofocus.applehealth",
        "data_type": "heart_rate",
        "payload": { "bpm": 75.0 },
        "confidence": 0.97
    });
    let body = json!([a, b]);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: QueuePressureBody = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed, QueuePressureBody::queue_full(1, 1));

    let first = rx.try_recv().expect("first enqueued");
    assert_eq!(
        first.id,
        Uuid::parse_str("0190ecb5-7c2a-7123-8901-23456789abcd").expect("uuid")
    );
    assert!(rx.try_recv().is_err(), "second item must not be enqueued");
}

#[tokio::test]
async fn post_ingest_queue_full_on_first_item_accepted_zero() {
    let (tx, mut rx) = observation_channel(1).expect("channel");
    // Saturate so the next try_send fails immediately.
    tx.try_send(serde_json::from_value(sample_observation_json()).expect("obs"))
        .expect("prefill");

    let state = IngestState::new(TOKEN, tx);
    let app = ingest_router(state);

    let body = json!([
        {
            "id": "0190ecb5-7c2a-7123-8901-23456789abcf",
            "timestamp": 1721990500,
            "provider_id": "com.biofocus.applehealth",
            "data_type": "heart_rate",
            "payload": { "bpm": 80.0 },
            "confidence": 0.9
        },
        {
            "id": "0190ecb5-7c2a-7123-8901-23456789abd0",
            "timestamp": 1721990501,
            "provider_id": "com.biofocus.applehealth",
            "data_type": "heart_rate",
            "payload": { "bpm": 81.0 },
            "confidence": 0.9
        }
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: QueuePressureBody = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed, QueuePressureBody::queue_full(0, 2));

    // Prefill still present; neither new item accepted.
    let _ = rx.try_recv().expect("prefill");
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn live_server_accepts_over_loopback_http() {
    let (tx, mut rx) = observation_channel(8).expect("channel");
    let (listener, addr) = bind_loopback(0).await.expect("bind");
    assert_eq!(addr.ip(), std::net::IpAddr::V4(INGEST_BIND_HOST));

    let state = IngestState::new(TOKEN, tx);
    let app = ingest_router(state);

    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    // Give the accept loop a tick (event-driven; no busy spin in server).
    tokio::time::sleep(Duration::from_millis(20)).await;

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/v1/ingest");
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
        .json(&json!([sample_observation_json()]))
        .send()
        .await
        .expect("http");

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body: IngestResponse = response.json().await.expect("json");
    assert_eq!(body, IngestResponse::queued(1));

    let obs: Observation = rx.recv().await.expect("queued");
    assert_eq!(obs.provider_id, "com.biofocus.applehealth");

    server.abort();
}

#[test]
fn default_config_documents_port_and_test_token() {
    let cfg = IngestConfig::default();
    assert_eq!(cfg.port, ingest::DEFAULT_INGEST_PORT);
    assert_eq!(cfg.token, ingest::DEFAULT_TEST_TOKEN);
    assert_eq!(ingest::DEFAULT_SKELETON_TOKEN, ingest::DEFAULT_TEST_TOKEN);
}

#[tokio::test]
async fn persisted_token_is_accepted_wrong_token_rejected() {
    let dir = std::env::temp_dir().join(format!(
        "biofocus-ingest-http-token-{}-{}",
        std::process::id(),
        Uuid::now_v7()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(ingest::PAIRING_TOKEN_FILE);
    let token = ingest::load_or_create_pairing_token(&path).expect("token");

    let (tx, _rx) = observation_channel(8).expect("channel");
    let state = IngestState::new(token.as_str(), tx);
    let app = ingest_router(state);

    let ok = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ok.status(), StatusCode::ACCEPTED);

    let bad = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, "Bearer wrong-token")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn get_status_ok_shape_has_no_observation_fields() {
    let (state, _rx) = test_state(8);
    let app = ingest_router(state.with_version("9.9.9"));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/status")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: StatusResponse = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed.version, "9.9.9");
    assert_eq!(parsed.db_status, "ok");
    assert_eq!(parsed.db_error, None);

    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("value");
    let obj = value.as_object().expect("object");
    assert!(obj.contains_key("version"));
    assert!(obj.contains_key("db_status"));
    assert!(!obj.contains_key("observations"));
    assert!(!obj.contains_key("payload"));
    assert!(!obj.contains_key("hrv"));
    assert!(!obj.contains_key("dbStatus")); // HTTP uses snake_case
}

#[tokio::test]
async fn get_status_maps_db_probe_error_soft() {
    let (tx, _rx) = observation_channel(8).expect("channel");
    let state = IngestState::new(TOKEN, tx)
        .with_version("0.1.0")
        .with_db_probe(|| Err("simulated open failure".into()));
    let app = ingest_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/status")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: StatusResponse = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed.db_status, "error");
    assert_eq!(parsed.db_error.as_deref(), Some("simulated open failure"));
}

#[tokio::test]
async fn get_status_probes_real_temp_db_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("status_probe.db");
    let (tx, _rx) = observation_channel(8).expect("channel");
    let state = IngestState::new(TOKEN, tx).with_db_path(db_path.clone());
    let app = ingest_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/status")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: StatusResponse = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed.db_status, "ok");
    assert!(db_path.exists(), "probe should create DB via open+migrate");
}

#[test]
fn status_response_unit_mapping() {
    let ok = StatusResponse::from_probe("1.0.0", Ok(()));
    assert_eq!(ok.db_status, "ok");
    assert!(ok.db_error.is_none());

    let err = StatusResponse::from_probe("1.0.0", Err("disk full".into()));
    assert_eq!(err.db_status, "error");
    assert_eq!(err.db_error.as_deref(), Some("disk full"));
}
