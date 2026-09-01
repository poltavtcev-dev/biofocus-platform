//! Desktop IPC for Git watched-roots allowlist (P14-E3-T1 / ADR-014).
//!
//! UI invokes Tauri commands only — never opens the config file or SQLite.
//! Durable store remains `git-watched-roots.toml` (optional `BIOFOCUS_HOME`).

use macos_collector::{
    load_settings_watched_roots, set_settings_watched_roots, WatchedRoots, WatchedRootsSource,
};
use serde::Serialize;

/// CamelCase DTO for Menubar Settings.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitWatchedRootsDto {
    /// Absolute directory roots from the config file (empty → probe soft-fails).
    pub roots: Vec<String>,
    /// `file` / `env` / `empty` — Settings get is file-or-empty (not env as SoT).
    pub source: String,
    /// File schema version when present.
    pub version: Option<u32>,
}

fn to_dto(wr: WatchedRoots) -> GitWatchedRootsDto {
    GitWatchedRootsDto {
        roots: wr
            .roots
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        source: wr.source.as_str().to_string(),
        version: wr.version,
    }
}

/// Load watched roots from the ADR-014 config file (Settings SoT).
///
/// Missing / empty file → `{ roots: [], source: "empty" }`. Does **not** surface
/// env override as editable SoT. Never logs root path strings at info.
pub fn get_git_watched_roots() -> Result<GitWatchedRootsDto, String> {
    let wr = load_settings_watched_roots();
    // Settings get never reports Env — file absent is Empty.
    let wr = if wr.source == WatchedRootsSource::Env {
        WatchedRoots::empty()
    } else {
        wr
    };
    Ok(to_dto(wr))
}

/// Replace watched roots in the ADR-014 config file (absolute / `~/…` only).
///
/// Empty list writes `roots = []` (soft-fail idle for the live probe). UI ↛ SQLite.
pub fn set_git_watched_roots(roots: Vec<String>) -> Result<GitWatchedRootsDto, String> {
    let wr = set_settings_watched_roots(&roots)?;
    Ok(to_dto(wr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use macos_collector::{
        default_watched_roots_path, BIOFOCUS_HOME_ENV, GIT_WATCHED_ROOTS_FILE_NAME,
    };
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn get_set_round_trip_under_biofocus_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().expect("tempdir");
        unsafe {
            std::env::set_var(BIOFOCUS_HOME_ENV, dir.path());
        }

        let empty = get_git_watched_roots().expect("get empty");
        assert!(empty.roots.is_empty());
        assert_eq!(empty.source, "empty");

        let abs = dir.path().join("proj");
        std::fs::create_dir_all(&abs).expect("mkdir");
        let saved = set_git_watched_roots(vec![abs.to_string_lossy().into_owned()]).expect("set");
        assert_eq!(saved.roots.len(), 1);
        assert_eq!(saved.source, "file");
        assert_eq!(saved.version, Some(1));

        let path = default_watched_roots_path().expect("path");
        assert_eq!(path, dir.path().join(GIT_WATCHED_ROOTS_FILE_NAME));

        let cleared = set_git_watched_roots(vec![]).expect("clear");
        assert!(cleared.roots.is_empty());

        unsafe {
            std::env::remove_var(BIOFOCUS_HOME_ENV);
        }
    }

    #[test]
    fn set_rejects_relative_path() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().expect("tempdir");
        unsafe {
            std::env::set_var(BIOFOCUS_HOME_ENV, dir.path());
        }
        let err = set_git_watched_roots(vec!["relative/nope".into()]).expect_err("reject");
        assert!(err.contains("absolute"));
        unsafe {
            std::env::remove_var(BIOFOCUS_HOME_ENV);
        }
    }
}
