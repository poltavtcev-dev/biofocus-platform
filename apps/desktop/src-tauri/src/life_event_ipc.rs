//! Desktop quick-log for Life Event Observations (P6-E2-T1 / ADR-006).
//!
//! UI invokes Tauri commands only — no SQLite from the frontend.
//! Persist uses the existing [`storage::ObservationRepository`] (same store as
//! ingest persist worker). No parallel Life Events table.
//!
//! ## Timestamps
//! - `Observation.timestamp` = **happened at** (what Features / Insights / the
//!   chart use). Defaults to now; the user may back-date up to 24 h.
//! - payload `logged_at` = when the tap was recorded (kept for honesty).
//!
//! ## Remove / undo / re-time (append-only)
//! Rows are never deleted or mutated. Removing appends a
//! `life_event_retraction` marker; storage readers hide the target. Undo of a
//! removal and changing the time both append a fresh `life_event` copy with
//! `edited_from` = previous id (re-time also retracts the old row).

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{
    is_v1_life_event_kind, validate_observation_payload, Observation, UnixTimestamp,
    DATA_TYPE_LIFE_EVENT, DATA_TYPE_LIFE_EVENT_RETRACTION, V1_LIFE_EVENT_KINDS,
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

/// How far back "happened at" may be set (seconds).
pub const MAX_BACKDATE_SECS: i64 = 24 * 3600;

/// Small allowance for clock skew between UI and core (seconds).
const FUTURE_SKEW_SECS: i64 = 60;

/// Cap for chart-marker queries.
const MAX_RANGE_LIFE_EVENTS: usize = 500;

/// One Life Event row for Menubar (camelCase; no biometric payloads).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LifeEventDto {
    pub id: String,
    pub kind: String,
    /// Happened at (Unix secs) — what the data uses.
    pub timestamp: i64,
    /// When the user recorded it (Unix secs). Equals `timestamp` for legacy rows.
    pub logged_at: i64,
    /// `true` when this row replaced an earlier one (re-timed or restored).
    pub edited: bool,
    pub provider_id: String,
}

/// Result of a removal: the hidden event id and the marker id.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LifeEventRetractionDto {
    pub id: String,
    pub retraction_id: String,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn check_happened_at(happened_at: i64, now: i64) -> Result<(), String> {
    if happened_at > now + FUTURE_SKEW_SECS {
        return Err("A life event can’t be in the future.".into());
    }
    if happened_at < now - MAX_BACKDATE_SECS {
        return Err("Life events can be back-dated up to 24 hours.".into());
    }
    Ok(())
}

fn validated(obs: Observation) -> Result<Observation, String> {
    validate_observation_payload(&obs).map_err(|err| match err {
        bio_spec::SpecError::InvalidLifeEventPayload { .. } => {
            "That life event could not be validated.".to_string()
        }
        _ => "Could not validate that life event.".to_string(),
    })?;
    Ok(obs)
}

fn check_kind(kind: &str) -> Result<&str, String> {
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
    Ok(kind)
}

/// Builds a validated v1 Life Event with optional back-dated `happened_at`.
pub fn build_life_event_observation_at(
    kind: &str,
    happened_at: Option<i64>,
    now: i64,
) -> Result<Observation, String> {
    let kind = check_kind(kind)?;
    let happened_at = happened_at.unwrap_or(now);
    check_happened_at(happened_at, now)?;
    let obs = Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(happened_at),
        DESKTOP_LIFE_EVENT_PROVIDER_ID,
        DATA_TYPE_LIFE_EVENT,
        json!({ "kind": kind, "logged_at": now }),
        1.0,
    )
    .map_err(|_| "Could not build that life event.".to_string())?;
    validated(obs)
}

pub(crate) fn life_event_dto(obs: &Observation) -> Result<LifeEventDto, String> {
    let kind = obs
        .payload
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Stored life event is missing kind.".to_string())?;
    let logged_at = obs
        .payload
        .get("logged_at")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| obs.timestamp.as_secs());
    Ok(LifeEventDto {
        id: obs.id.to_string(),
        kind: kind.to_owned(),
        timestamp: obs.timestamp.as_secs(),
        logged_at,
        edited: obs.payload.get("edited_from").is_some(),
        provider_id: obs.provider_id.clone(),
    })
}

fn open_default_db() -> Result<Database, String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    Database::open(&path).map_err(|err| err.public_message())
}

fn insert(repo: &ObservationRepository<'_>, obs: &Observation) -> Result<(), String> {
    repo.insert(obs).map_err(|err| match err {
        storage::StorageError::DuplicateObservation { .. } => {
            "That life event was already logged.".to_string()
        }
        other => other.public_message(),
    })
}

