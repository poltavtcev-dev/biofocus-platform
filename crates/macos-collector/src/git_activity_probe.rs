//! Git activity probe: coarse activity_kind + optional event_count (no paths).
//!
//! Live [`SystemGitActivityProbe`] watches only ADR-014 allowlisted roots
//! (config file / optional env when file absent). Never emits paths, remotes,
//! branch names, SHAs, messages, diffs, or authors.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use tracing::debug;

use crate::error::CollectorResult;
use crate::git_watched_roots::{resolve_watched_roots, WatchedRoots};

/// Cap discovered repos per poll (idle-safe; personal dogfood scale).
const MAX_REPOS: usize = 64;
/// Max directory depth under each allowlisted root when searching for `.git`.
const MAX_WALK_DEPTH: u32 = 4;
/// Directory names skipped during nested discovery (performance / noise).
const SKIP_DIR_NAMES: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "Pods",
    "build",
    "dist",
    ".venv",
    "venv",
];

/// Privacy-safe Git activity sample (no path / remote / branch / SHA / message).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitActivitySample {
    /// Coarse v1 activity kind (`commit` / `checkout` / `sync` / `other` / `idle` / `unknown`).
    pub activity_kind: String,
    /// Optional batched event count since last emit (≥ 1 when present).
    pub event_count: Option<u64>,
}

impl GitActivitySample {
    /// Identity key used to detect changes (kind + count).
    #[must_use]
    pub fn identity_key(&self) -> String {
        match self.event_count {
            Some(n) => format!("{}|{n}", self.activity_kind),
            None => format!("{}|", self.activity_kind),
        }
    }
}

/// Injectable Git activity source (tests / OS stub).
pub trait GitActivityProbe: Send + Sync {
    /// Returns the current coarse sample, or `None` if unavailable.
    fn current(&self) -> CollectorResult<Option<GitActivitySample>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedGitActivityProbe {
    samples: Mutex<Vec<Option<GitActivitySample>>>,
}

impl ScriptedGitActivityProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture queue (consumed FIFO by [`GitActivityProbe::current`]).
    pub fn set_samples(&self, samples: Vec<Option<GitActivitySample>>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        *guard = samples;
    }

    /// Append one fixture sample.
    pub fn push(&self, sample: Option<GitActivitySample>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(sample);
    }
}

impl GitActivityProbe for ScriptedGitActivityProbe {
    fn current(&self) -> CollectorResult<Option<GitActivitySample>> {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Opaque per-repo FS fingerprint (paths never leave this module into Observations/logs).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoSnapshot {
    head: Vec<u8>,
    head_log_len: u64,
    fetch_mtime: Option<SystemTime>,
    index_mtime: Option<SystemTime>,
}

#[derive(Debug, Default)]
struct LiveProbeState {
    /// Opaque repo-root keys → last snapshot. Keys are path strings held in-process only.
    baselines: HashMap<String, RepoSnapshot>,
    primed: bool,
}

/// Production probe: allowlisted roots only (ADR-014). Empty allowlist → soft-fail idle.
#[derive(Debug)]
pub struct SystemGitActivityProbe {
    /// When set, skip file/env resolution (unit / integration fixtures).
    fixed_roots: Option<Vec<PathBuf>>,
    state: Mutex<LiveProbeState>,
}

impl Default for SystemGitActivityProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemGitActivityProbe {
    /// Resolve roots from ADR-014 config / env each poll.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fixed_roots: None,
            state: Mutex::new(LiveProbeState::default()),
        }
    }

    /// Fixed allowlist roots (tests). Empty → always soft-fail idle.
    #[must_use]
    pub fn with_roots(roots: Vec<PathBuf>) -> Self {
        Self {
            fixed_roots: Some(roots),
            state: Mutex::new(LiveProbeState::default()),
        }
    }
}

impl GitActivityProbe for SystemGitActivityProbe {
    fn current(&self) -> CollectorResult<Option<GitActivitySample>> {
        let roots = match &self.fixed_roots {
            Some(r) => WatchedRoots {
                source: if r.is_empty() {
                    crate::git_watched_roots::WatchedRootsSource::Empty
                } else {
                    crate::git_watched_roots::WatchedRootsSource::File
                },
                roots: r.clone(),
                version: Some(1),
            },
            None => resolve_watched_roots(),
        };

        if roots.is_empty() {
            debug!("git_activity live probe idle (empty watched-roots allowlist)");
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.baselines.clear();
            guard.primed = false;
            return Ok(None);
        }

        let repos = discover_repos(&roots.roots, MAX_REPOS, MAX_WALK_DEPTH);
        debug!(
            root_count = roots.roots.len(),
            repo_count = repos.len(),
            "git_activity scanning allowlisted roots"
        );

        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        Ok(diff_repos(&mut guard, &repos))
    }
}

