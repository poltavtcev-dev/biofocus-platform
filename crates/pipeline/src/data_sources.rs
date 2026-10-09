//! Read-side summary for the «Источники данных» screen (ADR-030).
//!
//! Storage still keeps every row. This module only counts what arrived.

use bio_spec::{
    DATA_TYPE_HRV, DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_RESPIRATORY_RATE,
    DATA_TYPE_SLEEP_INTERVAL, DATA_TYPE_SLEEPING_WRIST_TEMPERATURE, SLEEP_STAGE_ASLEEP_CORE,
    SLEEP_STAGE_ASLEEP_DEEP, SLEEP_STAGE_ASLEEP_REM,
};

use crate::source_select::SourcePriority;

const DAY_SECS: i64 = 86_400;
const WINDOW_DAYS: i64 = 30;

/// Shown when no source wrote HRV in the window.
pub const HINT_HRV_ABSENT: &str = "HRV: не видно ни от одного источника — Xiaomi через Zepp Life его не передаёт; нужен Apple Watch.";

const HINT_SPO2_ABSENT: &str = "Кислород в крови: не видно ни от одного источника. Mi Fitness и Zepp Life обычно не пишут его в Health.";
const HINT_RR_ABSENT: &str = "Дыхание ночью: не видно ни от одного источника. Mi Fitness и Zepp Life обычно его не передают.";
const HINT_TEMP_ABSENT: &str = "Температура запястья во сне: не видно ни от одного источника.";
const HINT_SLEEP_ABSENT: &str = "Сон: не видно ни от одного источника.";
const HINT_SLEEP_NO_STAGES: &str = "Сон: есть только общий сон, без фаз core, deep и REM.";

/// One wearable sample already reduced to the fields the screen needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSample {
    pub data_type: String,
    /// When the measurement happened (Unix seconds).
    pub timestamp: i64,
    /// When the row was stored (Unix seconds).
    pub received_at: i64,
    pub kind: String,
    pub app: String,
    pub device_model: String,
    pub sleep_stage: Option<String>,
}

/// Counts of one `data_type` inside the rolling windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCount {
    pub data_type: String,
    pub last_24h: u32,
    pub last_7d: u32,
    pub last_30d: u32,
}

/// One `src.kind` + app + device model card.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceSummary {
    pub kind: String,
    pub app: String,
    pub device_model: String,
    pub last_sample_unix: i64,
    pub last_received_unix: i64,
    pub counts: Vec<TypeCount>,
    /// Share of Europe/Belgrade days in the last 30 days that have any sample.
    pub coverage: f64,
    /// Index in the priority list. `0` is preferred.
    pub priority_rank: usize,
    /// This kind is the winner for at least one type it actually wrote.
    pub active: bool,
}

/// Everything the desktop tab renders, except companion progress.
#[derive(Debug, Clone, PartialEq)]
pub struct SourcesReport {
    pub sources: Vec<SourceSummary>,
    pub priority: Vec<String>,
    pub hints: Vec<String>,
}

