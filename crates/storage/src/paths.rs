//! Default on-disk locations for BioFocus data.

use std::path::PathBuf;

use crate::{StorageError, StorageResult};

/// Relative data directory under the user home (`docs/04-storage.md`).
pub const BIOFOCUS_DATA_DIR: &str = ".biofocus/data";

/// Default SQLite file name.
pub const DEFAULT_DB_FILE_NAME: &str = "biofocus_main.db";

/// Returns `~/.biofocus/data/biofocus_main.db`.
pub fn default_db_path() -> StorageResult<PathBuf> {
    let home = dirs_home().ok_or(StorageError::HomeDirUnavailable)?;
    Ok(home.join(BIOFOCUS_DATA_DIR).join(DEFAULT_DB_FILE_NAME))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
