//! Desktop quick-log for Life Event Observations (P6-E2-T1 / ADR-006).
//!
//! UI invokes Tauri commands only — no SQLite from the frontend.
//! Persist uses the existing [`storage::ObservationRepository`] (same store as
//! ingest persist worker). No parallel Life Events table.

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{
    is_v1_life_event_kind, validate_observation_payload, Observation, UnixTimestamp,
    DATA_TYPE_LIFE_EVENT, V1_LIFE_EVENT_KINDS,
};
use serde::Serialize;
use serde_json::json;
use storage::{Database, ObservationRepository};
use uuid::Uuid;

/// Provider id for manual Desktop quick-log (`docs/07-contracts.md`).
pub const DESKTOP_LIFE_EVENT_PROVIDER_ID: &str = "com.biofocus.desktop";

/// Default number of recent Life Events returned to the Menubar.
pub const DEFAULT_RECENT_LIFE_EVENTS_LIMIT: usize = 8;

/// Max recent rows the IPC will return (hard cap).
const MAX_RECENT_LIFE_EVENTS_LIMIT: usize = 32;

/// One Life Event row for Menubar (camelCase; no biometric payloads).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LifeEventDto {
    pub id: String,
    pub kind: String,
    pub timestamp: i64,
    pub provider_id: String,
}

/// Builds a validated v1 Life Event [`Observation`] for Desktop quick-log.
pub fn build_life_event_observation(kind: &str) -> Result<Observation, String> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err("Choose a life event to log.".into());
    }
    if !is_v1_life_event_kind(kind) {
        return Err(format!(
            "That event kind isn’t available. Use one of: {}.",
            V1_LIFE_EVENT_KINDS.join(", ")
        ));
    }

    let timestamp_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let obs = Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(timestamp_secs),
        DESKTOP_LIFE_EVENT_PROVIDER_ID,
        DATA_TYPE_LIFE_EVENT,
        json!({ "kind": kind }),
        1.0,
    )
    .map_err(|_| "Could not build that life event.".to_string())?;

    validate_observation_payload(&obs).map_err(|err| match err {
        bio_spec::SpecError::InvalidLifeEventPayload { .. } => {
            "That life event could not be validated.".to_string()
        }
        _ => "Could not validate that life event.".to_string(),
    })?;

    Ok(obs)
}

fn life_event_dto(obs: &Observation) -> Result<LifeEventDto, String> {
    let kind = obs
        .payload
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Stored life event is missing kind.".to_string())?;
    Ok(LifeEventDto {
        id: obs.id.to_string(),
        kind: kind.to_owned(),
        timestamp: obs.timestamp.as_secs(),
        provider_id: obs.provider_id.clone(),
    })
}

/// Appends a Life Event Observation via the shared Observation repository.
pub fn log_life_event(kind: &str) -> Result<LifeEventDto, String> {
    let obs = build_life_event_observation(kind)?;
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    repo.insert(&obs).map_err(|err| match err {
        storage::StorageError::DuplicateObservation { .. } => {
            "That life event was already logged.".to_string()
        }
        other => other.public_message(),
    })?;
    life_event_dto(&obs)
}

fn clamp_recent_limit(limit: Option<u32>) -> usize {
    let requested = limit
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_RECENT_LIFE_EVENTS_LIMIT);
    requested.clamp(1, MAX_RECENT_LIFE_EVENTS_LIMIT)
}

/// Lists recent Life Event Observations (newest first) for Menubar confirmation.
pub fn list_recent_life_events(limit: Option<u32>) -> Result<Vec<LifeEventDto>, String> {
    let take = clamp_recent_limit(limit);
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    let rows = repo
        .list_by_data_type(DATA_TYPE_LIFE_EVENT)
        .map_err(|err| err.public_message())?;

    let mut out = Vec::new();
    for obs in rows.into_iter().rev().take(take) {
        // Skip malformed rows quietly — UI stays calm.
        if let Ok(dto) = life_event_dto(&obs) {
            out.push(dto);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::Database;

    #[test]
    fn build_accepts_v1_kinds() {
        for kind in V1_LIFE_EVENT_KINDS {
            let obs = build_life_event_observation(kind).expect("v1 kind");
            assert_eq!(obs.data_type, DATA_TYPE_LIFE_EVENT);
            assert_eq!(obs.provider_id, DESKTOP_LIFE_EVENT_PROVIDER_ID);
            assert_eq!(obs.payload["kind"], json!(kind));
            validate_observation_payload(&obs).expect("payload ok");
        }
    }

    #[test]
    fn build_rejects_unknown_kind() {
        let err = build_life_event_observation("meeting").expect_err("reject");
        assert!(err.contains("isn’t available") || err.contains("isn't available"));
    }

    #[test]
    fn insert_and_list_round_trip_temp_db() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "biofocus-life-event-ipc-{}-{}.db",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_file(&path);

        let obs = build_life_event_observation("coffee").expect("build");
        {
            let db = Database::open(&path).expect("open");
            ObservationRepository::new(&db)
                .insert(&obs)
                .expect("insert");
            let listed = ObservationRepository::new(&db)
                .list_by_data_type(DATA_TYPE_LIFE_EVENT)
                .expect("list");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].id, obs.id);
            assert_eq!(listed[0].payload["kind"], json!("coffee"));
        }

        let _ = std::fs::remove_file(&path);
    }
}
