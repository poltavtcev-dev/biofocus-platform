//! Schema migrations. Schema is frozen to `/docs/04-storage.md` (no ad-hoc column changes).

use rusqlite::Connection;

use crate::clock::unix_now_secs;
use crate::StorageResult;

/// Current schema version applied by this crate.
pub const SCHEMA_VERSION: u32 = 1;

/// SQL for migration v1 — exact freeze from `/docs/04-storage.md` §2 (`observations`).
const MIGRATION_V1_OBSERVATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS observations (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp INTEGER NOT NULL,
    provider_id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    payload JSON NOT NULL,
    confidence REAL DEFAULT 1.0,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_obs_ts ON observations(timestamp);
CREATE INDEX IF NOT EXISTS idx_obs_type_ts ON observations(data_type, timestamp);
"#;

/// Applies pending migrations. Safe to call repeatedly (`IF NOT EXISTS` + version gate).
pub fn run(conn: &Connection) -> StorageResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )?;

    let current: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if current < 1 {
        conn.execute_batch(MIGRATION_V1_OBSERVATIONS)?;
        let now = unix_now_secs()?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![1_i64, now],
        )?;
        tracing::info!(version = 1_u32, "applied storage migration");
    } else {
        // Re-open / re-migrate: keep objects idempotent even if version row exists.
        conn.execute_batch(MIGRATION_V1_OBSERVATIONS)?;
    }

    Ok(())
}
