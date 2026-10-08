//! Immutable append/read API for `observations` (`docs/04-storage.md`).

use bio_spec::{
    Confidence, DATA_TYPE_LIFE_EVENT, DATA_TYPE_LIFE_EVENT_RETRACTION, DATA_TYPE_SOURCE_DELETION,
    DataType, Observation, ObservationId, UnixTimestamp,
};
use rusqlite::{Connection, ErrorCode, OptionalExtension, Row, params};
use uuid::Uuid;

use crate::clock::unix_now_secs;
use crate::{Database, StorageError, StorageResult};

/// Repository over the `observations` table.
///
/// Inserts are append-only; duplicates of primary key are rejected (no UPSERT).
pub struct ObservationRepository<'db> {
    conn: &'db Connection,
}

/// SQL predicate (alias-free) hiding retracted Life Events and the retraction
/// markers themselves. Rows are never deleted — this is a read-side filter.
///
/// Data type names are inlined bio-spec constants (closed set, never user input).
fn retraction_filter_sql() -> String {
    format!(
        "data_type <> '{r}'
         AND NOT (data_type = '{l}' AND id IN (
             SELECT json_extract(payload, '$.target_id') FROM observations
             WHERE data_type = '{r}'))",
        r = DATA_TYPE_LIFE_EVENT_RETRACTION,
        l = DATA_TYPE_LIFE_EVENT,
    )
}

/// Hides `source_deletion` markers and targets that share the marker's provider.
/// A marker from another provider does not hide the row (ADR-030).
fn source_deletion_filter_sql() -> String {
    format!(
        "data_type <> '{d}'
         AND NOT EXISTS (
             SELECT 1 FROM observations AS src_del
             WHERE src_del.data_type = '{d}'
               AND json_extract(src_del.payload, '$.target_id') = observations.id
               AND src_del.provider_id = observations.provider_id)",
        d = DATA_TYPE_SOURCE_DELETION,
    )
}

impl<'db> ObservationRepository<'db> {
    /// Borrows the open [`Database`] connection (migrations already applied on open).
    #[must_use]
    pub fn new(db: &'db Database) -> Self {
        Self {
            conn: db.connection(),
        }
    }

    /// Appends an Observation. `created_at` is set to Unix UTC seconds at insert time
    /// and is **not** part of [`Observation`] (distinct from `timestamp`).
    pub fn insert(&self, observation: &Observation) -> StorageResult<()> {
        let payload = serde_json::to_string(&observation.payload)
            .map_err(|source| StorageError::PayloadSerialize { source })?;
        let created_at = unix_now_secs()?;
        let id = observation.id.to_string();

        let result = self.conn.execute(
            "INSERT INTO observations
                (id, timestamp, provider_id, data_type, payload, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                observation.timestamp.as_secs(),
                observation.provider_id,
                observation.data_type,
                payload,
                observation.confidence.get(),
                created_at,
            ],
        );

        match result {
            Ok(_) => Ok(()),
            Err(err) if is_primary_key_constraint(&err) => {
                Err(StorageError::DuplicateObservation { id: observation.id })
            }
            Err(err) => Err(StorageError::Sqlite(err)),
        }
    }

    /// Loads a single Observation by id, if present.
    pub fn get_by_id(&self, id: ObservationId) -> StorageResult<Option<Observation>> {
        let id_str = id.to_string();
        let row = self
            .conn
            .query_row(
                "SELECT id, timestamp, provider_id, data_type, payload, confidence
                 FROM observations WHERE id = ?1",
                params![id_str],
                read_observation_columns,
            )
            .optional()?;

        match row {
            None => Ok(None),
            Some(cols) => Ok(Some(cols.into_observation()?)),
        }
    }

    /// Lists Observations with `timestamp` in inclusive `[start, end]`, ordered by time then id.
    ///
    /// Retracted Life Events and `life_event_retraction` markers are excluded
    /// (append-only undo; see [`Self::list_by_time_range_raw`] for every row).
    pub fn list_by_time_range(
        &self,
        start: UnixTimestamp,
        end: UnixTimestamp,
    ) -> StorageResult<Vec<Observation>> {
        self.list_by_time_range_inner(start, end, true)
    }

    /// Like [`Self::list_by_time_range`] but returns every stored row,
    /// including retracted Life Events and retraction markers (audit/export).
    pub fn list_by_time_range_raw(
        &self,
        start: UnixTimestamp,
        end: UnixTimestamp,
    ) -> StorageResult<Vec<Observation>> {
        self.list_by_time_range_inner(start, end, false)
    }