fn parse_id(id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(id.trim()).map_err(|_| "That life event id isn’t valid.".to_string())
}

/// Loads a stored life event (even if already retracted).
fn load_life_event(repo: &ObservationRepository<'_>, id: Uuid) -> Result<Observation, String> {
    let obs = repo
        .get_by_id(id)
        .map_err(|err| err.public_message())?
        .ok_or_else(|| "That life event wasn’t found.".to_string())?;
    if obs.data_type != DATA_TYPE_LIFE_EVENT {
        return Err("Only life events can be changed here.".into());
    }
    Ok(obs)
}

fn copy_of(original: &Observation, happened_at: i64, now: i64) -> Result<Observation, String> {
    let kind = original
        .payload
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Stored life event is missing kind.".to_string())?;
    let logged_at = original
        .payload
        .get("logged_at")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| original.timestamp.as_secs());
    let obs = Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(happened_at),
        DESKTOP_LIFE_EVENT_PROVIDER_ID,
        DATA_TYPE_LIFE_EVENT,
        json!({
            "kind": kind,
            "logged_at": logged_at,
            "edited_from": original.id.to_string(),
            "edited_at": now,
        }),
        1.0,
    )
    .map_err(|_| "Could not build that life event.".to_string())?;
    validated(obs)
}

fn retraction_for(target: &Observation, now: i64) -> Result<Observation, String> {
    let marker = Observation::try_new(
        Uuid::now_v7(),
        UnixTimestamp(now),
        DESKTOP_LIFE_EVENT_PROVIDER_ID,
        DATA_TYPE_LIFE_EVENT_RETRACTION,
        json!({
            "target_id": target.id.to_string(),
            "target_kind": target.payload.get("kind").cloned().unwrap_or(json!(null)),
        }),
        1.0,
    )
    .map_err(|_| "Could not remove that life event.".to_string())?;
    validated(marker)
}

/// Appends a Life Event (optionally back-dated) to `db`.
pub fn log_life_event_in(
    db: &Database,
    kind: &str,
    happened_at: Option<i64>,
    now: i64,
) -> Result<LifeEventDto, String> {
    let obs = build_life_event_observation_at(kind, happened_at, now)?;
    insert(&ObservationRepository::new(db), &obs)?;
    life_event_dto(&obs)
}

/// Appends a Life Event Observation via the shared Observation repository.
pub fn log_life_event(kind: &str, happened_at: Option<i64>) -> Result<LifeEventDto, String> {
    let db = open_default_db()?;
    log_life_event_in(&db, kind, happened_at, now_secs())
}

/// Hides a Life Event by appending a retraction marker (nothing is deleted).
pub fn retract_life_event_in(
    db: &Database,
    id: &str,
    now: i64,
) -> Result<LifeEventRetractionDto, String> {
    let repo = ObservationRepository::new(db);
    let target = load_life_event(&repo, parse_id(id)?)?;
    if repo.is_retracted(target.id).map_err(|e| e.public_message())? {
        return Err("That life event was already removed.".into());
    }
    let marker = retraction_for(&target, now)?;
    insert(&repo, &marker)?;
    Ok(LifeEventRetractionDto {
        id: target.id.to_string(),
        retraction_id: marker.id.to_string(),
    })
}

/// Tauri entry: remove (retract) a Life Event.
pub fn retract_life_event(id: &str) -> Result<LifeEventRetractionDto, String> {
    let db = open_default_db()?;
    retract_life_event_in(&db, id, now_secs())
}

/// Undo a removal: appends a copy of the retracted event (same happened-at
/// and logged-at, `edited_from` = removed id). The marker stays for audit.
pub fn restore_life_event_in(db: &Database, id: &str, now: i64) -> Result<LifeEventDto, String> {
    let repo = ObservationRepository::new(db);
    let original = load_life_event(&repo, parse_id(id)?)?;
    if !repo.is_retracted(original.id).map_err(|e| e.public_message())? {
        return Err("That life event is still logged.".into());
    }
    let already = repo
        .list_by_data_type(DATA_TYPE_LIFE_EVENT)
        .map_err(|e| e.public_message())?
        .into_iter()
        .any(|o| o.payload.get("edited_from").and_then(|v| v.as_str()) == Some(&original.id.to_string()));
    if already {
        return Err("That life event was already restored.".into());
    }
    let copy = copy_of(&original, original.timestamp.as_secs(), now)?;
    insert(&repo, &copy)?;
    life_event_dto(&copy)
}

/// Tauri entry: undo a removal.
pub fn restore_life_event(id: &str) -> Result<LifeEventDto, String> {
    let db = open_default_db()?;
    restore_life_event_in(&db, id, now_secs())
}

