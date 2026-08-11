//! Opt-in user-declared health context (ADR-024 / P23-E2).
//!
//! Local config under `~/.biofocus/health-context.toml` (Git-folders pattern).
//! Consumed by prompt packs / reports as **interpret-only** context — never
//! invents diagnoses, never rewrites Feature math, never syncs to cloud by default.
//! Missing / empty / invalid → no injection (`None`).

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Optional override for the BioFocus config directory (tests / custom layouts).
pub const BIOFOCUS_HOME_ENV: &str = "BIOFOCUS_HOME";

/// File name under `~/.biofocus/` (or under `BIOFOCUS_HOME` when set).
pub const HEALTH_CONTEXT_FILE_NAME: &str = "health-context.toml";

/// Config directory name under the user home.
pub const BIOFOCUS_CONFIG_DIR_NAME: &str = ".biofocus";

/// Closed-set curated condition ids the user may declare (v1).
pub const V1_HEALTH_CONDITION_IDS: &[&str] = &[
    "sleep_sensitive",
    "migraine_prone",
    "caffeine_sensitive",
];

#[derive(Debug, Deserialize)]
struct HealthContextFile {
    #[serde(default)]
    health_context: HealthContextSection,
}

#[derive(Debug, Default, Deserialize)]
struct HealthContextSection {
    #[serde(default)]
    conditions: Vec<String>,
    #[serde(default)]
    note: String,
}

/// User-declared health context ready for prompt/report injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthContext {
    /// Closed-set condition ids (unknown ids filtered out).
    pub conditions: Vec<String>,
    /// Optional free-text note (local-only; trimmed; may be empty).
    pub note: String,
}

impl HealthContext {
    /// `true` when there is nothing to inject.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty() && self.note.trim().is_empty()
    }
}

/// Default config path.
///
/// 1. `$BIOFOCUS_HOME/health-context.toml` if `BIOFOCUS_HOME` is set
/// 2. otherwise `$HOME/.biofocus/health-context.toml`
#[must_use]
pub fn default_health_context_path() -> Option<PathBuf> {
    if let Some(home) = biofocus_home_override() {
        return Some(home.join(HEALTH_CONTEXT_FILE_NAME));
    }
    user_home_dir().map(|home| {
        home.join(BIOFOCUS_CONFIG_DIR_NAME)
            .join(HEALTH_CONTEXT_FILE_NAME)
    })
}

/// Load opt-in health context from the default path.
///
/// Missing file / empty conditions+note / unreadable / invalid → [`None`]
/// (no injection).
#[must_use]
pub fn load_health_context() -> Option<HealthContext> {
    let path = default_health_context_path()?;
    load_health_context_file(&path)
}

/// Load + validate a health-context TOML file.
#[must_use]
pub fn load_health_context_file(path: &Path) -> Option<HealthContext> {
    if !path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(path).ok()?;
    parse_health_context_toml(&raw)
}

/// Parse TOML body into a non-empty [`HealthContext`].
#[must_use]
pub fn parse_health_context_toml(raw: &str) -> Option<HealthContext> {
    let parsed: HealthContextFile = toml::from_str(raw).ok()?;
    let mut conditions = Vec::new();
    for id in parsed.health_context.conditions {
        let id = id.trim().to_owned();
        if id.is_empty() {
            continue;
        }
        if V1_HEALTH_CONDITION_IDS.contains(&id.as_str()) && !conditions.iter().any(|c| c == &id)
        {
            conditions.push(id);
        }
    }
    let note = parsed.health_context.note.trim().to_owned();
    let ctx = HealthContext { conditions, note };
    if ctx.is_empty() {
        None
    } else {
        Some(ctx)
    }
}

/// `true` when `id` is in the v1 closed set.
#[must_use]
pub fn is_v1_health_condition(id: &str) -> bool {
    V1_HEALTH_CONDITION_IDS.contains(&id)
}

fn biofocus_home_override() -> Option<PathBuf> {
    std::env::var_os(BIOFOCUS_HOME_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_toml_yields_none() {
        let raw = r#"
[health_context]
conditions = []
note = ""
"#;
        assert!(parse_health_context_toml(raw).is_none());
    }

    #[test]
    fn known_conditions_and_note() {
        let raw = r#"
[health_context]
conditions = ["sleep_sensitive", "caffeine_sensitive", "not_a_real_id"]
note = "  I already know late caffeine hits me  "
"#;
        let ctx = parse_health_context_toml(raw).expect("ctx");
        assert_eq!(
            ctx.conditions,
            vec!["sleep_sensitive".to_owned(), "caffeine_sensitive".to_owned()]
        );
        assert_eq!(ctx.note, "I already know late caffeine hits me");
        assert!(!ctx.is_empty());
    }

    #[test]
    fn unknown_only_yields_none_without_note() {
        let raw = r#"
[health_context]
conditions = ["invented_disease"]
note = ""
"#;
        assert!(parse_health_context_toml(raw).is_none());
    }

    #[test]
    fn load_from_explicit_path_file() {
        let dir = std::env::temp_dir().join(format!(
            "biofocus-health-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join(HEALTH_CONTEXT_FILE_NAME);
        std::fs::write(
            &path,
            r#"
[health_context]
conditions = ["migraine_prone"]
note = ""
"#,
        )
        .expect("write");
        let loaded = load_health_context_file(&path).expect("load");
        assert_eq!(loaded.conditions, vec!["migraine_prone".to_owned()]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
