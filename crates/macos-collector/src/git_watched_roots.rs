//! ADR-014 Git watched-roots allowlist (local config file; no SQLite).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::{debug, warn};

/// Env override for tests/CI when the config file is **absent** (ADR-014).
/// Colon- or comma-separated absolute paths. File remains source of truth when present.
pub const GIT_WATCHED_ROOTS_ENV: &str = "BIOFOCUS_GIT_WATCHED_ROOTS";

/// File name under `~/.biofocus/`.
pub const GIT_WATCHED_ROOTS_FILE_NAME: &str = "git-watched-roots.toml";

/// Config directory name under the user home.
pub const BIOFOCUS_CONFIG_DIR_NAME: &str = ".biofocus";

#[derive(Debug, Deserialize)]
struct WatchedRootsFile {
    version: u32,
    #[serde(default)]
    roots: Vec<String>,
}

/// Where resolved roots came from (for tests / debug counts only — never log paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchedRootsSource {
    /// `~/.biofocus/git-watched-roots.toml` (or override path in tests).
    File,
    /// `BIOFOCUS_GIT_WATCHED_ROOTS` when the config file is missing.
    Env,
    /// Missing / empty / unreadable → soft-fail idle.
    Empty,
}

/// Resolved allowlist (absolute directory roots only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedRoots {
    pub source: WatchedRootsSource,
    pub roots: Vec<PathBuf>,
    pub version: Option<u32>,
}

impl WatchedRoots {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    #[must_use]
    pub fn empty() -> Self {
        Self {
            source: WatchedRootsSource::Empty,
            roots: Vec::new(),
            version: None,
        }
    }
}

/// Default config path: `$HOME/.biofocus/git-watched-roots.toml`.
#[must_use]
pub fn default_watched_roots_path() -> Option<PathBuf> {
    home_dir().map(|home| {
        home.join(BIOFOCUS_CONFIG_DIR_NAME)
            .join(GIT_WATCHED_ROOTS_FILE_NAME)
    })
}

/// Resolve allowlist with ADR-014 precedence:
/// 1. If config **file exists** → file is SoT (empty/invalid → empty allowlist; **no** env merge).
/// 2. Else if `BIOFOCUS_GIT_WATCHED_ROOTS` set → env roots.
/// 3. Else → empty (soft-fail idle).
#[must_use]
pub fn resolve_watched_roots() -> WatchedRoots {
    match default_watched_roots_path() {
        Some(path) if path.is_file() => load_watched_roots_file(&path),
        _ => load_watched_roots_from_env().unwrap_or_else(WatchedRoots::empty),
    }
}

/// Load + validate a watched-roots TOML file (absolute roots only).
#[must_use]
pub fn load_watched_roots_file(path: &Path) -> WatchedRoots {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            warn!(
                error = %err,
                "git watched-roots config unreadable; soft-fail idle (no whole-disk scan)"
            );
            return WatchedRoots::empty();
        }
    };

    let parsed: WatchedRootsFile = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(err) => {
            warn!(
                error = %err,
                "git watched-roots config invalid; soft-fail idle (no whole-disk scan)"
            );
            return WatchedRoots::empty();
        }
    };

    if parsed.version == 0 {
        warn!("git watched-roots version must be ≥ 1; soft-fail idle");
        return WatchedRoots::empty();
    }

    let roots = normalize_root_strings(&parsed.roots);
    debug!(
        root_count = roots.len(),
        version = parsed.version,
        "git watched-roots loaded from config file"
    );

    if roots.is_empty() {
        WatchedRoots {
            source: WatchedRootsSource::Empty,
            roots,
            version: Some(parsed.version),
        }
    } else {
        WatchedRoots {
            source: WatchedRootsSource::File,
            roots,
            version: Some(parsed.version),
        }
    }
}

fn load_watched_roots_from_env() -> Option<WatchedRoots> {
    let raw = std::env::var(GIT_WATCHED_ROOTS_ENV).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let parts: Vec<String> = raw
        .split(|c| c == ':' || c == ',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let roots = normalize_root_strings(&parts);
    debug!(
        root_count = roots.len(),
        "git watched-roots loaded from env override (config file absent)"
    );
    if roots.is_empty() {
        Some(WatchedRoots::empty())
    } else {
        Some(WatchedRoots {
            source: WatchedRootsSource::Env,
            roots,
            version: None,
        })
    }
}

fn normalize_root_strings(raw: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut rejected = 0u32;
    for s in raw {
        let path = expand_user_path(s.trim());
        if !path.is_absolute() {
            rejected += 1;
            continue;
        }
        // Dedup while preserving order.
        if out.iter().any(|p| p == &path) {
            continue;
        }
        out.push(path);
    }
    if rejected > 0 {
        debug!(
            rejected,
            "skipped non-absolute git watched-roots entries"
        );
    }
    out
}

fn expand_user_path(raw: &str) -> PathBuf {
    if raw == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return match home_dir() {
            Some(home) => home.join(rest),
            None => PathBuf::from(raw),
        };
    }
    PathBuf::from(raw)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_file_without_env_is_empty() {
        // resolve_watched_roots depends on real HOME; unit-test load helpers instead.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.toml");
        assert!(!path.exists());
        // load_watched_roots_file on missing → empty via read error
        let loaded = load_watched_roots_file(&path);
        assert!(loaded.is_empty());
        assert_eq!(loaded.source, WatchedRootsSource::Empty);
    }

    #[test]
    fn parses_absolute_roots_and_rejects_relative() {
        let dir = tempfile::tempdir().expect("tempdir");
        let abs = dir.path().join("proj");
        std::fs::create_dir_all(&abs).expect("mkdir");
        let path = dir.path().join(GIT_WATCHED_ROOTS_FILE_NAME);
        let mut f = std::fs::File::create(&path).expect("create");
        writeln!(
            f,
            "version = 1\nroots = [\n  \"{}\",\n  \"relative/nope\",\n]\n",
            abs.display()
        )
        .expect("write");

        let loaded = load_watched_roots_file(&path);
        assert_eq!(loaded.source, WatchedRootsSource::File);
        assert_eq!(loaded.version, Some(1));
        assert_eq!(loaded.roots, vec![abs]);
    }

    #[test]
    fn empty_roots_array_is_soft_fail() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(GIT_WATCHED_ROOTS_FILE_NAME);
        std::fs::write(&path, "version = 1\nroots = []\n").expect("write");
        let loaded = load_watched_roots_file(&path);
        assert!(loaded.is_empty());
        assert_eq!(loaded.source, WatchedRootsSource::Empty);
    }
}
