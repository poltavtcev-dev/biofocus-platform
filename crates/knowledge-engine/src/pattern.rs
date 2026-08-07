//! Optional Pattern Discovery inputs for baseline / multi-window rules (ADR-008).

use bio_spec::Feature;

/// Recompute-on-read series supplied by Core for pattern Insight rules.
///
/// Host loads Observations, recomputes a bounded Feature series (via
/// `feature-engine`), then passes it here. Empty default keeps snapshot-only
/// rules unchanged. Not persisted — process-local only.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PatternInputs {
    /// Prior comparable windows (e.g. FocusScore afternoon means, N ≤ 7).
    pub baseline_series: Vec<Feature>,
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
        Self { baseline_series }
    }
}
