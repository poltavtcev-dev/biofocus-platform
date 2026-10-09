//! Phone → Mac sync progress. Counts and a phase only — no health values.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// In-memory slot shared by the HTTP handler and the desktop IPC read.
#[derive(Clone, Default)]
pub struct CompanionStatusSlot {
    inner: Arc<Mutex<Option<CompanionStatus>>>,
}

/// Last companion progress report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CompanionStatus {
    pub phase: String,
    pub types_ok: u32,
    pub types_empty: u32,
    pub types_total: u32,
    pub pending: u32,
    pub received_at: i64,
}

/// Body of `POST /v1/companion/status`. Extra fields are rejected.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionStatusBody {
    pub phase: String,
    pub types_ok: u32,
    pub types_empty: u32,
    pub types_total: u32,
    pub pending: u32,
}

const PHASES: &[&str] = &["idle", "recent", "history", "done"];

impl CompanionStatusSlot {
    pub fn store(&self, body: CompanionStatusBody) -> Result<CompanionStatus, &'static str> {
        if !PHASES.contains(&body.phase.as_str()) {
            return Err("invalid_companion_status");
        }
        if body.types_total > 64
            || body.types_ok > body.types_total
            || body.types_empty > body.types_total
        {
            return Err("invalid_companion_status");
        }
        if body.pending > 1_000_000 {
            return Err("invalid_companion_status");
        }
        let status = CompanionStatus {
            phase: body.phase,
            types_ok: body.types_ok,
            types_empty: body.types_empty,
            types_total: body.types_total,
            pending: body.pending,
            received_at: unix_now(),
        };
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(status.clone());
        }
        Ok(status)
    }

    #[must_use]
    pub fn snapshot(&self) -> Option<CompanionStatus> {
        self.inner.lock().ok().and_then(|guard| guard.clone())
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

static PUBLISHED: Mutex<Option<CompanionStatusSlot>> = Mutex::new(None);

/// Remembers the slot the running ingest server updates.
pub fn publish_companion_status(slot: CompanionStatusSlot) {
    if let Ok(mut guard) = PUBLISHED.lock() {
        *guard = Some(slot);
    }
}

/// Latest progress, if the desktop host has started ingest.
#[must_use]
pub fn published_companion_status() -> Option<CompanionStatus> {
    PUBLISHED
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(CompanionStatusSlot::snapshot))
}