/// Groups samples and compares them with `priority`.
///
/// Rows outside the last 30 days (by sample time) are ignored. Coverage uses
/// Europe/Belgrade calendar days. Rolling 24 h / 7 d / 30 d counts use sample time.
#[must_use]
pub fn summarize_sources(
    samples: &[SourceSample],
    now: i64,
    priority: &SourcePriority,
) -> SourcesReport {
    let start = now.saturating_sub(WINDOW_DAYS * DAY_SECS);
    let window: Vec<&SourceSample> = samples
        .iter()
        .filter(|sample| sample.timestamp >= start && sample.timestamp <= now)
        .filter(|sample| bio_spec::is_src_kind(&sample.kind))
        .collect();

    let mut groups: Vec<Group> = Vec::new();
    for sample in &window {
        let key = (&sample.kind, &sample.app, &sample.device_model);
        if let Some(group) = groups
            .iter_mut()
            .find(|group| (&group.kind, &group.app, &group.device_model) == key)
        {
            group.samples.push(*sample);
        } else {
            groups.push(Group {
                kind: sample.kind.clone(),
                app: sample.app.clone(),
                device_model: sample.device_model.clone(),
                samples: vec![*sample],
            });
        }
    }

    let mut best_rank_for_type: Vec<(String, usize)> = Vec::new();
    for sample in &window {
        let rank = rank_of(priority, &sample.kind);
        if let Some(existing) = best_rank_for_type
            .iter_mut()
            .find(|(data_type, _)| data_type == &sample.data_type)
        {
            if rank < existing.1 {
                existing.1 = rank;
            }
        } else {
            best_rank_for_type.push((sample.data_type.clone(), rank));
        }
    }

    let mut sources = Vec::with_capacity(groups.len());
    for group in groups {
        let mut last_sample = i64::MIN;
        let mut last_received = i64::MIN;
        let mut days = Vec::new();
        let mut type_names = Vec::new();
        for sample in &group.samples {
            last_sample = last_sample.max(sample.timestamp);
            last_received = last_received.max(sample.received_at);
            let day = belgrade_day(sample.timestamp);
            if !days.contains(&day) {
                days.push(day);
            }
            if !type_names.iter().any(|name| name == &sample.data_type) {
                type_names.push(sample.data_type.clone());
            }
        }
        type_names.sort();
        let counts = type_names
            .iter()
            .map(|data_type| count_type(&group.samples, data_type, now))
            .collect();
        let rank = rank_of(priority, &group.kind);
        let active = type_names.iter().any(|data_type| {
            best_rank_for_type
                .iter()
                .any(|(name, best)| name == data_type && *best == rank)
        });
        let coverage = (days.len() as f64 / WINDOW_DAYS as f64).min(1.0);
        sources.push(SourceSummary {
            kind: group.kind,
            app: group.app,
            device_model: group.device_model,
            last_sample_unix: last_sample,
            last_received_unix: last_received,
            counts,
            coverage,
            priority_rank: rank,
            active,
        });
    }
    sources.sort_by(|left, right| {
        left.priority_rank
            .cmp(&right.priority_rank)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.app.cmp(&right.app))
            .then_with(|| left.device_model.cmp(&right.device_model))
    });

    SourcesReport {
        hints: hints(&window),
        priority: priority.order().to_vec(),
        sources,
    }
}

struct Group<'a> {
    kind: String,
    app: String,
    device_model: String,
    samples: Vec<&'a SourceSample>,
}

fn rank_of(priority: &SourcePriority, kind: &str) -> usize {
    priority
        .order()
        .iter()
        .position(|name| name == kind)
        .unwrap_or(priority.order().len())
}

fn count_type(samples: &[&SourceSample], data_type: &str, now: i64) -> TypeCount {
    let mut last_24h = 0u32;
    let mut last_7d = 0u32;
    let mut last_30d = 0u32;
    let h24 = now.saturating_sub(DAY_SECS);
    let d7 = now.saturating_sub(7 * DAY_SECS);
    let d30 = now.saturating_sub(WINDOW_DAYS * DAY_SECS);
    for sample in samples {
        if sample.data_type != data_type || sample.timestamp > now {
            continue;
        }
        if sample.timestamp >= d30 {
            last_30d += 1;
        }
        if sample.timestamp >= d7 {
            last_7d += 1;
        }
        if sample.timestamp >= h24 {
            last_24h += 1;
        }
    }
    TypeCount {
        data_type: data_type.to_owned(),
        last_24h,
        last_7d,
        last_30d,
    }
}

fn hints(window: &[&SourceSample]) -> Vec<String> {
    if window.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if !window
        .iter()
        .any(|sample| sample.data_type == DATA_TYPE_HRV)
    {
        out.push(HINT_HRV_ABSENT.to_owned());
    }
    if !window
        .iter()
        .any(|sample| sample.data_type == DATA_TYPE_OXYGEN_SATURATION)
    {
        out.push(HINT_SPO2_ABSENT.to_owned());
    }
    if !window
        .iter()
        .any(|sample| sample.data_type == DATA_TYPE_RESPIRATORY_RATE)
    {
        out.push(HINT_RR_ABSENT.to_owned());
    }
    if !window
        .iter()
        .any(|sample| sample.data_type == DATA_TYPE_SLEEPING_WRIST_TEMPERATURE)
    {
        out.push(HINT_TEMP_ABSENT.to_owned());
    }
    let sleep: Vec<_> = window
        .iter()
        .filter(|sample| sample.data_type == DATA_TYPE_SLEEP_INTERVAL)
        .collect();
    if sleep.is_empty() {
        out.push(HINT_SLEEP_ABSENT.to_owned());
    } else if !sleep.iter().any(|sample| is_stage(&sample.sleep_stage)) {
        out.push(HINT_SLEEP_NO_STAGES.to_owned());
    }
    out
}

