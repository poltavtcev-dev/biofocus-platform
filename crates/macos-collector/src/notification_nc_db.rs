//! Privacy-safe Notification Center DB reader (ADR-020 / P19-E2).
//!
//! # OS surface (named)
//!
//! Read-only SQLite at the macOS **usernoted / Notification Center** path:
//! - Primary (macOS Sequoia+): `~/Library/Group Containers/group.com.apple.usernoted/db2/db`
//! - Fallback (older layouts): `$DARWIN_USER_DIR/com.apple.notificationcenter/db2/db`
//!
//! # Field allowlist (hard)
//!
//! May `SELECT`: `record.delivered_date`, `app.identifier` (bundle id, **in-memory only**).
//! Must **never** `SELECT` / deserialize: `record.data`, title, body, subtitle, message,
//! userInfo, attachments, or free-form display names.
//!
//! Soft-fail (`None`) when the DB is missing, TCC/authorization denies access, or the
//! schema lacks required allowlisted columns.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use tracing::{debug, warn};

/// Primary Sequoia+ usernoted DB relative to `$HOME`.
pub const USERNOTED_DB_REL: &str = "Library/Group Containers/group.com.apple.usernoted/db2/db";

/// Max new deliveries considered per poll (idle-safe; avoid history dumps).
pub const MAX_DELTA_ROWS: usize = 64;

/// Candidate read-only DB paths (first existing path wins at open time).
#[must_use]
pub fn candidate_nc_db_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            out.push(PathBuf::from(home).join(USERNOTED_DB_REL));
        }
    }
    if let Ok(darwin) = std::env::var("DARWIN_USER_DIR") {
        let p = PathBuf::from(darwin.trim()).join("com.apple.notificationcenter/db2/db");
        if !out.iter().any(|x| x == &p) {
            out.push(p);
        }
    }
    out
}

/// Opens the first readable candidate DB, or `None` (soft-fail).
pub fn open_nc_db_readonly(override_path: Option<&Path>) -> Option<(PathBuf, Connection)> {
    let paths: Vec<PathBuf> = match override_path {
        Some(p) => vec![p.to_path_buf()],
        None => candidate_nc_db_paths(),
    };
    for path in paths {
        if !path.is_file() {
            continue;
        }
        match Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(conn) => {
                if schema_supports_privacy_safe_query(&conn) {
                    return Some((path, conn));
                }
                debug!(
                    path = %path.display(),
                    "notification NC DB schema missing allowlisted columns; soft-fail"
                );
            }
            Err(err) => {
                // TCC / authorization denied is expected without Full Disk Access.
                debug!(
                    path = %path.display(),
                    error = %err,
                    "notification NC DB unreadable; soft-fail idle"
                );
            }
        }
    }
    None
}

fn table_columns(conn: &Connection, table: &str) -> Option<Vec<String>> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .ok()?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .ok()?;
    let mut cols = Vec::new();
    for r in rows {
        cols.push(r.ok()?);
    }
    Some(cols)
}

fn has_col(cols: &[String], name: &str) -> bool {
    cols.iter().any(|c| c.eq_ignore_ascii_case(name))
}

/// Requires `record.delivered_date` + `record.app_id` and `app.identifier` + `app.app_id`.
/// Does **not** require or read `record.data`.
pub fn schema_supports_privacy_safe_query(conn: &Connection) -> bool {
    let Some(record_cols) = table_columns(conn, "record") else {
        return false;
    };
    let Some(app_cols) = table_columns(conn, "app") else {
        return false;
    };
    has_col(&record_cols, "delivered_date")
        && has_col(&record_cols, "app_id")
        && has_col(&app_cols, "app_id")
        && has_col(&app_cols, "identifier")
}

/// One allowlisted delivery row (bundle id in-memory only — never Observation payload).
#[derive(Debug, Clone, PartialEq)]
pub struct NcDeliveryRow {
    /// Poster bundle identifier (in-memory mapping only).
    pub bundle_id: String,
    /// NC `delivered_date` (CF absolute time or whatever the DB stores — ordinal only).
    pub delivered_date: f64,
}

