//! Sliding-window helpers shared by catalog Feature nodes (15m / configurable step).

use bio_spec::{Observation, TimeWindow, UnixTimestamp};

use crate::ComputeContext;

/// Catalog Focus / context Features: 15-minute window (`docs/06-feature-catalog.md`).
pub const WINDOW_SECS: i64 = 15 * 60;

/// Default catalog step between successive window ends (1 minute).
pub const STEP_SECS: i64 = 60;

/// Hard cap on emitted window ends (avoids pathological year-long 1m steps).
const MAX_WINDOW_ENDS: usize = 24 * 60;

/// Builds inclusive `[end - WINDOW_SECS, end]` (saturating on underflow).
#[must_use]
pub fn window_ending_at(end_secs: i64) -> TimeWindow {
    TimeWindow {
        start: UnixTimestamp::from_secs(end_secs.saturating_sub(WINDOW_SECS)),
        end: UnixTimestamp::from_secs(end_secs),
    }
}

/// Step-aligned sliding window ends covering `[min_ts, max_ts]`.
///
/// Ends are floored to `step_secs` (minimum 1). Only ends with `end >= min_ts`
/// are yielded, chronological order. When `max_ts` is past the last aligned
/// step, `max_ts` itself is appended so the tip Observation is covered.
/// Empty / inverted range → empty vec. Caps at [`MAX_WINDOW_ENDS`] points.
#[must_use]
pub fn sliding_window_ends(min_ts: i64, max_ts: i64, step_secs: i64) -> Vec<i64> {
    if max_ts < min_ts {
        return Vec::new();
    }
    let step = step_secs.max(1);
    let aligned_max = (max_ts / step) * step;
    let mut end = aligned_max;
    let mut ends = Vec::new();
    while end >= min_ts {
        ends.push(end);
        end -= step;
        if ends.len() >= MAX_WINDOW_ENDS {
            break;
        }
    }
    ends.reverse();
    if aligned_max < max_ts && ends.len() < MAX_WINDOW_ENDS {
        if ends.last().copied() != Some(max_ts) {
            ends.push(max_ts);
        }
    }
    // Single tip with no aligned end in range (e.g. min==max off-step).
    if ends.is_empty() && max_ts >= min_ts {
        ends.push(max_ts);
    }
    ends
}

/// Window ends for a compute run using the context step (ADR-018 series).
#[must_use]
pub fn sliding_window_ends_for(ctx: &ComputeContext<'_>, min_ts: i64, max_ts: i64) -> Vec<i64> {
    sliding_window_ends(min_ts, max_ts, ctx.step_secs())
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
