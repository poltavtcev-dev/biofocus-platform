//! Storage integration tests for ObservationRepository (P1-E2-T4).
//!
//! Covers Integration layer from `/docs/11-testing.md` plus documented edge errors.

use bio_spec::{Observation, SpecError, UnixTimestamp};
use serde_json::json;
use storage::{Database, ObservationRepository, StorageError};
use uuid::Uuid;

fn sample_observation(id: &str, timestamp: i64, data_type: &str) -> Observation {
    Observation::try_new(
        Uuid::parse_str(id).expect("uuid"),
        UnixTimestamp::from_secs(timestamp),
        "com.biofocus.applehealth",
        data_type,
        json!({ "bpm": 74.0, "source": "Apple Watch Series 9" }),
        0.98,
    )
    .expect("valid observation")
}

fn insert_raw_row(
    db: &Database,
    id: &str,
    timestamp: i64,
    data_type: &str,
    payload: &str,
    confidence: f64,
) {
    db.connection()
        .execute(
            "INSERT INTO observations
                (id, timestamp, provider_id, data_type, payload, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                timestamp,
                "com.biofocus.applehealth",
                data_type,
                payload,
                confidence,
                1_721_990_400_i64,
            ],
        )
        .expect("raw sql insert");
}

#[test]
fn insert_then_get_by_id_round_trip() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let obs = sample_observation(
        "0190ecb5-7c2a-7123-8901-23456789abcd",
        1_721_990_400,
        "heart_rate",
    );
    repo.insert(&obs).expect("insert");

    let loaded = repo.get_by_id(obs.id).expect("get").expect("must exist");
    assert_eq!(loaded, obs);

    let created_at: i64 = db
        .connection()
        .query_row(
            "SELECT created_at FROM observations WHERE id = ?1",
            [obs.id.to_string()],
            |row| row.get(0),
        )
        .expect("created_at");
    assert!(created_at > 0);
    assert_ne!(created_at, obs.timestamp.as_secs());
}

#[test]
fn get_by_id_missing_returns_none() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    let id = Uuid::parse_str("0190ecb5-7c2a-7123-8901-23456789abcd").expect("uuid");

    let loaded = repo.get_by_id(id).expect("get");
    assert!(loaded.is_none());
}

#[test]
fn insert_duplicate_pk_returns_explicit_error() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let obs = sample_observation(
        "0190ecb5-7c2a-7123-8901-23456789abcd",
        1_721_990_400,
        "heart_rate",
    );
    repo.insert(&obs).expect("first insert");

    let err = repo.insert(&obs).expect_err("duplicate must fail");
    match err {
        StorageError::DuplicateObservation { id } => assert_eq!(id, obs.id),
        other => panic!("expected DuplicateObservation, got {other:?}"),
    }

    let count: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 1, "duplicate must not overwrite");
}

#[test]
fn list_by_time_range_inclusive_bounds_and_order() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let before = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc0", 99, "heart_rate");
    let start = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc1", 100, "heart_rate");
    let mid = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc2", 150, "hrv");
    let end = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc3", 200, "heart_rate");
    let after = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc4", 201, "heart_rate");

    for obs in [&before, &start, &mid, &end, &after] {
        repo.insert(obs).expect("insert");
    }

    let ranged = repo
        .list_by_time_range(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(200))
        .expect("range");
    assert_eq!(
        ranged,
        vec![start, mid, end],
        "window is inclusive on both ends"
    );
}

#[test]
fn list_by_time_range_empty_when_no_rows_in_window() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let obs = sample_observation("0190ecb5-7c2a-7123-8901-23456789abcd", 500, "heart_rate");
    repo.insert(&obs).expect("insert");

    let ranged = repo
        .list_by_time_range(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(200))
        .expect("range");
    assert!(ranged.is_empty());
}

#[test]
fn list_by_time_range_single_instant_window() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let hit = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc1", 100, "heart_rate");
    let miss = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc2", 101, "heart_rate");
    repo.insert(&hit).expect("insert hit");
    repo.insert(&miss).expect("insert miss");

    let ranged = repo
        .list_by_time_range(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(100))
        .expect("point window");
    assert_eq!(ranged, vec![hit]);
}

