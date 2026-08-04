//! Migration tests (P1-E2-T2).

use rusqlite::Connection;
use storage::{Database, SCHEMA_VERSION};

#[test]
fn open_creates_observations_table_and_indexes() {
    let db = Database::open_in_memory().expect("open");
    let conn = db.connection();

    assert_eq!(table_columns(conn, "observations"), expected_observation_columns());
    assert!(index_exists(conn, "idx_obs_ts"));
    assert!(index_exists(conn, "idx_obs_type_ts"));
    assert_eq!(schema_version(conn), SCHEMA_VERSION);
}

#[test]
fn migrate_twice_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("biofocus_main.db");

    let db = Database::open(&path).expect("first open");
    db.migrate().expect("explicit migrate #1");
    db.migrate().expect("explicit migrate #2");

    let columns_before = table_columns(db.connection(), "observations");
    drop(db);

    let db2 = Database::open(&path).expect("second open");
    db2.migrate().expect("migrate after reopen");
    let columns_after = table_columns(db2.connection(), "observations");

    assert_eq!(columns_before, columns_after);
    assert_eq!(columns_after, expected_observation_columns());
    assert_eq!(schema_version(db2.connection()), 1);
    assert!(index_exists(db2.connection(), "idx_obs_ts"));
    assert!(index_exists(db2.connection(), "idx_obs_type_ts"));
}

fn expected_observation_columns() -> Vec<(String, String)> {
    vec![
        ("id".into(), "TEXT".into()),
        ("timestamp".into(), "INTEGER".into()),
        ("provider_id".into(), "TEXT".into()),
        ("data_type".into(), "TEXT".into()),
        ("payload".into(), "JSON".into()),
        ("confidence".into(), "REAL".into()),
        ("created_at".into(), "INTEGER".into()),
    ]
}

fn table_columns(conn: &Connection, table: &str) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("prepare table_info");
    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            Ok((name, col_type))
        })
        .expect("query table_info");
    rows.map(|r| r.expect("row")).collect()
}

fn index_exists(conn: &Connection, name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
            [name],
            |row| row.get(0),
        )
        .expect("sqlite_master");
    count == 1
}

fn schema_version(conn: &Connection) -> u32 {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
    .expect("schema version")
}