    fn list_by_time_range_inner(
        &self,
        start: UnixTimestamp,
        end: UnixTimestamp,
        hide_retracted: bool,
    ) -> StorageResult<Vec<Observation>> {
        if end.as_secs() < start.as_secs() {
            return Err(StorageError::InvalidTimeRange {
                start: start.as_secs(),
                end: end.as_secs(),
            });
        }

        let filter = if hide_retracted {
            format!(
                " AND {} AND {}",
                retraction_filter_sql(),
                source_deletion_filter_sql()
            )
        } else {
            String::new()
        };
        let sql = format!(
            "SELECT id, timestamp, provider_id, data_type, payload, confidence
             FROM observations
             WHERE timestamp >= ?1 AND timestamp <= ?2{filter}
             ORDER BY timestamp ASC, id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(
            params![start.as_secs(), end.as_secs()],
            read_observation_columns,
        )?;
        collect_observations(rows)
    }

    /// Lists Observations matching `data_type`, ordered by time then id.
    ///
    /// For `life_event`, retracted events are excluded. Asking for
    /// `life_event_retraction` explicitly returns the markers.
    pub fn list_by_data_type(&self, data_type: &str) -> StorageResult<Vec<Observation>> {
        let mut filter = String::new();
        if data_type != DATA_TYPE_LIFE_EVENT_RETRACTION {
            filter.push_str(" AND ");
            filter.push_str(&retraction_filter_sql());
        }
        if data_type != DATA_TYPE_SOURCE_DELETION {
            filter.push_str(" AND ");
            filter.push_str(&source_deletion_filter_sql());
        }
        let sql = format!(
            "SELECT id, timestamp, provider_id, data_type, payload, confidence
             FROM observations
             WHERE data_type = ?1{filter}
             ORDER BY timestamp ASC, id ASC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![data_type], read_observation_columns)?;
        collect_observations(rows)
    }

    /// `true` when a `life_event_retraction` marker targets `id`.
    pub fn is_retracted(&self, id: ObservationId) -> StorageResult<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM observations
             WHERE data_type = ?1 AND json_extract(payload, '$.target_id') = ?2",
            params![DATA_TYPE_LIFE_EVENT_RETRACTION, id.to_string()],
            |row| row.get(0),
        )?;
        Ok(n > 0)
    }

    /// Highest `(created_at, id)` cursor in the table, if any rows exist.
    ///
    /// Used by the Feature Worker to skip historical backlog and only follow
    /// inserts after worker start (no schema change; uses existing `created_at`).
    pub fn max_created_cursor(&self) -> StorageResult<Option<(i64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT created_at, id FROM observations
             ORDER BY created_at DESC, id DESC
             LIMIT 1",
        )?;
        let row = stmt
            .query_row([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .optional()?;
        Ok(row)
    }

    /// Lists Observations strictly after `(after_created_at, after_id)` in
    /// `(created_at, id)` order, up to `limit`.
    ///
    /// Empty when no newer rows (idle-friendly). `limit == 0` → empty `Ok`.
    /// Does not mutate rows. Enables incremental Feature Worker polls without
    /// a new SQLite table. **Raw**: retraction markers are returned so the
    /// worker can drop the target from its in-memory window.
    pub fn list_after_created_cursor(
        &self,
        after_created_at: i64,
        after_id: &str,
        limit: usize,
    ) -> StorageResult<Vec<ObservationCreated>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, provider_id, data_type, payload, confidence, created_at
             FROM observations
             WHERE created_at > ?1 OR (created_at = ?1 AND id > ?2)
             ORDER BY created_at ASC, id ASC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![after_created_at, after_id, limit as i64],
            read_observation_created_columns,
        )?;
        collect_observations_created(rows)
    }
}

/// Observation plus append-time `created_at` (not part of [`Observation`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationCreated {
    /// Domain Observation row.
    pub observation: Observation,
    /// Unix UTC seconds written at insert time.
    pub created_at: i64,
}

struct ObservationColumns {
    id: String,
    timestamp: i64,
    provider_id: String,
    data_type: String,
    payload: String,
    confidence: f64,
}

impl ObservationColumns {
    fn into_observation(self) -> StorageResult<Observation> {
        let id =
            Uuid::parse_str(&self.id).map_err(|source| StorageError::InvalidObservationId {
                value: self.id,
                source,
            })?;
        let payload = serde_json::from_str(&self.payload)
            .map_err(|source| StorageError::PayloadDeserialize { source })?;
        let confidence = Confidence::try_new(self.confidence)?;

        Ok(Observation {
            id,
            timestamp: UnixTimestamp::from_secs(self.timestamp),
            provider_id: self.provider_id,
            data_type: self.data_type,
            payload,
            confidence,
        })
    }
}

struct ObservationCreatedColumns {
    base: ObservationColumns,
    created_at: i64,
}

impl ObservationCreatedColumns {
    fn into_observation_created(self) -> StorageResult<ObservationCreated> {
        Ok(ObservationCreated {
            observation: self.base.into_observation()?,
            created_at: self.created_at,
        })
    }
}

fn read_observation_columns(row: &Row<'_>) -> rusqlite::Result<ObservationColumns> {
    Ok(ObservationColumns {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        provider_id: row.get(2)?,
        data_type: row.get::<_, DataType>(3)?,
        payload: row.get(4)?,
        confidence: row.get(5)?,
    })
}

fn read_observation_created_columns(row: &Row<'_>) -> rusqlite::Result<ObservationCreatedColumns> {
    Ok(ObservationCreatedColumns {
        base: read_observation_columns(row)?,
        created_at: row.get(6)?,
    })
}

fn collect_observations(
    rows: impl Iterator<Item = Result<ObservationColumns, rusqlite::Error>>,
) -> StorageResult<Vec<Observation>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row?.into_observation()?);
    }
    Ok(out)
}

fn collect_observations_created(
    rows: impl Iterator<Item = Result<ObservationCreatedColumns, rusqlite::Error>>,
) -> StorageResult<Vec<ObservationCreated>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row?.into_observation_created()?);
    }
    Ok(out)
}

fn is_primary_key_constraint(err: &rusqlite::Error) -> bool {
    match err {
        rusqlite::Error::SqliteFailure(info, _) => {
            info.code == ErrorCode::ConstraintViolation
                || info.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                || info.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
        }
        _ => false,
    }
}