#[test]
fn list_by_time_range_end_before_start_is_invalid() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let err = repo
        .list_by_time_range(UnixTimestamp::from_secs(200), UnixTimestamp::from_secs(100))
        .expect_err("end < start");

    match err {
        StorageError::InvalidTimeRange { start, end } => {
            assert_eq!(start, 200);
            assert_eq!(end, 100);
        }
        other => panic!("expected InvalidTimeRange, got {other:?}"),
    }
}

#[test]
fn list_by_data_type_filters_and_empty() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    let a = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc1", 100, "heart_rate");
    let b = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc2", 200, "hrv");
    let c = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc3", 300, "heart_rate");
    repo.insert(&a).expect("insert a");
    repo.insert(&b).expect("insert b");
    repo.insert(&c).expect("insert c");

    let hr = repo.list_by_data_type("heart_rate").expect("by type");
    assert_eq!(hr, vec![a, c]);

    let empty = repo.list_by_data_type("sleep").expect("missing type");
    assert!(empty.is_empty());
}

#[test]
fn observation_try_new_rejects_invalid_confidence() {
    let err = Observation::try_new(
        Uuid::parse_str("0190ecb5-7c2a-7123-8901-23456789abcd").expect("uuid"),
        UnixTimestamp::from_secs(1_721_990_400),
        "com.biofocus.applehealth",
        "heart_rate",
        json!({ "bpm": 74.0 }),
        1.5,
    )
    .expect_err("confidence out of range");

    assert_eq!(err, SpecError::InvalidConfidence(1.5));
}

#[test]
fn get_by_id_invalid_confidence_in_db_maps_to_domain_error() {
    let db = Database::open_in_memory().expect("open");
    let id = "0190ecb5-7c2a-7123-8901-23456789abcd";
    insert_raw_row(&db, id, 1_721_990_400, "heart_rate", r#"{"bpm":74.0}"#, 1.5);

    let repo = ObservationRepository::new(&db);
    let err = repo
        .get_by_id(Uuid::parse_str(id).expect("uuid"))
        .expect_err("invalid confidence must fail read");

    match err {
        StorageError::Domain(SpecError::InvalidConfidence(value)) => {
            assert_eq!(value, 1.5);
        }
        other => panic!("expected Domain(InvalidConfidence), got {other:?}"),
    }
}

#[test]
fn get_by_id_corrupt_payload_json_returns_deserialize_error() {
    let db = Database::open_in_memory().expect("open");
    let id = "0190ecb5-7c2a-7123-8901-23456789abcd";
    insert_raw_row(&db, id, 1_721_990_400, "heart_rate", "{not-json", 0.9);

    let repo = ObservationRepository::new(&db);
    let err = repo
        .get_by_id(Uuid::parse_str(id).expect("uuid"))
        .expect_err("corrupt payload must fail read");

    match err {
        StorageError::PayloadDeserialize { .. } => {}
        other => panic!("expected PayloadDeserialize, got {other:?}"),
    }
}

#[test]
fn list_by_time_range_stops_on_corrupt_payload() {
    let db = Database::open_in_memory().expect("open");
    let good = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc1", 100, "heart_rate");
    ObservationRepository::new(&db)
        .insert(&good)
        .expect("insert good");

    insert_raw_row(
        &db,
        "0190ecb5-7c2a-7123-8901-23456789abc2",
        150,
        "heart_rate",
        "{broken",
        0.9,
    );

    let repo = ObservationRepository::new(&db);
    let err = repo
        .list_by_time_range(UnixTimestamp::from_secs(100), UnixTimestamp::from_secs(200))
        .expect_err("corrupt row must fail list");

    match err {
        StorageError::PayloadDeserialize { .. } => {}
        other => panic!("expected PayloadDeserialize, got {other:?}"),
    }
}

#[test]
fn list_after_created_cursor_incremental_and_idle_empty() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);

    assert!(repo.max_created_cursor().expect("max").is_none());
    assert!(
        repo.list_after_created_cursor(0, "", 10)
            .expect("empty list")
            .is_empty()
    );

    let a = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc1", 100, "heart_rate");
    let b = sample_observation("0190ecb5-7c2a-7123-8901-23456789abc2", 200, "heart_rate");
    repo.insert(&a).expect("insert a");
    repo.insert(&b).expect("insert b");

    let tip = repo.max_created_cursor().expect("max").expect("some tip");
    let first_page = repo.list_after_created_cursor(0, "", 1).expect("page 1");
    assert_eq!(first_page.len(), 1);
    assert_eq!(first_page[0].observation.id, a.id);

    let second = repo
        .list_after_created_cursor(
            first_page[0].created_at,
            &first_page[0].observation.id.to_string(),
            10,
        )
        .expect("page 2");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].observation.id, b.id);

    let idle = repo
        .list_after_created_cursor(tip.0, &tip.1, 10)
        .expect("idle");
    assert!(idle.is_empty());

    assert!(
        repo.list_after_created_cursor(0, "", 0)
            .expect("limit 0")
            .is_empty()
    );
}

