//! Ambient light probe: coarse light_kind + optional level (no camera / screen / geo).

use std::sync::Mutex;

use crate::error::CollectorResult;

/// Privacy-safe ambient light sample (no frames / screenshots / geo / mic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbientLightSample {
    /// Coarse v1 light kind (`dark` / `dim` / `moderate` / `bright` / `unknown`).
    pub light_kind: String,
    /// Optional relative brightness band 0–100.
    pub level: Option<u8>,
}

impl AmbientLightSample {
    /// Identity key used to detect changes (kind + optional level).
    #[must_use]
    pub fn identity_key(&self) -> String {
        match self.level {
            Some(n) => format!("{}|{n}", self.light_kind),
            None => format!("{}|", self.light_kind),
        }
    }
}

/// Injectable ambient light source (tests / OS stub).
pub trait AmbientLightProbe: Send + Sync {
    /// Returns the current coarse sample, or `None` if unavailable.
    fn current(&self) -> CollectorResult<Option<AmbientLightSample>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedAmbientLightProbe {
    samples: Mutex<Vec<Option<AmbientLightSample>>>,
}

impl ScriptedAmbientLightProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture queue (consumed FIFO by [`AmbientLightProbe::current`]).
    pub fn set_samples(&self, samples: Vec<Option<AmbientLightSample>>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        *guard = samples;
    }

    /// Append one fixture sample.
    pub fn push(&self, sample: Option<AmbientLightSample>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(sample);
    }
}

impl AmbientLightProbe for ScriptedAmbientLightProbe {
    fn current(&self) -> CollectorResult<Option<AmbientLightSample>> {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Production probe: soft-fail when OS ambient-light mapping is unavailable.
///
/// v1 does not wire camera-based scene capture or content-bearing screen
/// sampling — those paths risk frames / screenshots. Returns `None` (idle, no
/// emit) until a privacy-safe OS mapping exists. Tests use
/// [`ScriptedAmbientLightProbe`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemAmbientLightProbe;

impl AmbientLightProbe for SystemAmbientLightProbe {
    fn current(&self) -> CollectorResult<Option<AmbientLightSample>> {
        system_ambient_light()
    }
}

fn system_ambient_light() -> CollectorResult<Option<AmbientLightSample>> {
    // Soft-fail: no camera / screen / geo OS probe in v1 (compile + idle-safe).
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_probe_fifo() {
        let probe = ScriptedAmbientLightProbe::new();
        probe.push(Some(AmbientLightSample {
            light_kind: "dim".into(),
            level: Some(25),
        }));
        probe.push(None);
        let first = probe.current().expect("ok").expect("sample");
        assert_eq!(first.light_kind, "dim");
        assert_eq!(first.level, Some(25));
        assert!(probe.current().expect("ok").is_none());
        assert!(probe.current().expect("ok").is_none());
    }

    #[test]
    fn system_probe_soft_fails_idle() {
        let sample = SystemAmbientLightProbe.current().expect("ok");
        assert!(sample.is_none());
    }

    #[test]
    fn identity_key_includes_level() {
        let a = AmbientLightSample {
            light_kind: "dim".into(),
            level: Some(25),
        };
        let b = AmbientLightSample {
            light_kind: "dim".into(),
            level: Some(40),
        };
        assert_ne!(a.identity_key(), b.identity_key());
    }
}
