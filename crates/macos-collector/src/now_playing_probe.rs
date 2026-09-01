//! Now Playing probe: coarse media_kind + is_playing (no titles / lyrics).

use std::sync::Mutex;

use crate::error::CollectorResult;

/// Privacy-safe Now Playing sample (no title / artist / lyrics / playlist).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlayingSample {
    /// Coarse v1 media kind (`music` / `podcast` / `other` / `none` / `unknown`).
    pub media_kind: String,
    /// Whether media is actively playing.
    pub is_playing: bool,
}

impl NowPlayingSample {
    /// Identity key used to detect changes (kind + playing flag).
    #[must_use]
    pub fn identity_key(&self) -> String {
        format!("{}|{}", self.media_kind, self.is_playing)
    }
}

/// Injectable Now Playing source (tests / OS stub).
pub trait NowPlayingProbe: Send + Sync {
    /// Returns the current coarse sample, or `None` if unavailable.
    fn current(&self) -> CollectorResult<Option<NowPlayingSample>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedNowPlayingProbe {
    samples: Mutex<Vec<Option<NowPlayingSample>>>,
}

impl ScriptedNowPlayingProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture queue (consumed FIFO by [`NowPlayingProbe::current`]).
    pub fn set_samples(&self, samples: Vec<Option<NowPlayingSample>>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        *guard = samples;
    }

    /// Append one fixture sample.
    pub fn push(&self, sample: Option<NowPlayingSample>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(sample);
    }
}

impl NowPlayingProbe for ScriptedNowPlayingProbe {
    fn current(&self) -> CollectorResult<Option<NowPlayingSample>> {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Production probe: soft-fail when OS Now Playing mapping is unavailable.
///
/// v1 does not wire private MediaRemote / content-bearing AppleScript — those
/// paths risk titles or always-on capture. Returns `None` (idle, no emit) until
/// a privacy-safe OS mapping exists. Tests use [`ScriptedNowPlayingProbe`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemNowPlayingProbe;

impl NowPlayingProbe for SystemNowPlayingProbe {
    fn current(&self) -> CollectorResult<Option<NowPlayingSample>> {
        system_now_playing()
    }
}

fn system_now_playing() -> CollectorResult<Option<NowPlayingSample>> {
    // Soft-fail: no content-bearing OS probe in v1 (compile + idle-safe).
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_probe_fifo() {
        let probe = ScriptedNowPlayingProbe::new();
        probe.push(Some(NowPlayingSample {
            media_kind: "music".into(),
            is_playing: true,
        }));
        probe.push(None);
        let first = probe.current().expect("ok").expect("sample");
        assert_eq!(first.media_kind, "music");
        assert!(first.is_playing);
        assert!(probe.current().expect("ok").is_none());
        assert!(probe.current().expect("ok").is_none());
    }

    #[test]
    fn system_probe_soft_fails_idle() {
        let sample = SystemNowPlayingProbe.current().expect("ok");
        assert!(sample.is_none());
    }

    #[test]
    fn identity_key_includes_playing() {
        let a = NowPlayingSample {
            media_kind: "music".into(),
            is_playing: true,
        };
        let b = NowPlayingSample {
            media_kind: "music".into(),
            is_playing: false,
        };
        assert_ne!(a.identity_key(), b.identity_key());
    }
}
