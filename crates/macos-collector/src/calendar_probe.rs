//! Calendar probe: local ICS / synthetic fixtures → privacy-safe events.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::CollectorResult;
use crate::ics::parse_ics_file;

/// Privacy-safe calendar event (no title / body / attendees).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarEvent {
    /// Stable local id (ICS UID).
    pub uid: String,
    /// Event start, Unix seconds UTC.
    pub start: i64,
    /// Event end, Unix seconds UTC (`end >= start`).
    pub end: i64,
    /// All-day flag from ICS `VALUE=DATE`.
    pub all_day: bool,
    /// Busy when not `TRANSP:TRANSPARENT`.
    pub busy: bool,
}

impl CalendarEvent {
    /// Dedup / emit key: uid + start + end.
    #[must_use]
    pub fn emit_key(&self) -> String {
        format!("{}:{}:{}", self.uid, self.start, self.end)
    }
}

/// Injectable calendar source (tests / ICS file / future EventKit).
pub trait CalendarProbe: Send + Sync {
    /// Returns events overlapping `[horizon_start, horizon_end]` (unix secs).
    fn events_in_range(
        &self,
        horizon_start: i64,
        horizon_end: i64,
    ) -> CollectorResult<Vec<CalendarEvent>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedCalendarProbe {
    events: Mutex<Vec<CalendarEvent>>,
}

impl ScriptedCalendarProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture set.
    pub fn set_events(&self, events: Vec<CalendarEvent>) {
        let mut guard = self.events.lock().unwrap_or_else(|e| e.into_inner());
        *guard = events;
    }

    /// Append one fixture event.
    pub fn push(&self, event: CalendarEvent) {
        let mut guard = self.events.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(event);
    }
}

impl CalendarProbe for ScriptedCalendarProbe {
    fn events_in_range(
        &self,
        horizon_start: i64,
        horizon_end: i64,
    ) -> CollectorResult<Vec<CalendarEvent>> {
        let guard = self.events.lock().unwrap_or_else(|e| e.into_inner());
        Ok(guard
            .iter()
            .filter(|e| e.end > horizon_start && e.start < horizon_end)
            .cloned()
            .collect())
    }
}

/// Local ICS file probe (no cloud OAuth).
pub struct IcsFileCalendarProbe {
    path: PathBuf,
}

impl IcsFileCalendarProbe {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl CalendarProbe for IcsFileCalendarProbe {
    fn events_in_range(
        &self,
        horizon_start: i64,
        horizon_end: i64,
    ) -> CollectorResult<Vec<CalendarEvent>> {
        let events = parse_ics_file(&self.path)?;
        Ok(events
            .into_iter()
            .filter(|e| e.end > horizon_start && e.start < horizon_end)
            .collect())
    }
}
