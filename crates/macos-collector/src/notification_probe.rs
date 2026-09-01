//! Notification event probe: coarse count + optional closed-set labels (no content).
//!
//! Live [`SystemNotificationEventProbe`] reads the macOS **usernoted** Notification
//! Center SQLite DB (ADR-020) with a hard non-content field allowlist — never
//! `record.data` / title / body / userInfo. Soft-fail when unavailable.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tracing::debug;

use crate::error::CollectorResult;
use crate::notification_nc_db::{
    max_delivered_date, open_nc_db_readonly, query_deliveries_after, sample_labels_from_rows,
};

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

#[derive(Debug, Default)]
struct LiveProbeState {
    /// Last seen `delivered_date` watermark. `None` until first successful open
    /// (initialized to DB max without emitting — no historical dump).
    watermark: Option<f64>,
}

/// Production probe: hybrid usernoted DB mapping (ADR-020); soft-fail when unavailable.
#[derive(Debug, Default)]
pub struct SystemNotificationEventProbe {
    db_override: Option<PathBuf>,
    state: Mutex<LiveProbeState>,
}

impl SystemNotificationEventProbe {
    /// Production probe using default NC DB candidate paths.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Test / dogfood helper: force a specific SQLite path (fixture or custom).
    #[must_use]
    pub fn with_db_path(path: impl Into<PathBuf>) -> Self {
        Self {
            db_override: Some(path.into()),
            state: Mutex::new(LiveProbeState::default()),
        }
    }
}

impl NotificationEventProbe for SystemNotificationEventProbe {
    fn current(&self) -> CollectorResult<Option<NotificationEventSample>> {
        system_notification_event(self.db_override.as_deref(), &self.state)
    }
}

fn system_notification_event(
    db_override: Option<&Path>,
    state: &Mutex<LiveProbeState>,
) -> CollectorResult<Option<NotificationEventSample>> {
    let Some((_path, conn)) = open_nc_db_readonly(db_override) else {
        debug!("notification_event live probe idle (NC DB unavailable)");
        return Ok(None);
    };

    let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());

    // First successful open: set watermark to current max without emitting history.
    if guard.watermark.is_none() {
        let wm = max_delivered_date(&conn).unwrap_or(0.0);
        guard.watermark = Some(wm);
        debug!(watermark = wm, "notification_event live probe watermark initialized");
        return Ok(None);
    }

    let watermark = guard.watermark.unwrap_or(0.0);
    let rows = match query_deliveries_after(&conn, watermark) {
        Ok(rows) => rows,
        Err(err) => {
            debug!(error = %err, "notification_event NC query failed; soft-fail");
            return Ok(None);
        }
    };

    if rows.is_empty() {
        return Ok(None);
    }

    if let Some(last) = rows.last() {
        guard.watermark = Some(last.delivered_date);
    }

    let count = rows.len() as u64;
    if count < 1 {
        return Ok(None);
    }

    let (category, app_kind) = sample_labels_from_rows(&rows);
    // interruption_level omitted in v1 — style/bands live in content-bearing blobs.
    Ok(Some(NotificationEventSample {
        count,
        category,
        interruption_level: None,
        app_kind,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification_nc_db::write_fixture_nc_db;
    use tempfile::tempdir;

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
    fn system_probe_soft_fails_when_db_missing() {
        let dir = tempdir().expect("temp");
        let missing = dir.path().join("no-such-db");
        let probe = SystemNotificationEventProbe::with_db_path(missing);
        let sample = probe.current().expect("ok");
        assert!(sample.is_none());
    }

    #[test]
    fn live_fixture_emits_delta_without_content_keys() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("db");
        write_fixture_nc_db(
            &path,
            &[
                ("com.apple.mail", 10.0),
                ("com.apple.MobileSMS", 20.0),
            ],
        )
        .expect("fixture");

        let probe = SystemNotificationEventProbe::with_db_path(&path);
        // First poll: watermark init, no emit.
        assert!(probe.current().expect("ok").is_none());

        // Append a new delivery after watermark.
        {
            use rusqlite::Connection;
            let conn = Connection::open(&path).expect("open");
            conn.execute(
                "INSERT INTO app (app_id, identifier) VALUES (3, 'com.apple.mail')",
                [],
            )
            .ok();
            // reuse app_id 1 (mail) if present
            let app_id: i64 = conn
                .query_row(
                    "SELECT app_id FROM app WHERE identifier = 'com.apple.mail' LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .expect("app");
            conn.execute(
                "INSERT INTO record (rec_id, app_id, delivered_date) VALUES (99, ?1, 30.0)",
                [app_id],
            )
            .expect("insert");
        }

        let sample = probe.current().expect("ok").expect("delta sample");
        assert_eq!(sample.count, 1);
        assert_eq!(sample.category.as_deref(), Some("communication"));
        assert_eq!(sample.app_kind.as_deref(), Some("mail"));
        assert!(sample.interruption_level.is_none());
        // No content fields exist on the sample struct / payload builder path.
        let payload = crate::payload::notification_event_payload(&sample);
        assert!(payload.get("title").is_none());
        assert!(payload.get("body").is_none());
        assert!(payload.get("bundle_id").is_none());
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