fn is_stage(stage: &Option<String>) -> bool {
    matches!(
        stage.as_deref(),
        Some(SLEEP_STAGE_ASLEEP_CORE | SLEEP_STAGE_ASLEEP_DEEP | SLEEP_STAGE_ASLEEP_REM)
    )
}

fn belgrade_day(unix: i64) -> i64 {
    (unix + belgrade_offset_secs(unix)).div_euclid(DAY_SECS)
}

/// EU civil-time offset for Europe/Belgrade: +2h in summer, +1h otherwise.
fn belgrade_offset_secs(unix: i64) -> i64 {
    let (year, _, _) = civil_from_days(unix.div_euclid(DAY_SECS));
    let start = last_sunday_utc(year, 3) + 3_600;
    let end = last_sunday_utc(year, 10) + 3_600;
    if unix >= start && unix < end {
        7_200
    } else {
        3_600
    }
}

fn last_sunday_utc(year: i32, month: u32) -> i64 {
    let last = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => 28,
    };
    let mut day = last;
    loop {
        let days = days_from_civil(year, month, day);
        if (days + 4).rem_euclid(7) == 0 {
            return days * DAY_SECS;
        }
        day -= 1;
        if day == 0 {
            return 0;
        }
    }
}

fn days_from_civil(mut y: i32, m: u32, d: u32) -> i64 {
    y -= i32::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let shifted = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * shifted + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(
        data_type: &str,
        timestamp: i64,
        received_at: i64,
        kind: &str,
        app: &str,
        model: &str,
    ) -> SourceSample {
        SourceSample {
            data_type: data_type.to_owned(),
            timestamp,
            received_at,
            kind: kind.to_owned(),
            app: app.to_owned(),
            device_model: model.to_owned(),
            sleep_stage: None,
        }
    }

    #[test]
    fn civil_epoch_and_belgrade_summer_offset() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        let oct = days_from_civil(2026, 10, 8) * DAY_SECS + 10 * 3_600;
        assert_eq!(belgrade_offset_secs(oct), 7_200);
        let jan = days_from_civil(2026, 1, 15) * DAY_SECS + 12 * 3_600;
        assert_eq!(belgrade_offset_secs(jan), 3_600);
    }

    #[test]
    fn groups_kind_app_and_model_and_keeps_receipt_apart_from_sample_time() {
        let now = days_from_civil(2026, 10, 8) * DAY_SECS + 12 * 3_600;
        let watch = sample(
            "heart_rate",
            now - 3_600,
            now - 60,
            "apple_watch",
            "Health",
            "Watch",
        );
        let mut late = watch.clone();
        late.timestamp = now - 10 * DAY_SECS;
        late.received_at = now - 30;
        let zepp = sample(
            "step_count",
            now - 2 * 3_600,
            now - 2 * 3_600,
            "zepp_life",
            "Zepp Life",
            "Band",
        );
        let report = summarize_sources(&[watch, late, zepp], now, &SourcePriority::default());
        assert_eq!(report.sources.len(), 2);
        let card = &report.sources[0];
        assert_eq!(card.kind, "apple_watch");
        assert_eq!(card.last_sample_unix, now - 3_600);
        assert_eq!(card.last_received_unix, now - 30);
        let hr = card
            .counts
            .iter()
            .find(|c| c.data_type == "heart_rate")
            .expect("hr");
        assert_eq!(hr.last_24h, 1);
        assert_eq!(hr.last_7d, 1);
        assert_eq!(hr.last_30d, 2);
        assert!(card.active);
        assert!(report.sources[1].active);
    }

    #[test]
    fn higher_priority_wins_only_when_it_has_the_type() {
        let now = 1_800_000_000;
        let watch = sample(
            "heart_rate",
            now - 10,
            now - 10,
            "apple_watch",
            "Health",
            "Watch",
        );
        let zepp_hr = sample(
            "heart_rate",
            now - 20,
            now - 20,
            "zepp_life",
            "Zepp Life",
            "Band",
        );
        let zepp_steps = sample(
            "step_count",
            now - 20,
            now - 20,
            "zepp_life",
            "Zepp Life",
            "Band",
        );
        let report = summarize_sources(
            &[watch, zepp_hr, zepp_steps],
            now,
            &SourcePriority::default(),
        );
        let watch_card = report
            .sources
            .iter()
            .find(|s| s.kind == "apple_watch")
            .expect("watch");
        let zepp_card = report
            .sources
            .iter()
            .find(|s| s.kind == "zepp_life")
            .expect("zepp");
        assert!(watch_card.active);
        assert!(zepp_card.active);
        assert!(watch_card.priority_rank < zepp_card.priority_rank);
    }

    #[test]
    fn missing_hrv_uses_the_xiaomi_hint_and_avoids_clinical_words() {
        let now = 1_800_000_000;
        let row = sample(
            "heart_rate",
            now - 10,
            now - 5,
            "zepp_life",
            "Zepp Life",
            "Band",
        );
        let report = summarize_sources(&[row], now, &SourcePriority::default());
        assert!(report.hints.iter().any(|hint| hint == HINT_HRV_ABSENT));
        let blob = report.hints.join(" ");
        for banned in ["апноэ", "аритм", "болезн", "выгоран", "стресс"]
        {
            assert!(!blob.to_lowercase().contains(banned), "{banned}");
        }
    }

    #[test]
    fn hrv_present_drops_the_hint_and_stages_need_core_deep_or_rem() {
        let now = 1_800_000_000;
        let hrv = sample("hrv", now - 10, now - 10, "apple_watch", "Health", "Watch");
        let mut sleep = sample(
            "sleep_interval",
            now - 8 * 3_600,
            now - 10,
            "apple_watch",
            "Health",
            "Watch",
        );
        sleep.sleep_stage = Some("asleep".to_owned());
        let without = summarize_sources(
            &[hrv.clone(), sleep.clone()],
            now,
            &SourcePriority::default(),
        );
        assert!(!without.hints.iter().any(|hint| hint.contains("HRV:")));
        assert!(
            without
                .hints
                .iter()
                .any(|hint| hint == HINT_SLEEP_NO_STAGES)
        );
        sleep.sleep_stage = Some(SLEEP_STAGE_ASLEEP_DEEP.to_owned());
        let with_stage = summarize_sources(&[hrv, sleep], now, &SourcePriority::default());
        assert!(!with_stage.hints.iter().any(|hint| hint.contains("фаз")));
    }

    #[test]
    fn coverage_is_the_share_of_belgrade_days() {
        let now = days_from_civil(2026, 10, 8) * DAY_SECS + 12 * 3_600;
        let mut rows = Vec::new();
        for day in 1..=8 {
            rows.push(sample(
                "heart_rate",
                days_from_civil(2026, 10, day) * DAY_SECS + 10 * 3_600,
                now,
                "apple_watch",
                "Health",
                "Watch",
            ));
        }
        let report = summarize_sources(&rows, now, &SourcePriority::default());
        let coverage = report.sources[0].coverage;
        assert!((coverage - 8.0 / 30.0).abs() < 0.000_1, "{coverage}");
    }

    #[test]
    fn empty_window_has_no_hints() {
        let report = summarize_sources(&[], 1_800_000_000, &SourcePriority::default());
        assert!(report.sources.is_empty());
        assert!(report.hints.is_empty());
        assert_eq!(
            report.priority.first().map(String::as_str),
            Some("apple_watch")
        );
    }

    #[test]
    fn priority_toml_round_trip_keeps_a_custom_order() {
        let priority = SourcePriority::from_order(["zepp_life", "apple_watch"]);
        let again = SourcePriority::from_toml_str(&priority.to_toml());
        assert_eq!(again.order(), priority.order());
        assert_eq!(again.order().first().map(String::as_str), Some("zepp_life"));
    }
}
