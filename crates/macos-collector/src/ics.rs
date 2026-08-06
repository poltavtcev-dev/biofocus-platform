//! Minimal local ICS (iCalendar) parser for dogfood Calendar Observations.
//!
//! Privacy: SUMMARY / DESCRIPTION / LOCATION / ATTENDEE are **ignored** and never
//! returned. Only scheduling metadata needed for meeting-density Features.

use std::path::Path;

use crate::calendar_probe::CalendarEvent;
use crate::error::{CollectorError, CollectorResult};

/// Parses VEVENT blocks from ICS text into privacy-safe [`CalendarEvent`]s.
pub fn parse_ics_events(ics: &str) -> CollectorResult<Vec<CalendarEvent>> {
    let unfolded = unfold_lines(ics);
    let mut events = Vec::new();
    let mut in_event = false;
    let mut uid: Option<String> = None;
    let mut start: Option<(i64, bool)> = None;
    let mut end: Option<(i64, bool)> = None;
    let mut duration_secs: Option<i64> = None;
    let mut transparent = false;

    for line in unfolded.lines() {
        let line = line.trim_end();
        if line.eq_ignore_ascii_case("BEGIN:VEVENT") {
            in_event = true;
            uid = None;
            start = None;
            end = None;
            duration_secs = None;
            transparent = false;
            continue;
        }
        if line.eq_ignore_ascii_case("END:VEVENT") {
            if in_event {
                if let Some(event) =
                    finish_event(uid.take(), start.take(), end.take(), duration_secs.take(), transparent)
                {
                    events.push(event);
                }
            }
            in_event = false;
            continue;
        }
        if !in_event {
            continue;
        }

        let (name, params, value) = split_property(line);
        let name_upper = name.to_ascii_uppercase();
        match name_upper.as_str() {
            "UID" => {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    uid = Some(trimmed.to_string());
                }
            }
            "DTSTART" => {
                if let Some(parsed) = parse_ical_datetime(value, &params) {
                    start = Some(parsed);
                }
            }
            "DTEND" => {
                if let Some(parsed) = parse_ical_datetime(value, &params) {
                    end = Some(parsed);
                }
            }
            "DURATION" => {
                duration_secs = parse_ical_duration(value);
            }
            "TRANSP" => {
                transparent = value.eq_ignore_ascii_case("TRANSPARENT");
            }
            // Intentionally ignored for privacy: SUMMARY, DESCRIPTION, LOCATION, …
            _ => {}
        }
    }

    Ok(events)
}

/// Reads and parses an ICS file from disk.
pub fn parse_ics_file(path: &Path) -> CollectorResult<Vec<CalendarEvent>> {
    let text = std::fs::read_to_string(path).map_err(|err| {
        CollectorError::Probe(format!("failed to read calendar ICS {}: {err}", path.display()))
    })?;
    parse_ics_events(&text)
}

fn finish_event(
    uid: Option<String>,
    start: Option<(i64, bool)>,
    end: Option<(i64, bool)>,
    duration_secs: Option<i64>,
    transparent: bool,
) -> Option<CalendarEvent> {
    let uid = uid?;
    let (start_secs, start_all_day) = start?;
    let (end_secs, end_all_day) = match end {
        Some(e) => e,
        None => {
            let dur = duration_secs.unwrap_or(if start_all_day { 86_400 } else { 3_600 });
            (start_secs.saturating_add(dur), start_all_day)
        }
    };
    let all_day = start_all_day || end_all_day;
    let end_secs = end_secs.max(start_secs);
    Some(CalendarEvent {
        uid,
        start: start_secs,
        end: end_secs,
        all_day,
        busy: !transparent,
    })
}

fn unfold_lines(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            out.push_str(line.trim_start());
        } else {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
        }
    }
    out
}

fn split_property(line: &str) -> (&str, Vec<&str>, &str) {
    let (left, value) = match line.split_once(':') {
        Some((l, v)) => (l, v),
        None => (line, ""),
    };
    let mut parts = left.split(';');
    let name = parts.next().unwrap_or(left);
    let params: Vec<&str> = parts.collect();
    (name, params, value)
}

fn param_value_is_date(params: &[&str]) -> bool {
    params.iter().any(|p| {
        let p = p.trim();
        p.eq_ignore_ascii_case("VALUE=DATE")
    })
}