/// Change "happened at": append a re-timed copy, then retract the old row
/// (single SQLite transaction). `logged_at` is preserved.
pub fn retime_life_event_in(
    db: &Database,
    id: &str,
    happened_at: i64,
    now: i64,
) -> Result<LifeEventDto, String> {
    check_happened_at(happened_at, now)?;
    let repo = ObservationRepository::new(db);
    let original = load_life_event(&repo, parse_id(id)?)?;
    if repo.is_retracted(original.id).map_err(|e| e.public_message())? {
        return Err("That life event was removed.".into());
    }
    let copy = copy_of(&original, happened_at, now)?;
    let marker = retraction_for(&original, now)?;
    let tx = db
        .connection()
        .unchecked_transaction()
        .map_err(|_| "Could not update that life event.".to_string())?;
    insert(&repo, &copy)?;
    insert(&repo, &marker)?;
    tx.commit()
        .map_err(|_| "Could not update that life event.".to_string())?;
    life_event_dto(&copy)
}

/// Tauri entry: change when a Life Event happened.
pub fn retime_life_event(id: &str, happened_at: i64) -> Result<LifeEventDto, String> {
    let db = open_default_db()?;
    retime_life_event_in(&db, id, happened_at, now_secs())
}

fn clamp_recent_limit(limit: Option<u32>) -> usize {
    let requested = limit
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_RECENT_LIFE_EVENTS_LIMIT);
    requested.clamp(1, MAX_RECENT_LIFE_EVENTS_LIMIT)
}

/// Recent (non-retracted) Life Events, newest happened-at first.
pub fn list_recent_life_events_in(db: &Database, limit: Option<u32>) -> Result<Vec<LifeEventDto>, String> {
    let take = clamp_recent_limit(limit);
    let rows = ObservationRepository::new(db)
        .list_by_data_type(DATA_TYPE_LIFE_EVENT)
        .map_err(|err| err.public_message())?;
    // Skip malformed rows quietly — UI stays calm.
    Ok(rows
        .iter()
        .rev()
        .filter_map(|obs| life_event_dto(obs).ok())
        .take(take)
        .collect())
}

/// Lists recent Life Event Observations (newest first) for Menubar confirmation.
pub fn list_recent_life_events(limit: Option<u32>) -> Result<Vec<LifeEventDto>, String> {
    let db = open_default_db()?;
    list_recent_life_events_in(&db, limit)
}

/// Non-retracted Life Events with happened-at in `[start, end]`, oldest first
/// (chart markers / report). Capped at 500.
pub fn list_life_events_between_in(
    db: &Database,
    start: i64,
    end: i64,
) -> Result<Vec<LifeEventDto>, String> {
    if end < start {
        return Ok(Vec::new());
    }
    let rows = ObservationRepository::new(db)
        .list_by_data_type(DATA_TYPE_LIFE_EVENT)
        .map_err(|err| err.public_message())?;
    let mut out: Vec<LifeEventDto> = rows
        .iter()
        .filter(|o| (start..=end).contains(&o.timestamp.as_secs()))
        .filter_map(|o| life_event_dto(o).ok())
        .collect();
    if out.len() > MAX_RANGE_LIFE_EVENTS {
        out.drain(0..out.len() - MAX_RANGE_LIFE_EVENTS);
    }
    Ok(out)
}

