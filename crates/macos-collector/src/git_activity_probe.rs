//! Git activity probe: coarse activity_kind + optional event_count (no paths).

use std::sync::Mutex;

use crate::error::CollectorResult;

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

/// Production probe: soft-fail when mapping / watched roots are unavailable.
///
/// v1 has **no** persisted path-allowlist table (ADR-013 deferred). Without an
/// approved allowlist, scanning the filesystem for repos would risk path leakage
/// and workplace-style surveillance. Returns `None` (idle, no emit) until a
/// privacy-safe probe exists. Tests use [`ScriptedGitActivityProbe`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemGitActivityProbe;

impl GitActivityProbe for SystemGitActivityProbe {
    fn current(&self) -> CollectorResult<Option<GitActivitySample>> {
        system_git_activity()
    }
}

fn system_git_activity() -> CollectorResult<Option<GitActivitySample>> {
    // Soft-fail: no path-allowlist / content-bearing probe in v1 (compile + idle-safe).
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn system_probe_soft_fails_idle() {
        let sample = SystemGitActivityProbe.current().expect("ok");
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
}