fn life_event(id: u128, timestamp: i64, kind: &str) -> Observation {
    Observation::try_new(
        Uuid::from_u128(id),
        UnixTimestamp::from_secs(timestamp),
        "com.biofocus.desktop",
        bio_spec::DATA_TYPE_LIFE_EVENT,
        json!({ "kind": kind, "logged_at": timestamp }),
        1.0,
    )
    .expect("life event")
}

fn retraction(id: u128, timestamp: i64, target: u128) -> Observation {
    Observation::try_new(
        Uuid::from_u128(id),
        UnixTimestamp::from_secs(timestamp),
        "com.biofocus.desktop",
        bio_spec::DATA_TYPE_LIFE_EVENT_RETRACTION,
        json!({ "target_id": Uuid::from_u128(target).to_string() }),
        1.0,
    )
    .expect("retraction")
}

#[test]
fn retracted_life_event_is_hidden_but_never_deleted() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    repo.insert(&life_event(1, 1_000, "coffee"))
        .expect("coffee");
    repo.insert(&life_event(2, 1_100, "walk")).expect("walk");
    repo.insert(&sample_observation(
        "0190ecb5-7c2a-7123-8901-23456789abcd",
        1_050,
        "heart_rate",
    ))
    .expect("hr");
    assert!(!repo.is_retracted(Uuid::from_u128(1)).expect("q"));

    repo.insert(&retraction(3, 1_200, 1)).expect("retract");

    // Hidden from the normal readers…
    let ranged = repo
        .list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("range");
    let ids: Vec<String> = ranged.iter().map(|o| o.data_type.clone()).collect();
    assert_eq!(ids, vec!["heart_rate", "life_event"]);
    assert_eq!(ranged[1].id, Uuid::from_u128(2));
    let by_type = repo
        .list_by_data_type(bio_spec::DATA_TYPE_LIFE_EVENT)
        .expect("type");
    assert_eq!(by_type.len(), 1);
    assert_eq!(by_type[0].id, Uuid::from_u128(2));
    assert!(repo.is_retracted(Uuid::from_u128(1)).expect("q"));

    // …but the original row and the marker are still stored (append-only).
    assert!(repo.get_by_id(Uuid::from_u128(1)).expect("get").is_some());
    let raw = repo
        .list_by_time_range_raw(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("raw");
    assert_eq!(raw.len(), 4);
    let markers = repo
        .list_by_data_type(bio_spec::DATA_TYPE_LIFE_EVENT_RETRACTION)
        .expect("markers");
    assert_eq!(markers.len(), 1);
    // Incremental worker cursor sees the marker (raw).
    let after = repo.list_after_created_cursor(0, "", 10).expect("cursor");
    assert_eq!(after.len(), 4);
}