fn discover_repos(roots: &[PathBuf], max_repos: usize, max_depth: u32) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        if out.len() >= max_repos {
            break;
        }
        if !root.is_dir() {
            continue;
        }
        if is_git_workdir(root) {
            push_unique(&mut out, root.clone());
        }
        walk_for_git(root, 0, max_depth, max_repos, &mut out);
    }
    out
}

fn walk_for_git(
    dir: &Path,
    depth: u32,
    max_depth: u32,
    max_repos: usize,
    out: &mut Vec<PathBuf>,
) {
    if depth >= max_depth || out.len() >= max_repos {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if out.len() >= max_repos {
            return;
        }
        let path = entry.path();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || SKIP_DIR_NAMES.iter().any(|s| *s == name) {
            continue;
        }
        if is_git_workdir(&path) {
            push_unique(out, path);
            continue;
        }
        walk_for_git(&path, depth + 1, max_depth, max_repos, out);
    }
}

fn push_unique(out: &mut Vec<PathBuf>, path: PathBuf) {
    if out.iter().any(|p| p == &path) {
        return;
    }
    out.push(path);
}

fn is_git_workdir(path: &Path) -> bool {
    let git = path.join(".git");
    git.is_dir() || git.is_file()
}

fn git_dir_path(workdir: &Path) -> Option<PathBuf> {
    let git = workdir.join(".git");
    if git.is_dir() {
        return Some(git);
    }
    if git.is_file() {
        // gitfile: "gitdir: /absolute/path" — resolve for worktrees; ignore relative for safety.
        let text = std::fs::read_to_string(&git).ok()?;
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("gitdir:") {
                let p = PathBuf::from(rest.trim());
                if p.is_absolute() && p.is_dir() {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn snapshot_repo(workdir: &Path) -> Option<(String, RepoSnapshot)> {
    let git = git_dir_path(workdir)?;
    let head = std::fs::read(git.join("HEAD")).ok()?;
    let head_log_len = std::fs::metadata(git.join("logs").join("HEAD"))
        .map(|m| m.len())
        .unwrap_or(0);
    let fetch_mtime = std::fs::metadata(git.join("FETCH_HEAD"))
        .ok()
        .and_then(|m| m.modified().ok());
    let index_mtime = std::fs::metadata(git.join("index"))
        .ok()
        .and_then(|m| m.modified().ok());

    let key = workdir.to_string_lossy().into_owned();
    Some((
        key,
        RepoSnapshot {
            head,
            head_log_len,
            fetch_mtime,
            index_mtime,
        },
    ))
}

fn diff_repos(state: &mut LiveProbeState, repos: &[PathBuf]) -> Option<GitActivitySample> {
    let mut next: HashMap<String, RepoSnapshot> = HashMap::new();
    let mut kinds: Vec<&'static str> = Vec::new();

    for workdir in repos {
        let Some((key, snap)) = snapshot_repo(workdir) else {
            continue;
        };
        if state.primed {
            if let Some(prev) = state.baselines.get(&key) {
                if let Some(kind) = classify_change(prev, &snap, workdir) {
                    kinds.push(kind);
                }
            } else {
                // New repo appeared under allowlist — coarse "other".
                kinds.push(bio_spec::ACTIVITY_KIND_OTHER);
            }
        }
        next.insert(key, snap);
    }

    // Repos that disappeared are ignored (no emit).
    state.baselines = next;

    if !state.primed {
        state.primed = true;
        return None;
    }

    if kinds.is_empty() {
        return None;
    }

    let activity_kind = pick_dominant_kind(&kinds).to_string();
    let event_count = Some(kinds.len() as u64);
    Some(GitActivitySample {
        activity_kind,
        event_count,
    })
}

fn classify_change(
    prev: &RepoSnapshot,
    next: &RepoSnapshot,
    workdir: &Path,
) -> Option<&'static str> {
    let fetch_changed = prev.fetch_mtime != next.fetch_mtime && next.fetch_mtime.is_some();
    if fetch_changed {
        return Some(bio_spec::ACTIVITY_KIND_SYNC);
    }

    let head_changed = prev.head != next.head;
    let log_changed = prev.head_log_len != next.head_log_len;
    if head_changed || log_changed {
        if let Some(git) = git_dir_path(workdir) {
            if let Some(kind) = classify_reflog_tail(&git) {
                return Some(kind);
            }
        }
        if head_changed {
            return Some(bio_spec::ACTIVITY_KIND_CHECKOUT);
        }
        return Some(bio_spec::ACTIVITY_KIND_OTHER);
    }

    if prev.index_mtime != next.index_mtime {
        return Some(bio_spec::ACTIVITY_KIND_OTHER);
    }

    None
}

/// Parse last reflog action keyword only — never return message / SHA / branch text.
fn classify_reflog_tail(git_dir: &Path) -> Option<&'static str> {
    let data = std::fs::read(git_dir.join("logs").join("HEAD")).ok()?;
    if data.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&data);
    let line = text.lines().next_back()?;
    // Format: "...\t<action>: <details>" — take action token only.
    let msg = line.rsplit('\t').next().unwrap_or(line);
    let action = msg.split(':').next().unwrap_or("").trim().to_ascii_lowercase();
    if action.starts_with("commit") {
        return Some(bio_spec::ACTIVITY_KIND_COMMIT);
    }
    if action.starts_with("checkout") {
        return Some(bio_spec::ACTIVITY_KIND_CHECKOUT);
    }
    if action.starts_with("pull")
        || action.starts_with("fetch")
        || action.starts_with("push")
        || action.starts_with("clone")
    {
        return Some(bio_spec::ACTIVITY_KIND_SYNC);
    }
    if action.is_empty() {
        return None;
    }
    Some(bio_spec::ACTIVITY_KIND_OTHER)
}

fn pick_dominant_kind(kinds: &[&'static str]) -> &'static str {
    let rank = |k: &str| -> u8 {
        match k {
            bio_spec::ACTIVITY_KIND_COMMIT => 4,
            bio_spec::ACTIVITY_KIND_SYNC => 3,
            bio_spec::ACTIVITY_KIND_CHECKOUT => 2,
            bio_spec::ACTIVITY_KIND_OTHER => 1,
            _ => 0,
        }
    };
    kinds
        .iter()
        .copied()
        .max_by_key(|k| rank(k))
        .unwrap_or(bio_spec::ACTIVITY_KIND_OTHER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn scripted_probe_fifo() {
        let probe = ScriptedGitActivityProbe::new();
        probe.push(Some(GitActivitySample {
            activity_kind: "commit".into(),
            event_count: Some(1),
        }));
        probe.push(None);
        let first = probe.current().expect("ok").expect("sample");
        assert_eq!(first.activity_kind, "commit");
        assert_eq!(first.event_count, Some(1));
        assert!(probe.current().expect("ok").is_none());
        assert!(probe.current().expect("ok").is_none());
    }

    #[test]
    fn system_probe_empty_allowlist_soft_fails_idle() {
        let probe = SystemGitActivityProbe::with_roots(Vec::new());
        let sample = probe.current().expect("ok");
        assert!(sample.is_none());
    }

    #[test]
    fn identity_key_includes_count() {
        let a = GitActivitySample {
            activity_kind: "sync".into(),
            event_count: Some(1),
        };
        let b = GitActivitySample {
            activity_kind: "sync".into(),
            event_count: Some(3),
        };
        assert_ne!(a.identity_key(), b.identity_key());
    }

    #[test]
    fn live_probe_emits_commit_under_fixture_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir");

        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "dogfood@example.com"]);
        run_git(&repo, &["config", "user.name", "Dogfood"]);
        std::fs::write(repo.join("a.txt"), "one\n").expect("write");
        run_git(&repo, &["add", "a.txt"]);
        run_git(&repo, &["commit", "-m", "init"]);

        let probe = SystemGitActivityProbe::with_roots(vec![dir.path().to_path_buf()]);
        // Prime baselines — no emit.
        assert!(probe.current().expect("ok").is_none());

        std::fs::write(repo.join("a.txt"), "two\n").expect("write");
        run_git(&repo, &["add", "a.txt"]);
        run_git(&repo, &["commit", "-m", "second"]);

        let sample = probe.current().expect("ok").expect("activity");
        assert_eq!(sample.activity_kind, bio_spec::ACTIVITY_KIND_COMMIT);
        assert!(sample.event_count.unwrap_or(0) >= 1);
        // Privacy: sample must not carry path-like fields (struct has none).
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .env("GIT_AUTHOR_NAME", "Dogfood")
            .env("GIT_AUTHOR_EMAIL", "dogfood@example.com")
            .env("GIT_COMMITTER_NAME", "Dogfood")
            .env("GIT_COMMITTER_EMAIL", "dogfood@example.com")
            .status()
            .expect("spawn git");
        assert!(status.success(), "git {args:?} failed");
    }
}
