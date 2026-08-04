//! Sliding-window helpers shared by catalog Feature nodes (15m / 1m step).

use bio_spec::{Observation, TimeWindow, UnixTimestamp};

/// Catalog Focus / context Features: 15-minute window (`docs/06-feature-catalog.md`).
pub const WINDOW_SECS: i64 = 15 * 60;

/// Catalog step between successive window ends (1 minute).
pub const STEP_SECS: i64 = 60;

/// Builds inclusive `[end - WINDOW_SECS, end]` (saturating on underflow).
#[must_use]
pub fn window_ending_at(end_secs: i64) -> TimeWindow {
    TimeWindow {
        start: UnixTimestamp::from_secs(end_secs.saturating_sub(WINDOW_SECS)),
        end: UnixTimestamp::from_secs(end_secs),
    }
}

/// Minute-aligned sliding window ends covering `[min_ts, max_ts]`.
///
/// Ends are floored to [`STEP_SECS`]. Only ends with `end >= min_ts` are yielded,
/// chronological order. Empty input range → empty vec.
#[must_use]
pub fn sliding_window_ends(min_ts: i64, max_ts: i64) -> Vec<i64> {
    if max_ts < min_ts {
        return Vec::new();
    }
    let mut end = (max_ts / STEP_SECS) * STEP_SECS;
    let mut ends = Vec::new();
    while end >= min_ts {
        ends.push(end);
        end -= STEP_SECS;
        // Bound pathological ranges (years of 1m steps).
        if ends.len() > 24 * 60 {
            break;
        }
    }
    ends.reverse();
    ends
}

/// `true` when Observation timestamp is inside inclusive `[start, end]`.
#[must_use]
pub fn in_window(obs: &Observation, window: &TimeWindow) -> bool {
    let t = obs.timestamp.as_secs();
    t >= window.start.as_secs() && t <= window.end.as_secs()
}

/// Min/max Observation timestamps in the snapshot, if non-empty.
#[must_use]
pub fn snapshot_time_span(observations: &[Observation]) -> Option<(i64, i64)> {
    let mut iter = observations.iter().map(|o| o.timestamp.as_secs());
    let first = iter.next()?;
    let (min_ts, max_ts) = iter.fold((first, first), |(lo, hi), t| (lo.min(t), hi.max(t)));
    Some((min_ts, max_ts))
}
