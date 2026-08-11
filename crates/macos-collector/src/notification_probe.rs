//! Notification event probe: coarse count + optional closed-set labels (no content).

use std::sync::Mutex;

use crate::error::CollectorResult;

/// Privacy-safe notification sample (no body / title / message / screenshots).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationEventSample {
    /// Notification count represented by this sample (≥ 1).
    pub count: u64,
    /// Optional closed-set category.
    pub category: Option<String>,
    /// Optional closed-set interruption level.
    pub interruption_level: Option<String>,
    /// Optional closed-set app kind.
    pub app_kind: Option<String>,
}

impl NotificationEventSample {
    /// Identity key used to detect changes (count + optional labels).
    #[must_use]
    pub fn identity_key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.count,
            self.category.as_deref().unwrap_or(""),
            self.interruption_level.as_deref().unwrap_or(""),
            self.app_kind.as_deref().unwrap_or("")
        )
    }
}

/// Injectable notification source (tests / OS stub).
pub trait NotificationEventProbe: Send + Sync {
    /// Returns the current coarse sample, or `None` if unavailable.
    fn current(&self) -> CollectorResult<Option<NotificationEventSample>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedNotificationEventProbe {
    samples: Mutex<Vec<Option<NotificationEventSample>>>,
}

impl ScriptedNotificationEventProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture queue (consumed FIFO by [`NotificationEventProbe::current`]).
    pub fn set_samples(&self, samples: Vec<Option<NotificationEventSample>>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        *guard = samples;
    }

    /// Append one fixture sample.
    pub fn push(&self, sample: Option<NotificationEventSample>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(sample);
    }
}

impl NotificationEventProbe for ScriptedNotificationEventProbe {
    fn current(&self) -> CollectorResult<Option<NotificationEventSample>> {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Production probe: soft-fail when OS notification mapping is unavailable.
///
/// v1 does not wire Notification Center content APIs — those paths risk
/// title/body capture. Returns `None` (idle, no emit) until a privacy-safe
/// OS mapping exists. Tests use [`ScriptedNotificationEventProbe`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemNotificationEventProbe;

impl NotificationEventProbe for SystemNotificationEventProbe {
    fn current(&self) -> CollectorResult<Option<NotificationEventSample>> {
        system_notification_event()
    }
}

fn system_notification_event() -> CollectorResult<Option<NotificationEventSample>> {
    // Soft-fail: no content-bearing notification OS probe in v1 (compile + idle-safe).
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_probe_fifo() {
        let probe = ScriptedNotificationEventProbe::new();
        probe.push(Some(NotificationEventSample {
            count: 1,
            category: Some("communication".into()),
            interruption_level: Some("active".into()),
            app_kind: Some("messaging".into()),
        }));
        probe.push(None);
        let first = probe.current().expect("ok").expect("sample");
        assert_eq!(first.count, 1);
        assert_eq!(first.category.as_deref(), Some("communication"));
        assert!(probe.current().expect("ok").is_none());
        assert!(probe.current().expect("ok").is_none());
    }

    #[test]
    fn system_probe_soft_fails_idle() {
        let sample = SystemNotificationEventProbe.current().expect("ok");
        assert!(sample.is_none());
    }

    #[test]
    fn identity_key_includes_labels() {
        let a = NotificationEventSample {
            count: 1,
            category: Some("system".into()),
            interruption_level: None,
            app_kind: None,
        };
        let b = NotificationEventSample {
            count: 2,
            category: Some("system".into()),
            interruption_level: None,
            app_kind: None,
        };
        assert_ne!(a.identity_key(), b.identity_key());
    }
}
