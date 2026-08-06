//! Shared helpers for calendar-derived catalog Features (P6-E3-T2).
//!
//! Parses `calendar_event` Observation payloads (`uid` / `start` / `end` /
//! optional `busy` / `all_day`). Titles are never required. Tolerates partial
//! calendars: malformed payloads are skipped, not fatal.

use bio_spec::{Observation, TimeWindow};
use uuid::Uuid;

/// Canonical `data_type` for Calendar / meeting Observations.
pub const DATA_TYPE_CALENDAR_EVENT: &str = "calendar_event";

/// Parsed busy meeting interval from a `calendar_event` Observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusyMeeting {
    /// Source Observation id (provenance).
    pub observation_id: Uuid,
    /// Event start (unix secs).
    pub start: i64,
    /// Event end (unix secs).
    pub end: i64,
}

/// Collects busy (non-all-day) meetings from a snapshot.
///
/// - Missing / invalid `uid`/`start`/`end` → skip.
/// - `all_day == true` → skip (not timed meeting load).
/// - `busy == false` → skip; missing `busy` defaults to busy (dogfood ICS).
#[must_use]
pub fn collect_busy_meetings(observations: &[Observation]) -> Vec<BusyMeeting> {
    let mut out = Vec::new();
    for obs in observations {
        if obs.data_type != DATA_TYPE_CALENDAR_EVENT {
            continue;
        }
        if let Some(m) = parse_busy_meeting(obs) {
            out.push(m);
        }
    }
    out.sort_by_key(|m| (m.start, m.end, m.observation_id));
    out
}

/// Min start / max end over busy meetings, if any.
#[must_use]
pub fn meeting_time_span(meetings: &[BusyMeeting]) -> Option<(i64, i64)> {
    let first = meetings.first()?;
    let (min_ts, max_ts) = meetings.iter().fold(
        (first.start, first.end),
        |(lo, hi), m| (lo.min(m.start), hi.max(m.end)),
    );
    Some((min_ts, max_ts))
}

/// Meetings whose `[start, end]` intersects the inclusive Feature window.
#[must_use]
pub fn meetings_overlapping_window<'a>(
    meetings: &'a [BusyMeeting],
    window: &TimeWindow,
) -> Vec<&'a BusyMeeting> {
    let w_start = window.start.as_secs();
    let w_end = window.end.as_secs();
    meetings
        .iter()
        .filter(|m| intervals_overlap(m.start, m.end, w_start, w_end))
        .collect()
}

/// Union length (seconds) of busy intervals clipped to `[w_start, w_end]`.
#[must_use]
pub fn merged_overlap_secs(meetings: &[&BusyMeeting], w_start: i64, w_end: i64) -> i64 {
    if w_end <= w_start {
        return 0;
    }
    let mut clipped: Vec<(i64, i64)> = meetings
        .iter()
        .filter_map(|m| {
            let start = m.start.max(w_start);
            let end = m.end.min(w_end);
            if end > start {
                Some((start, end))
            } else {
                None
            }
        })
        .collect();
    if clipped.is_empty() {
        return 0;
    }
    clipped.sort_by_key(|(s, _)| *s);
    let mut total = 0i64;
    let (mut cur_s, mut cur_e) = clipped[0];
    for &(s, e) in clipped.iter().skip(1) {
        if s <= cur_e {
            cur_e = cur_e.max(e);
        } else {
            total += cur_e - cur_s;
            cur_s = s;
            cur_e = e;
        }
    }
    total += cur_e - cur_s;
    total
}

/// Free gaps (prev.end → next.start) between consecutive meetings, in order.
///
/// Zero / negative gaps (back-to-back or overlapping) are omitted.
#[must_use]
pub fn inter_meeting_gaps(meetings: &[BusyMeeting]) -> Vec<(i64, i64, Uuid, Uuid)> {
    let mut gaps = Vec::new();
    for pair in meetings.windows(2) {
        let prev = &pair[0];
        let next = &pair[1];
        if next.start > prev.end {
            gaps.push((prev.end, next.start, prev.observation_id, next.observation_id));
        }
    }
    gaps
}

/// `true` when gap `[gap_start, gap_end]` intersects inclusive Feature window.
#[must_use]
pub fn gap_intersects_window(gap_start: i64, gap_end: i64, window: &TimeWindow) -> bool {
    intervals_overlap(
        gap_start,
        gap_end,
        window.start.as_secs(),
        window.end.as_secs(),
    )
}

fn intervals_overlap(a_start: i64, a_end: i64, b_start: i64, b_end: i64) -> bool {
    a_start <= b_end && b_start <= a_end && a_end >= a_start && b_end >= b_start
}

fn parse_busy_meeting(obs: &Observation) -> Option<BusyMeeting> {
    let payload = obs.payload.as_object()?;
    let uid = payload.get("uid")?.as_str()?;
    if uid.trim().is_empty() {
        return None;
    }
    let start = payload.get("start")?.as_i64()?;
    let end = payload.get("end")?.as_i64()?;
    if end < start {
        return None;
    }
    if payload.get("all_day").and_then(|v| v.as_bool()) == Some(true) {
        return None;
    }
    if payload.get("busy").and_then(|v| v.as_bool()) == Some(false) {
        return None;
    }
    Some(BusyMeeting {
        observation_id: obs.id,
        start,
        end,
    })
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn cal_obs(id: u128, ts: i64, uid: &str, start: i64, end: i64, busy: Option<bool>) -> Observation {
        let mut payload = json!({
            "uid": uid,
            "start": start,
            "end": end,
        });
        if let Some(b) = busy {
            payload
                .as_object_mut()
                .expect("obj")
                .insert("busy".into(), json!(b));
        }
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.calendar",
            DATA_TYPE_CALENDAR_EVENT,
            payload,
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn skips_free_and_all_day() {
        let free = cal_obs(1, 100, "a", 100, 200, Some(false));
        let mut all_day = cal_obs(2, 100, "b", 100, 200, Some(true));
        all_day
            .payload
            .as_object_mut()
            .expect("obj")
            .insert("all_day".into(), json!(true));
        let busy = cal_obs(3, 100, "c", 100, 200, None);
        let meetings = collect_busy_meetings(&[free, all_day, busy]);
        assert_eq!(meetings.len(), 1);
        assert_eq!(meetings[0].observation_id, Uuid::from_u128(3));
    }

    #[test]
    fn merges_overlapping_busy_time() {
        let a = BusyMeeting {
            observation_id: Uuid::from_u128(1),
            start: 0,
            end: 100,
        };
        let b = BusyMeeting {
            observation_id: Uuid::from_u128(2),
            start: 50,
            end: 150,
        };
        let secs = merged_overlap_secs(&[&a, &b], 0, 200);
        assert_eq!(secs, 150);
    }
}