/// Tauri entry: life events between two Unix seconds.
pub fn list_life_events_between(start: i64, end: i64) -> Result<Vec<LifeEventDto>, String> {
    let db = open_default_db()?;
    list_life_events_between_in(&db, start, end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::Database;

    const NOW: i64 = 1_800_000_000;

    #[test]
    fn build_accepts_v1_kinds() {
        for kind in V1_LIFE_EVENT_KINDS {
            let obs = build_life_event_observation_at(kind, None, NOW).expect("v1 kind");
            assert_eq!(obs.data_type, DATA_TYPE_LIFE_EVENT);
            assert_eq!(obs.provider_id, DESKTOP_LIFE_EVENT_PROVIDER_ID);
            assert_eq!(obs.payload["kind"], json!(kind));
            assert!(obs.payload["logged_at"].as_i64().is_some());
            validate_observation_payload(&obs).expect("payload ok");
        }
    }

    #[test]
    fn build_rejects_unknown_kind() {
        let err = build_life_event_observation_at("meeting", None, NOW).expect_err("reject");
        assert!(err.contains("isn’t available") || err.contains("isn't available"));
    }

    #[test]
    fn backdated_event_keeps_both_timestamps() {
        let obs = build_life_event_observation_at("walk", Some(NOW - 1800), NOW).expect("ok");
        assert_eq!(obs.timestamp.as_secs(), NOW - 1800);
        assert_eq!(obs.payload["logged_at"], json!(NOW));
        let dto = life_event_dto(&obs).expect("dto");
        assert_eq!(dto.timestamp, NOW - 1800);
        assert_eq!(dto.logged_at, NOW);
        assert!(!dto.edited);
    }

    #[test]
    fn happened_at_bounds() {
        assert!(build_life_event_observation_at("walk", Some(NOW + 3600), NOW).is_err());
        assert!(build_life_event_observation_at("walk", Some(NOW - MAX_BACKDATE_SECS - 1), NOW).is_err());
        assert!(build_life_event_observation_at("walk", Some(NOW - MAX_BACKDATE_SECS), NOW).is_ok());
    }

    #[test]
    fn legacy_row_without_logged_at_falls_back_to_timestamp() {
        let obs = Observation::try_new(
            Uuid::now_v7(),
            UnixTimestamp(NOW),
            DESKTOP_LIFE_EVENT_PROVIDER_ID,
            DATA_TYPE_LIFE_EVENT,
            json!({ "kind": "coffee" }),
            1.0,
        )
        .expect("obs");
        assert_eq!(life_event_dto(&obs).expect("dto").logged_at, NOW);
    }

    #[test]
    fn retract_hides_from_recent_and_range_and_rejects_double() {
        let db = Database::open_in_memory().expect("db");
        let coffee = log_life_event_in(&db, "coffee", Some(NOW - 600), NOW).expect("coffee");
        let walk = log_life_event_in(&db, "walk", None, NOW).expect("walk");

        let r = retract_life_event_in(&db, &coffee.id, NOW + 5).expect("retract");
        assert_eq!(r.id, coffee.id);

        let recent = list_recent_life_events_in(&db, None).expect("recent");
        assert_eq!(recent.iter().map(|d| d.id.clone()).collect::<Vec<_>>(), vec![walk.id.clone()]);
        let range = list_life_events_between_in(&db, NOW - 3600, NOW + 60).expect("range");
        assert_eq!(range.len(), 1);

        assert!(retract_life_event_in(&db, &coffee.id, NOW + 6)
            .expect_err("double")
            .contains("already removed"));
        assert!(retract_life_event_in(&db, "not-a-uuid", NOW).is_err());
        assert!(retract_life_event_in(&db, &Uuid::now_v7().to_string(), NOW).is_err());
    }

    #[test]
    fn retract_refuses_non_life_event() {
        let db = Database::open_in_memory().expect("db");
        let hr = Observation::try_new(
            Uuid::now_v7(),
            UnixTimestamp(NOW),
            "com.biofocus.test",
            "heart_rate",
            json!({ "bpm": 70.0 }),
            1.0,
        )
        .expect("hr");
        ObservationRepository::new(&db).insert(&hr).expect("insert");
        assert!(retract_life_event_in(&db, &hr.id.to_string(), NOW).is_err());
    }

    #[test]
    fn restore_brings_back_copy_once() {
        let db = Database::open_in_memory().expect("db");
        let coffee = log_life_event_in(&db, "coffee", Some(NOW - 900), NOW - 60).expect("coffee");
        assert!(restore_life_event_in(&db, &coffee.id, NOW).is_err(), "not removed yet");
        retract_life_event_in(&db, &coffee.id, NOW).expect("retract");
        let restored = restore_life_event_in(&db, &coffee.id, NOW + 1).expect("restore");
        assert_ne!(restored.id, coffee.id);
        assert_eq!(restored.timestamp, NOW - 900);
        assert_eq!(restored.logged_at, NOW - 60);
        assert!(restored.edited);
        let recent = list_recent_life_events_in(&db, None).expect("recent");
        assert_eq!(recent.len(), 1);
        assert!(restore_life_event_in(&db, &coffee.id, NOW + 2).is_err(), "only once");
    }

    #[test]
    fn retime_replaces_event_and_keeps_logged_at() {
        let db = Database::open_in_memory().expect("db");
        let walk = log_life_event_in(&db, "walk", None, NOW).expect("walk");
        let moved = retime_life_event_in(&db, &walk.id, NOW - 1800, NOW + 30).expect("retime");
        assert_eq!(moved.timestamp, NOW - 1800);
        assert_eq!(moved.logged_at, NOW);
        assert!(moved.edited);
        let recent = list_recent_life_events_in(&db, None).expect("recent");
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].id, moved.id);
        // Old row + marker still stored.
        let raw = ObservationRepository::new(&db)
            .list_by_time_range_raw(UnixTimestamp(0), UnixTimestamp(NOW + 100))
            .expect("raw");
        assert_eq!(raw.len(), 3);
        assert!(retime_life_event_in(&db, &walk.id, NOW, NOW + 40).is_err(), "old id is gone");
    }
}