#[test]
fn retraction_only_hides_life_events() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    let hr_id = "0190ecb5-7c2a-7123-8901-23456789abcd";
    repo.insert(&sample_observation(hr_id, 1_050, "heart_rate"))
        .expect("hr");
    let bogus = Observation::try_new(
        Uuid::from_u128(9),
        UnixTimestamp::from_secs(1_060),
        "com.biofocus.desktop",
        bio_spec::DATA_TYPE_LIFE_EVENT_RETRACTION,
        json!({ "target_id": hr_id }),
        1.0,
    )
    .expect("marker");
    repo.insert(&bogus).expect("insert marker");
    let ranged = repo
        .list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("range");
    assert_eq!(
        ranged.len(),
        1,
        "heart_rate must not be hidden by a retraction"
    );
}

#[test]
fn source_deletion_hides_same_provider_only() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    let target = "0190a030-0000-7000-8000-000000000001";
    repo.insert(&sample_observation(target, 1_050, "heart_rate"))
        .expect("hr");
    let other = Observation::try_new(
        Uuid::parse_str("0190a030-0000-7000-8000-000000000002").expect("uuid"),
        UnixTimestamp::from_secs(1_050),
        "com.example.other",
        "heart_rate",
        json!({ "bpm": 70 }),
        1.0,
    )
    .expect("other");
    repo.insert(&other).expect("insert other");
    let foreign = Observation::try_new(
        Uuid::from_u128(8),
        UnixTimestamp::from_secs(1_060),
        "com.example.other",
        bio_spec::DATA_TYPE_SOURCE_DELETION,
        json!({ "target_id": target }),
        1.0,
    )
    .expect("foreign marker");
    repo.insert(&foreign).expect("insert foreign");
    let still = repo
        .list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("range");
    assert_eq!(still.len(), 2, "another provider must not hide the row");

    let marker = Observation::try_new(
        Uuid::from_u128(9),
        UnixTimestamp::from_secs(1_070),
        "com.biofocus.applehealth",
        bio_spec::DATA_TYPE_SOURCE_DELETION,
        json!({ "target_id": target }),
        1.0,
    )
    .expect("marker");
    repo.insert(&marker).expect("insert marker");
    let hidden = repo
        .list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("range");
    assert_eq!(hidden.len(), 1);
    assert_eq!(hidden[0].provider_id, "com.example.other");
    let raw = repo
        .list_by_time_range_raw(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(2_000))
        .expect("raw");
    assert!(raw.len() >= 4);
}

#[test]
fn list_with_created_returns_receipt_time_for_the_asked_types() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    let hr = sample_observation(
        "0190ecb5-7c2a-7123-8901-23456789abcd",
        1_700_000_000,
        "heart_rate",
    );
    repo.insert(&hr).expect("insert");
    let other = sample_observation(
        "0190ecb5-7c2a-7123-8901-23456789abce",
        1_700_000_100,
        "keystrokes",
    );
    repo.insert(&other).expect("insert");

    let rows = repo
        .list_with_created_in_range(
            &["heart_rate"],
            UnixTimestamp::from_secs(1_600_000_000),
            UnixTimestamp::from_secs(1_800_000_000),
        )
        .expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].observation.data_type, "heart_rate");
    assert!(rows[0].created_at > 0);
}

#[test]
fn summarize_types_since_groups_without_payloads() {
    let db = Database::open_in_memory().expect("open");
    let repo = ObservationRepository::new(&db);
    let hr = sample_observation(
        "01900000-0000-7000-8000-0000000000a1",
        2_000,
        "heart_rate",
    );
    let hr2 = sample_observation(
        "01900000-0000-7000-8000-0000000000a2",
        3_000,
        "heart_rate",
    );
    let steps = sample_observation(
        "01900000-0000-7000-8000-0000000000a3",
        2_500,
        "step_count",
    );
    repo.insert(&hr).expect("hr");
    repo.insert(&hr2).expect("hr2");
    repo.insert(&steps).expect("steps");

    let rows = repo
        .summarize_types_since(UnixTimestamp::from_secs(2_500))
        .expect("summary");
    assert_eq!(rows.len(), 2);
    let hr_row = rows.iter().find(|r| r.data_type == "heart_rate").expect("hr");
    assert_eq!(hr_row.count, 1);
    assert_eq!(hr_row.latest_timestamp, 3_000);
    let step_row = rows.iter().find(|r| r.data_type == "step_count").expect("steps");
    assert_eq!(step_row.count, 1);
}
