//! Optional Pattern Discovery inputs for baseline / multi-window rules (ADR-008).

use bio_spec::{Feature, ObservationId, UnixTimestamp};

/// A user-logged Life Event (already filtered for retractions by the host).
#[derive(Debug, Clone, PartialEq)]
pub struct LifeEventMark {
    /// Observation id of the (non-retracted) `life_event` row.
    pub id: ObservationId,
    /// v1 kind (`coffee`, `walk`, `lunch`, `workout`).
    pub kind: String,
    /// When it happened (Observation timestamp).
    pub timestamp: UnixTimestamp,
}

/// Recompute-on-read series supplied by Core for pattern Insight rules.
///
/// Host loads Observations, recomputes a bounded Feature series (via
/// `feature-engine`), then passes it here. Empty default keeps snapshot-only
/// rules unchanged. Not persisted — process-local only.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PatternInputs {
    /// Prior comparable windows (e.g. FocusScore afternoon means, N ≤ 7).
    pub baseline_series: Vec<Feature>,
    /// Recent Life Events (retracted ones already removed by Core).
    pub life_events: Vec<LifeEventMark>,
    /// Recent stepped series (e.g. last 8 h of FocusScore / CognitiveLoad)
    /// used for before/after comparisons around [`Self::life_events`].
    pub recent_series: Vec<Feature>,
}

impl PatternInputs {
    /// Empty pattern context (no baseline series).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Construct with a baseline Feature series.
    #[must_use]
    pub fn with_baseline_series(baseline_series: Vec<Feature>) -> Self {
        Self {
            baseline_series,
            ..Self::default()
        }
    }

    /// Attach Life Events + the recent series they are compared against.
    #[must_use]
    pub fn with_life_events(mut self, life_events: Vec<LifeEventMark>, recent_series: Vec<Feature>) -> Self {
        self.life_events = life_events;
        self.recent_series = recent_series;
        self
    }
}
