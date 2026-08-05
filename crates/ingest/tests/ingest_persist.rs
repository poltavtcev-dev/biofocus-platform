//! Round-trip: HTTP ingest → bounded channel → ObservationRepository (temp DB).

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use bio_spec::{Observation, UnixTimestamp};
use http::header::AUTHORIZATION;
use ingest::{ingest_router, spawn_persist_worker, IngestResponse, IngestState};
use runtime::observation_channel;
use serde_json::json;
use storage::{Database, ObservationRepository};
use tower::ServiceExt;
use uuid::Uuid;

const TOKEN: &str = "test-persist-token";

const ID_A: &str = "0190ecb5-7c2a-7123-8901-23456789abcd";
const ID_B: &str = "0190ecb5-7c2a-7123-8901-23456789abce";

fn observation_json(id: &str, bpm: f64) -> serde_json::Value {
    json!({
        "id": id,
        "timestamp": 1721990400,
        "provider_id": "com.biofocus.applehealth",
        "data_type": "heart_rate",
        "payload": { "bpm": bpm },
        "confidence": 0.98
    })
}

async fn wait_for_id(db_path: &std::path::Path, id: Uuid, timeout: Duration) -> Observation {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        {
            let db = Database::open(db_path).expect("re-open db");
            let repo = ObservationRepository::new(&db);
            if let Some(obs) = repo.get_by_id(id).expect("get_by_id") {
                return obs;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for Observation {id}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn http_ingest_persists_to_sqlite_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let state = IngestState::new(TOKEN, tx.clone());
    let app = ingest_router(state);

    let body = json!([
        observation_json(ID_A, 74.0),
        observation_json(ID_B, 76.0)
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

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let bytes = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .expect("body");
    let parsed: IngestResponse = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(parsed, IngestResponse::queued(2));

    let id_a = Uuid::parse_str(ID_A).expect("uuid");
    let id_b = Uuid::parse_str(ID_B).expect("uuid");
    let obs_a = wait_for_id(&db_path, id_a, Duration::from_secs(2)).await;
    let obs_b = wait_for_id(&db_path, id_b, Duration::from_secs(2)).await;

    assert_eq!(obs_a.data_type, "heart_rate");
    assert_eq!(obs_a.payload["bpm"], json!(74.0));
    assert_eq!(obs_b.payload["bpm"], json!(76.0));

    let db = Database::open(&db_path).expect("list db");
    let repo = ObservationRepository::new(&db);
    let listed = repo
        .list_by_data_type("heart_rate")
        .expect("list");
    assert_eq!(listed.len(), 2);

    let ranged = repo
        .list_by_time_range(
            UnixTimestamp::from_secs(1721990400),
            UnixTimestamp::from_secs(1721990400),
        )
        .expect("range");
    assert_eq!(ranged.len(), 2);

    drop(tx);
    worker.join().expect("persist worker");
}

fn life_event_json(id: &str, kind: &str) -> serde_json::Value {
    json!({
        "id": id,
        "timestamp": 1721990400,
        "provider_id": "com.biofocus.desktop",
        "data_type": "life_event",
        "payload": { "kind": kind, "note": "morning" },
        "confidence": 1.0
    })
}

#[tokio::test]
async fn http_ingest_persists_life_event_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let state = IngestState::new(TOKEN, tx.clone());
    let app = ingest_router(state);

    const LIFE_ID: &str = "0190ecb5-7c2a-7123-8901-23456789abcf";
    let body = json!([life_event_json(LIFE_ID, "coffee")]);

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

    let id = Uuid::parse_str(LIFE_ID).expect("uuid");
    let obs = wait_for_id(&db_path, id, Duration::from_secs(2)).await;
    assert_eq!(obs.data_type, "life_event");
    assert_eq!(obs.payload["kind"], json!("coffee"));
    assert_eq!(obs.payload["note"], json!("morning"));
    assert_eq!(obs.provider_id, "com.biofocus.desktop");

    let db = Database::open(&db_path).expect("list db");
    let repo = ObservationRepository::new(&db);
    let listed = repo.list_by_data_type("life_event").expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, id);

    drop(tx);
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn duplicate_pk_does_not_overwrite_persisted_observation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let state = IngestState::new(TOKEN, tx.clone());
    let app = ingest_router(state);

    let first = json!([observation_json(ID_A, 74.0)]);
    let r1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(first.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(r1.status(), StatusCode::ACCEPTED);

    let id = Uuid::parse_str(ID_A).expect("uuid");
    let original = wait_for_id(&db_path, id, Duration::from_secs(2)).await;
    assert_eq!(original.payload["bpm"], json!(74.0));

    // Same PK, different payload — must not overwrite.
    let dup = json!([observation_json(ID_A, 99.0)]);
    let r2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest")
                .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                .header("content-type", "application/json")
                .body(Body::from(dup.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    // Queued successfully; persist worker rejects duplicate without overwrite.
    assert_eq!(r2.status(), StatusCode::ACCEPTED);

    // Give the worker time to attempt (and reject) the duplicate insert.
    tokio::time::sleep(Duration::from_millis(100)).await;

    let db = Database::open(&db_path).expect("re-open");
    let repo = ObservationRepository::new(&db);
    let stored = repo.get_by_id(id).expect("get").expect("present");
    assert_eq!(
        stored.payload["bpm"],
        json!(74.0),
        "duplicate must not overwrite"
    );
    let listed = repo.list_by_data_type("heart_rate").expect("list");
    assert_eq!(listed.len(), 1);

    drop(tx);
    worker.join().expect("persist worker");
}