/// Fetch deliveries with `delivered_date > watermark`, ascending, capped.
///
/// SQL selects **only** `app.identifier` + `record.delivered_date` (ADR-020 allowlist).
pub fn query_deliveries_after(
    conn: &Connection,
    watermark: f64,
) -> Result<Vec<NcDeliveryRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT a.identifier, r.delivered_date \
         FROM record AS r \
         INNER JOIN app AS a ON r.app_id = a.app_id \
         WHERE r.delivered_date > ?1 \
         ORDER BY r.delivered_date ASC \
         LIMIT ?2",
    )?;
    let rows = stmt.query_map((watermark, MAX_DELTA_ROWS as i64), |row| {
        let identifier: String = row.get(0)?;
        let delivered_date: f64 = row.get(1)?;
        Ok(NcDeliveryRow {
            bundle_id: identifier,
            delivered_date,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        let row = row?;
        if row.bundle_id.trim().is_empty() {
            continue;
        }
        if !row.delivered_date.is_finite() {
            continue;
        }
        out.push(row);
    }
    Ok(out)
}

/// Max `delivered_date` in DB, or `None` if empty / error.
pub fn max_delivered_date(conn: &Connection) -> Option<f64> {
    conn.query_row(
        "SELECT MAX(delivered_date) FROM record WHERE delivered_date IS NOT NULL",
        [],
        |row| row.get::<_, Option<f64>>(0),
    )
    .ok()
    .flatten()
    .filter(|v| v.is_finite())
}

/// Map bundle id → closed-set `app_kind` (in-memory; never persisted as bundle).
#[must_use]
pub fn app_kind_from_bundle(bundle_id: &str) -> &'static str {
    let b = bundle_id.to_ascii_lowercase();
    if matches!(
        b.as_str(),
        "com.apple.mobilesms" | "com.apple.messages"
    ) || b.contains("telegram")
        || b.contains("whatsapp")
        || b.contains("signal")
        || b.contains("slack")
        || b.contains("discord")
        || b.contains("messages")
    {
        return "messaging";
    }
    if b == "com.apple.mail" || b.ends_with(".mail") || b.contains("outlook") {
        return "mail";
    }
    if b == "com.apple.ical"
        || b == "com.apple.calendar"
        || b.contains("calendar")
        || b.contains("fantastical")
    {
        return "calendar";
    }
    if b.contains("twitter")
        || b.contains("tweetbot")
        || b.contains("facebook")
        || b.contains("instagram")
        || b.contains("linkedin")
        || b.contains("reddit")
    {
        return "social";
    }
    if b.starts_with("com.apple.") {
        return "system";
    }
    if b.contains("spotify") || b.contains("music") || b.contains("podcast") {
        // media apps often notify — coarse
        return "other";
    }
    "other"
}

/// Map bundle id → closed-set `category`.
#[must_use]
pub fn category_from_bundle(bundle_id: &str) -> &'static str {
    match app_kind_from_bundle(bundle_id) {
        "messaging" | "mail" => "communication",
        "calendar" => "calendar",
        "social" => "social",
        "system" => "system",
        _ => "other",
    }
}

/// Build privacy-safe sample fields from a delta of deliveries (dominant bundle).
#[must_use]
pub fn sample_labels_from_rows(rows: &[NcDeliveryRow]) -> (Option<String>, Option<String>) {
    if rows.is_empty() {
        return (None, None);
    }
    let mut counts: HashMap<&str, u64> = HashMap::new();
    for row in rows {
        *counts.entry(row.bundle_id.as_str()).or_insert(0) += 1;
    }
    let dominant = counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)))
        .map(|(k, _)| k);
    match dominant {
        Some(bundle) => (
            Some(category_from_bundle(bundle).to_owned()),
            Some(app_kind_from_bundle(bundle).to_owned()),
        ),
        None => (None, None),
    }
}

/// Creates a minimal fixture DB with **only** allowlisted columns (no `data` blob).
pub fn write_fixture_nc_db(path: &Path, rows: &[(&str, f64)]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE app (app_id INTEGER PRIMARY KEY, identifier VARCHAR);
         CREATE TABLE record (
           rec_id INTEGER PRIMARY KEY,
           app_id INTEGER,
           delivered_date REAL
         );",
    )
    .map_err(|e| e.to_string())?;

    let mut app_ids: HashMap<String, i64> = HashMap::new();
    let mut next_app: i64 = 1;
    let mut next_rec: i64 = 1;
    for (identifier, delivered) in rows {
        let app_id = if let Some(id) = app_ids.get(*identifier) {
            *id
        } else {
            let id = next_app;
            next_app += 1;
            conn.execute(
                "INSERT INTO app (app_id, identifier) VALUES (?1, ?2)",
                (id, *identifier),
            )
            .map_err(|e| e.to_string())?;
            app_ids.insert((*identifier).to_owned(), id);
            id
        };
        conn.execute(
            "INSERT INTO record (rec_id, app_id, delivered_date) VALUES (?1, ?2, ?3)",
            (next_rec, app_id, delivered),
        )
        .map_err(|e| e.to_string())?;
        next_rec += 1;
    }
    Ok(())
}

/// Log-friendly soft-fail reason (no paths at info by default).
#[allow(dead_code)]
pub fn warn_mapping_unavailable(reason: &str) {
    warn!(reason, "notification_event OS mapping unavailable; soft-fail idle");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fixture_query_allowlisted_only() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("db");
        write_fixture_nc_db(
            &path,
            &[
                ("com.apple.mail", 10.0),
                ("com.apple.MobileSMS", 11.0),
                ("com.apple.MobileSMS", 12.0),
            ],
        )
        .expect("fixture");
        let conn = Connection::open(&path).expect("open");
        assert!(schema_supports_privacy_safe_query(&conn));
        let rows = query_deliveries_after(&conn, 10.5).expect("query");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].bundle_id, "com.apple.MobileSMS");
        assert_eq!(app_kind_from_bundle("com.apple.mail"), "mail");
        assert_eq!(app_kind_from_bundle("com.apple.MobileSMS"), "messaging");
        let (cat, kind) = sample_labels_from_rows(&rows);
        assert_eq!(cat.as_deref(), Some("communication"));
        assert_eq!(kind.as_deref(), Some("messaging"));
    }

    #[test]
    fn schema_rejects_missing_identifier() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("bad.db");
        let conn = Connection::open(&path).expect("open");
        conn.execute_batch(
            "CREATE TABLE app (app_id INTEGER PRIMARY KEY);
             CREATE TABLE record (rec_id INTEGER PRIMARY KEY, app_id INTEGER, delivered_date REAL, data BLOB);",
        )
        .expect("schema");
        assert!(!schema_supports_privacy_safe_query(&conn));
    }
}