fn parse_ical_datetime(value: &str, params: &[&str]) -> Option<(i64, bool)> {
    let raw = value.trim();
    if raw.is_empty() {
        return None;
    }
    // Drop trailing Z for parsing; treat as UTC either way for dogfood.
    let zulu = raw.ends_with('Z') || raw.ends_with('z');
    let digits = if zulu {
        &raw[..raw.len() - 1]
    } else {
        raw
    };
    let all_day = param_value_is_date(params) || digits.len() == 8;
    if all_day {
        if digits.len() < 8 {
            return None;
        }
        let y: i32 = digits.get(0..4)?.parse().ok()?;
        let m: u32 = digits.get(4..6)?.parse().ok()?;
        let d: u32 = digits.get(6..8)?.parse().ok()?;
        let secs = civil_to_unix(y, m, d, 0, 0, 0)?;
        return Some((secs, true));
    }
    // YYYYMMDDTHHMMSS
    if digits.len() < 15 || digits.as_bytes().get(8) != Some(&b'T') {
        return None;
    }
    let y: i32 = digits.get(0..4)?.parse().ok()?;
    let m: u32 = digits.get(4..6)?.parse().ok()?;
    let d: u32 = digits.get(6..8)?.parse().ok()?;
    let h: u32 = digits.get(9..11)?.parse().ok()?;
    let mi: u32 = digits.get(11..13)?.parse().ok()?;
    let s: u32 = digits.get(13..15)?.parse().ok()?;
    let secs = civil_to_unix(y, m, d, h, mi, s)?;
    let _ = zulu; // dogfood: floating local treated as UTC
    Some((secs, false))
}

fn parse_ical_duration(value: &str) -> Option<i64> {
    // Subset: PnDTnHnMnS / PTnHnMnS / PnW
    let v = value.trim().to_ascii_uppercase();
    if !v.starts_with('P') {
        return None;
    }
    let mut rest = &v[1..];
    let mut total = 0i64;
    let mut in_time = false;

    while !rest.is_empty() {
        if rest.starts_with('T') {
            in_time = true;
            rest = &rest[1..];
            continue;
        }
        let digits_end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        if digits_end == 0 {
            return None;
        }
        let n: i64 = rest.get(..digits_end)?.parse().ok()?;
        let unit = rest.as_bytes().get(digits_end).copied()? as char;
        rest = &rest[digits_end + 1..];
        let secs = match (unit, in_time) {
            ('W', false) => n.saturating_mul(7 * 86_400),
            ('D', false) => n.saturating_mul(86_400),
            ('H', true) => n.saturating_mul(3_600),
            ('M', true) => n.saturating_mul(60),
            ('S', true) => n,
            _ => return None,
        };
        total = total.saturating_add(secs);
    }
    Some(total)
}

/// Days from civil date to Unix epoch using Howard Hinnant's algorithm.
fn civil_to_unix(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> Option<i64> {
    if !(1..=12).contains(&m) || d == 0 || d > 31 || hh > 23 || mm > 59 || ss > 59 {
        return None;
    }
    let y = i64::from(y);
    let m = i64::from(m);
    let d = i64::from(d);
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let tod = i64::from(hh) * 3_600 + i64::from(mm) * 60 + i64::from(ss);
    Some(days.saturating_mul(86_400).saturating_add(tod))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_busy_and_free_events_without_titles() {
        let ics = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:busy-1@biofocus
DTSTART:20240726T100000Z
DTEND:20240726T110000Z
SUMMARY:Secret Standup
DESCRIPTION:do not leak
END:VEVENT
BEGIN:VEVENT
UID:free-1@biofocus
DTSTART:20240726T130000Z
DTEND:20240726T133000Z
TRANSP:TRANSPARENT
SUMMARY:Focus block
END:VEVENT
END:VCALENDAR
"#;
        let events = parse_ics_events(ics).expect("parse");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "busy-1@biofocus");
        assert!(events[0].busy);
        assert!(!events[0].all_day);
        assert_eq!(events[0].end - events[0].start, 3_600);
        assert!(!events[1].busy);
        assert_eq!(events[1].end - events[1].start, 1_800);
    }

    #[test]
    fn parses_all_day_date_value() {
        let ics = r#"BEGIN:VEVENT
UID:allday@biofocus
DTSTART;VALUE=DATE:20240726
DTEND;VALUE=DATE:20240727
END:VEVENT
"#;
        let events = parse_ics_events(ics).expect("parse");
        assert_eq!(events.len(), 1);
        assert!(events[0].all_day);
        assert_eq!(events[0].end - events[0].start, 86_400);
    }
}
