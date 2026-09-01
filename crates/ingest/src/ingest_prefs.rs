//! Persisted opt-in LAN bind preference (`~/.biofocus/ingest_lan_enabled`).
//!
//! Env knobs ([`super::config::INGEST_LAN_ENV`], [`super::config::INGEST_BIND_HOST_ENV`])
//! always win over the on-disk flag (ADR-005).

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::{IngestError, IngestResult};
use crate::token::biofocus_config_dir;

/// File name under the BioFocus config directory.
pub const INGEST_LAN_PREFS_FILE: &str = "ingest_lan_enabled";

/// Returns `$BIOFOCUS_HOME/ingest_lan_enabled` or `$HOME/.biofocus/ingest_lan_enabled`.
pub fn ingest_lan_prefs_path() -> IngestResult<PathBuf> {
    Ok(biofocus_config_dir()?.join(INGEST_LAN_PREFS_FILE))
}

/// Reads persisted LAN opt-in. Missing or unreadable file → `false`.
#[must_use]
pub fn read_persisted_lan_enabled() -> bool {
    match ingest_lan_prefs_path() {
        Ok(path) => read_lan_flag_file(&path),
        Err(_) => false,
    }
}

/// Persists LAN opt-in (`true` writes `1`; `false` removes the file).
pub fn write_persisted_lan_enabled(enabled: bool) -> IngestResult<()> {
    let path = ingest_lan_prefs_path()?;
    if !enabled {
        if path.exists() {
            fs::remove_file(&path).map_err(|source| IngestError::TokenIo {
                path: path.display().to_string(),
                source,
            })?;
        }
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| IngestError::TokenIo {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|source| IngestError::TokenIo {
            path: path.display().to_string(),
            source,
        })?;
    file.write_all(b"1")
        .map_err(|source| IngestError::TokenIo {
            path: path.display().to_string(),
            source,
        })?;
    Ok(())
}

/// True when env explicitly sets bind host or LAN flag (persisted file ignored at runtime).
pub fn lan_preference_overridden_by_env() -> bool {
    if let Ok(raw) = std::env::var(super::config::INGEST_BIND_HOST_ENV) {
        if !raw.trim().is_empty() {
            return true;
        }
    }
    super::config::lan_flag_enabled(std::env::var_os(super::config::INGEST_LAN_ENV))
}

fn read_lan_flag_file(path: &std::path::Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lock::ENV_LOCK;

    fn with_temp_biofocus_home() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let home = dir.path().to_path_buf();
        unsafe {
            std::env::set_var(crate::token::BIOFOCUS_HOME_ENV, &home);
        }
        (dir, home.join(INGEST_LAN_PREFS_FILE))
    }

    fn clear_biofocus_home_env() {
        unsafe {
            std::env::remove_var(crate::token::BIOFOCUS_HOME_ENV);
        }
    }

    #[test]
    fn read_missing_prefs_is_false() {
        let _lock = ENV_LOCK.lock().expect("lock");
        let _dir = with_temp_biofocus_home();
        assert!(!read_persisted_lan_enabled());
        clear_biofocus_home_env();
    }

    #[test]
    fn write_and_read_persisted_lan() {
        let _lock = ENV_LOCK.lock().expect("lock");
        let (_dir, path) = with_temp_biofocus_home();
        write_persisted_lan_enabled(true).expect("write");
        assert!(path.is_file());
        assert!(read_persisted_lan_enabled());
        write_persisted_lan_enabled(false).expect("clear");
        assert!(!path.exists());
        assert!(!read_persisted_lan_enabled());
        clear_biofocus_home_env();
    }

    #[test]
    fn persisted_lan_pref_enables_unspecified_bind() {
        let _lock = ENV_LOCK.lock().expect("lock");
        let dir = tempfile::tempdir().expect("tempdir");
        unsafe {
            std::env::set_var(crate::token::BIOFOCUS_HOME_ENV, dir.path());
        }
        write_persisted_lan_enabled(true).expect("write");
        assert!(read_persisted_lan_enabled());
        let host = crate::config::resolve_bind_host_from_env(|_| None).expect("ok");
        assert_eq!(host, crate::config::INGEST_LAN_BIND_HOST);
        write_persisted_lan_enabled(false).expect("clear");
        clear_biofocus_home_env();
    }
}
