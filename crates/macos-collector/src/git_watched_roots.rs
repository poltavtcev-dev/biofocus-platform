//! ADR-014 Git watched-roots allowlist (local config file; no SQLite).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Env override for tests/CI when the config file is **absent** (ADR-014).
/// Colon- or comma-separated absolute paths. File remains source of truth when present.
pub const GIT_WATCHED_ROOTS_ENV: &str = "BIOFOCUS_GIT_WATCHED_ROOTS";

/// Optional override for the BioFocus config directory (tests / custom layouts).
///
/// When set, the allowlist path is `$BIOFOCUS_HOME/git-watched-roots.toml`
/// (same `BIOFOCUS_HOME` convention as the pairing token).
pub const BIOFOCUS_HOME_ENV: &str = "BIOFOCUS_HOME";

/// File name under `~/.biofocus/` (or under `BIOFOCUS_HOME` when set).
pub const GIT_WATCHED_ROOTS_FILE_NAME: &str = "git-watched-roots.toml";

/// Config directory name under the user home.
pub const BIOFOCUS_CONFIG_DIR_NAME: &str = ".biofocus";

/// On-disk schema version written by Settings / IPC.
pub const WATCHED_ROOTS_FILE_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
struct WatchedRootsFile {
    version: u32,
    #[serde(default)]
    roots: Vec<String>,
}

/// Where resolved roots came from (for tests / debug counts only — never log paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchedRootsSource {
    /// Config file (or override path in tests).
    File,
    /// `BIOFOCUS_GIT_WATCHED_ROOTS` when the config file is missing.
    Env,
    /// Missing / empty / unreadable → soft-fail idle.
    Empty,
}

impl WatchedRootsSource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Env => "env",
            Self::Empty => "empty",
        }
    }
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

/// Default config path.
///
/// Resolution:
/// 1. `$BIOFOCUS_HOME/git-watched-roots.toml` if `BIOFOCUS_HOME` is set and non-empty
/// 2. otherwise `$HOME/.biofocus/git-watched-roots.toml`
#[must_use]
pub fn default_watched_roots_path() -> Option<PathBuf> {
    if let Some(home) = biofocus_home_override() {
        return Some(home.join(GIT_WATCHED_ROOTS_FILE_NAME));
    }
    user_home_dir().map(|home| {
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

/// Settings/IPC get: load the **config file only** (not the env override).
///
/// Missing file → empty (soft-fail). Env roots are probe-only when no file exists.
#[must_use]
pub fn load_settings_watched_roots() -> WatchedRoots {
    match default_watched_roots_path() {
        Some(path) if path.is_file() => load_watched_roots_file(&path),
        _ => WatchedRoots::empty(),
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

/// Validate UI/IPC root inputs: absolute paths (or `~/…`); reject relatives.
///
/// Empty strings are skipped. Empty result after filtering is OK (soft-fail allowlist).
pub fn validate_watched_root_inputs(raw: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for s in raw {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        let path = expand_user_path(t);
        if !path.is_absolute() {
            return Err(
                "Each folder must be an absolute path (or start with ~/).".into(),
            );
        }
        if out.iter().any(|p| p == &path) {
            continue;
        }
        out.push(path);
    }
    Ok(out)
}

/// Write the ADR-014 config file (`version` + absolute `roots`). Creates parent dirs.
///
/// Empty `roots` writes `roots = []` — probe soft-fails idle. Never logs path strings.
pub fn write_watched_roots_file(path: &Path, roots: &[PathBuf]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            "Could not create the BioFocus config folder.".to_string()
        })?;
    }

    let file = WatchedRootsFile {
        version: WATCHED_ROOTS_FILE_VERSION,
        roots: roots
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
    };
    let body = toml::to_string_pretty(&file)
        .map_err(|_| "Could not format the watched-folders config.".to_string())?;
    std::fs::write(path, body)
        .map_err(|_| "Could not save watched folders.".to_string())?;
    debug!(
        root_count = roots.len(),
        "git watched-roots config saved"
    );
    Ok(())
}

/// Settings/IPC set: validate, write default config path, return reloaded file state.
pub fn set_settings_watched_roots(raw: &[String]) -> Result<WatchedRoots, String> {
    let roots = validate_watched_root_inputs(raw)?;
    let path = default_watched_roots_path()
        .ok_or_else(|| "Could not resolve the BioFocus config folder.".to_string())?;
    write_watched_roots_file(&path, &roots)?;
    Ok(load_watched_roots_file(&path))
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
        return user_home_dir().unwrap_or_else(|| PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return match user_home_dir() {
            Some(home) => home.join(rest),
            None => PathBuf::from(raw),
        };
    }
    PathBuf::from(raw)
}

fn biofocus_home_override() -> Option<PathBuf> {
    let home = std::env::var(BIOFOCUS_HOME_ENV).ok()?;
    let home = home.trim();
    if home.is_empty() {
        return None;
    }
    Some(PathBuf::from(home))
}

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn missing_file_without_env_is_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.toml");
        assert!(!path.exists());
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

    #[test]
    fn write_and_load_round_trip_under_biofocus_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().expect("tempdir");
        // SAFETY: serialized by ENV_LOCK; restored below.
        unsafe {
            std::env::set_var(BIOFOCUS_HOME_ENV, dir.path());
        }

        let abs = dir.path().join("code");
        std::fs::create_dir_all(&abs).expect("mkdir");
        let saved = set_settings_watched_roots(&[abs.to_string_lossy().into_owned()])
            .expect("set");
        assert_eq!(saved.source, WatchedRootsSource::File);
        assert_eq!(saved.roots, vec![abs.clone()]);

        let path = default_watched_roots_path().expect("path");
        assert_eq!(path, dir.path().join(GIT_WATCHED_ROOTS_FILE_NAME));
        assert!(path.is_file());

        let empty = set_settings_watched_roots(&[]).expect("clear");
        assert!(empty.is_empty());
        assert_eq!(empty.source, WatchedRootsSource::Empty);
        let body = std::fs::read_to_string(&path).expect("read");
        assert!(body.contains("roots"));

        unsafe {
            std::env::remove_var(BIOFOCUS_HOME_ENV);
        }
    }

    #[test]
    fn validate_rejects_relative_paths() {
        let err = validate_watched_root_inputs(&["relative/nope".into()]).expect_err("reject");
        assert!(err.contains("absolute"));
    }
}
