//! SQLite connection open + WAL pragmas + schema migrate on open.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::migrate;
use crate::{StorageError, StorageResult};

/// Open SQLite connection configured for BioFocus Local-First storage.
pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    /// Opens (or creates) a database at `path`, applies WAL pragmas, then runs migrations.
    ///
    /// **Single migration path:** every successful [`Database::open`] / [`Database::open_in_memory`]
    /// calls [`Database::migrate`]. Callers do not need a separate migrate step.
    ///
    /// Parent directories are created when missing. Pass a custom path in tests
    /// (temp dir); production hosts typically use [`crate::default_db_path`].
    pub fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        apply_pragmas(&conn)?;
        migrate::run(&conn)?;

        tracing::info!(path = %path.display(), "opened BioFocus SQLite database");
        Ok(Self { conn, path })
    }

    /// Opens an in-memory database with the same pragmas and migrations.
    pub fn open_in_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        apply_pragmas(&conn)?;
        migrate::run(&conn)?;
        Ok(Self {
            conn,
            path: PathBuf::from(":memory:"),
        })
    }

    /// Re-applies schema migrations (idempotent). Prefer relying on open; exposed for tests.
    pub fn migrate(&self) -> StorageResult<()> {
        migrate::run(&self.conn)
    }

    /// Filesystem path of this database (`:memory:` for in-memory).
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Shared access to the underlying connection (repositories / tests).
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Reads current `journal_mode` (normalized lowercase).
    pub fn journal_mode(&self) -> StorageResult<String> {
        let mode: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        Ok(mode.to_ascii_lowercase())
    }

    /// Reads current `synchronous` setting as integer string.
    pub fn synchronous(&self) -> StorageResult<i64> {
        let value: i64 = self
            .conn
            .query_row("PRAGMA synchronous", [], |row| row.get(0))?;
        Ok(value)
    }
}

fn apply_pragmas(conn: &Connection) -> StorageResult<()> {
    // `PRAGMA journal_mode=WAL` returns the mode name as a result row.
    let mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    let mode = mode.to_ascii_lowercase();
    // In-memory DBs may report `memory` instead of `wal`.
    if mode != "wal" && mode != "memory" {
        return Err(StorageError::PragmaMismatch {
            pragma: "journal_mode",
            expected: "wal".into(),
            actual: mode,
        });
    }

    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;

    // NORMAL == 1 in SQLite.
    let sync: i64 = conn.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    if sync != 1 {
        return Err(StorageError::PragmaMismatch {
            pragma: "synchronous",
            expected: "1 (NORMAL)".into(),
            actual: sync.to_string(),
        });
    }

    Ok(())
}
