//! Comparative timeline, slice A: daily totals, coverage, week / month comparisons.
//!
//! Spec: `spec-comparative-timeline-2026-11-06.md` §5–§8.
//!
//! - Days are local calendar days in Europe/Belgrade.
//! - Source choice is the P3 [`select_sources`] pass with the same
//!   [`SourcePriority`]; this module adds no second policy. When one local day
//!   still holds two kinds (two UTC buckets overlap it), the same priority
//!   order picks one kind for the day. `source_deletion` is applied by
//!   [`select_sources`] before anything is counted.
//! - Sleep sums only asleep stages. `in_bed` is its own metric.
//! - Period aggregate is the median of daily values. Days without data stay empty.
//! - Texts are fixed templates (§8). No health verdicts.

use std::collections::BTreeMap;

use bio_spec::{
    DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_SLEEP_INTERVAL, DATA_TYPE_STEP_COUNT, Observation,
    SLEEP_STAGE_ASLEEP, SLEEP_STAGE_ASLEEP_CORE, SLEEP_STAGE_ASLEEP_DEEP, SLEEP_STAGE_ASLEEP_REM,
    SLEEP_STAGE_IN_BED,
};

use crate::source_select::{SourcePriority, select_sources};

const SECS_PER_DAY: i64 = 86_400;
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";
/// Minimum `context_window` time for a Focus day (§5).
pub const FOCUS_MIN_CONTEXT_SECS: i64 = 2 * 3_600;
/// A gap between app switches longer than this counts only up to this cap.
const CONTEXT_GAP_CAP_SECS: i64 = 15 * 60;
const ASLEEP_STAGES: &[&str] = &[
    SLEEP_STAGE_ASLEEP,
    SLEEP_STAGE_ASLEEP_CORE,
    SLEEP_STAGE_ASLEEP_DEEP,
    SLEEP_STAGE_ASLEEP_REM,
];

/// Display zone for every day boundary.
pub const TIMELINE_TIMEZONE: &str = "Europe/Belgrade";

/// Slice A metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimelineMetric {
    /// Asleep minutes (asleep, core, deep, REM). Never includes `in_bed`.
    Sleep,
    /// Time in bed, minutes. Separate series.
    InBed,
    Steps,
    ActiveEnergy,
    /// Median FocusScore of the day.
    Focus,
}

impl TimelineMetric {
    pub const ALL: [Self; 5] = [
        Self::Sleep,
        Self::InBed,
        Self::Steps,
        Self::ActiveEnergy,
        Self::Focus,
    ];

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Sleep => "sleep",
            Self::InBed => "in_bed",
            Self::Steps => "steps",
            Self::ActiveEnergy => "active_energy",
            Self::Focus => "focus",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|metric| metric.id() == id)
    }

    fn is_night(self) -> bool {
        matches!(self, Self::Sleep | Self::InBed)
    }
}

/// Local calendar day as days since 1970-01-01 (local).
pub type LocalDay = i64;

/// One finished FocusScore window (from the Feature Engine).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusWindow {
    pub end: i64,
    pub score: f64,
    pub confidence: f64,
}

/// One row per local day and metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyTotal {
    pub day: LocalDay,
    pub metric: TimelineMetric,
    /// Minutes for sleep / in bed, steps, kcal, or FocusScore points.
    pub value: f64,
    /// Source rows (or Focus windows) behind the value.
    pub samples: u32,
    /// Measured minutes: sleep intervals, or `context_window` time for Focus.
    pub coverage_minutes: Option<u32>,
    /// `src.kind` the P3 selection kept for this day. `None` for Focus (Mac).
    pub src_kind: Option<String>,
    /// Mean confidence of the rows behind the value.
    pub confidence: f64,
}

// ---------- Europe/Belgrade time ----------

/// UTC offset of Europe/Belgrade at `ts` (CET +1, CEST +2; EU rule).
#[must_use]
pub fn belgrade_offset_secs(ts: i64) -> i64 {
    let (year, _, _) = civil_from_days(ts.div_euclid(SECS_PER_DAY));
    let start = last_sunday_utc(year, 3) + 3_600;
    let end = last_sunday_utc(year, 10) + 3_600;
    if ts >= start && ts < end {
        7_200
    } else {
        3_600
    }
}

/// Local day that contains `ts`.
#[must_use]
pub fn local_day(ts: i64) -> LocalDay {
    (ts + belgrade_offset_secs(ts)).div_euclid(SECS_PER_DAY)
}

/// `YYYY-MM-DD` for a local day.
#[must_use]
pub fn day_label(day: LocalDay) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `(year, month)` for a local day.
#[must_use]
pub fn day_month(day: LocalDay) -> (i64, u32) {
    let (y, m, _) = civil_from_days(day);
    (y, m)
}

/// Days since epoch for a civil date (proleptic Gregorian).
#[must_use]
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

fn last_sunday_utc(year: i64, month: u32) -> i64 {
    let next_first = if month == 12 {
        days_from_civil(year + 1, 1, 1)
    } else {
        days_from_civil(year, month + 1, 1)
    };
    let last = next_first - 1;
    // 1970-01-01 was a Thursday; weekday 0 = Monday.
    let weekday = (last + 3).rem_euclid(7);
    let sunday = last - ((weekday - 6).rem_euclid(7));
    sunday * SECS_PER_DAY
}

/// 0 = Monday … 6 = Sunday.
#[must_use]
pub fn weekday(day: LocalDay) -> i64 {
    (day + 3).rem_euclid(7)
}

// ---------- daily totals ----------

fn src_kind(obs: &Observation) -> Option<&str> {
    obs.payload.get("src")?.get("kind")?.as_str()
}

fn i64_field(obs: &Observation, key: &str) -> Option<i64> {
    obs.payload.get(key).and_then(serde_json::Value::as_i64)
}

#[derive(Default)]
struct DayAcc {
    value: f64,
    samples: u32,
    minutes: f64,
    confidence_sum: f64,
}

/// Daily totals for sleep, time in bed, steps, active energy, and Focus.
///
/// `observations` may hold raw rows, including `source_deletion` markers;
/// [`select_sources`] runs first with `priority`.
#[must_use]
pub fn daily_totals(
    observations: Vec<Observation>,
    focus_windows: &[FocusWindow],
    priority: &SourcePriority,
) -> Vec<DailyTotal> {
    let selected = select_sources(observations, priority);

    // (metric, day) -> kind -> acc
    let mut acc: BTreeMap<(TimelineMetric, LocalDay), BTreeMap<String, DayAcc>> = BTreeMap::new();
    let mut context_ts: Vec<i64> = Vec::new();

    for obs in &selected {
        let confidence = obs.confidence.get();
        match obs.data_type.as_str() {
            DATA_TYPE_SLEEP_INTERVAL => {
                let Some(start) = i64_field(obs, "start") else {
                    continue;
                };
                let Some(end) = i64_field(obs, "end") else {
                    continue;
                };
                let stage = obs
                    .payload
                    .get("stage")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(SLEEP_STAGE_ASLEEP);
                let metric = if ASLEEP_STAGES.contains(&stage) {
                    TimelineMetric::Sleep
                } else if stage == SLEEP_STAGE_IN_BED {
                    TimelineMetric::InBed
                } else {
                    continue;
                };
                let minutes = (end - start).max(0) as f64 / 60.0;
                let kind = src_kind(obs).unwrap_or("").to_owned();
                let entry = acc
                    .entry((metric, local_day(end)))
                    .or_default()
                    .entry(kind)
                    .or_default();
                entry.value += minutes;
                entry.minutes += minutes;
                entry.samples += 1;
                entry.confidence_sum += confidence;
            }
            DATA_TYPE_STEP_COUNT | DATA_TYPE_ACTIVE_ENERGY => {
                let (metric, value) = if obs.data_type == DATA_TYPE_STEP_COUNT {
                    let Some(count) = obs.payload.get("count").and_then(serde_json::Value::as_f64)
                    else {
                        continue;
                    };
                    (TimelineMetric::Steps, count)
                } else {
                    let Some(kcal) = obs.payload.get("kcal").and_then(serde_json::Value::as_f64)
                    else {
                        continue;
                    };
                    (TimelineMetric::ActiveEnergy, kcal)
                };
                let kind = src_kind(obs).unwrap_or("").to_owned();
                let entry = acc
                    .entry((metric, local_day(obs.timestamp.as_secs())))
                    .or_default()
                    .entry(kind)
                    .or_default();
                entry.value += value;
                entry.samples += 1;
                entry.confidence_sum += confidence;
            }
            DATA_TYPE_CONTEXT_WINDOW => context_ts.push(obs.timestamp.as_secs()),
            _ => {}
        }
    }

    let mut out = Vec::new();
    for ((metric, day), kinds) in acc {
        // Same P3 priority order picks one kind per local day. Never summed.
        let Some((kind, day_acc)) = kinds
            .into_iter()
            .min_by_key(|(kind, _)| priority.rank_of(kind))
        else {
            continue;
        };
        if day_acc.samples == 0 {
            continue;
        }
        out.push(DailyTotal {
            day,
            metric,
            value: day_acc.value,
            samples: day_acc.samples,
            coverage_minutes: metric.is_night().then(|| day_acc.minutes.round() as u32),
            src_kind: (!kind.is_empty()).then_some(kind),
            confidence: day_acc.confidence_sum / f64::from(day_acc.samples),
        });
    }

    out.extend(focus_totals(&context_ts, focus_windows));
    out.sort_by(|a, b| (a.metric, a.day).cmp(&(b.metric, b.day)));
    out
}

/// Seconds of `context_window` time per local day. Each switch counts until the
/// next one, capped at 15 minutes (idle Mac does not count).
#[must_use]
pub fn context_seconds_by_day(context_ts: &[i64]) -> BTreeMap<LocalDay, i64> {
    let mut ts = context_ts.to_vec();
    ts.sort_unstable();
    ts.dedup();
    let mut out: BTreeMap<LocalDay, i64> = BTreeMap::new();
    for (index, &at) in ts.iter().enumerate() {
        let next = ts
            .get(index + 1)
            .copied()
            .unwrap_or(at + CONTEXT_GAP_CAP_SECS);
        let span = (next - at).clamp(0, CONTEXT_GAP_CAP_SECS);
        *out.entry(local_day(at)).or_default() += span;
    }
    out
}

fn focus_totals(context_ts: &[i64], windows: &[FocusWindow]) -> Vec<DailyTotal> {
    let context = context_seconds_by_day(context_ts);
    let mut by_day: BTreeMap<LocalDay, Vec<FocusWindow>> = BTreeMap::new();
    for window in windows {
        if window.score.is_finite() {
            by_day
                .entry(local_day(window.end))
                .or_default()
                .push(*window);
        }
    }
    by_day
        .into_iter()
        .filter_map(|(day, day_windows)| {
            let secs = context.get(&day).copied().unwrap_or(0);
            if secs < FOCUS_MIN_CONTEXT_SECS {
                return None;
            }
            let scores: Vec<f64> = day_windows.iter().map(|w| w.score).collect();
            let value = median(&scores)?;
            let confidence =
                day_windows.iter().map(|w| w.confidence).sum::<f64>() / day_windows.len() as f64;
            Some(DailyTotal {
                day,
                metric: TimelineMetric::Focus,
                value,
                samples: day_windows.len() as u32,
                coverage_minutes: Some((secs / 60) as u32),
                src_kind: None,
                confidence,
            })
        })
        .collect()
}

/// Median of finite values. Empty → `None`.
#[must_use]
pub fn median(values: &[f64]) -> Option<f64> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    Some(if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    })
}

// ---------- coverage calendar ----------

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageDay {
    pub day: LocalDay,
    /// `None` = no data. Never filled.
    pub value: Option<f64>,
    pub src_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageMonth {
    pub year: i64,
    pub month: u32,
    pub days_with_data: u32,
    pub days_in_range: u32,
    /// Median of days with data. `None` when the month is empty.
    pub median: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageCalendar {
    pub metric: TimelineMetric,
    pub days: Vec<CoverageDay>,
    pub months: Vec<CoverageMonth>,
}

/// Every local day in `[from, to]`. Days without data keep `value: None`.
#[must_use]
pub fn coverage_calendar(
    totals: &[DailyTotal],
    metric: TimelineMetric,
    from: LocalDay,
    to: LocalDay,
) -> CoverageCalendar {
    let by_day: BTreeMap<LocalDay, &DailyTotal> = totals
        .iter()
        .filter(|t| t.metric == metric)
        .map(|t| (t.day, t))
        .collect();
    let mut days = Vec::new();
    let mut months: Vec<CoverageMonth> = Vec::new();
    let mut month_values: Vec<f64> = Vec::new();
    for day in from..=to.max(from) {
        let total = by_day.get(&day);
        let (year, month) = day_month(day);
        if months
            .last()
            .is_none_or(|m| (m.year, m.month) != (year, month))
        {
            if let Some(last) = months.last_mut() {
                last.median = median(&month_values);
            }
            month_values.clear();
            months.push(CoverageMonth {
                year,
                month,
                days_with_data: 0,
                days_in_range: 0,
                median: None,
            });
        }
        let current = months.last_mut().expect("pushed above");
        current.days_in_range += 1;
        if let Some(total) = total {
            current.days_with_data += 1;
            month_values.push(total.value);
        }
        days.push(CoverageDay {
            day,
            value: total.map(|t| t.value),
            src_kind: total.and_then(|t| t.src_kind.clone()),
        });
    }
    if let Some(last) = months.last_mut() {
        last.median = median(&month_values);
    }
    CoverageCalendar {
        metric,
        days,
        months,
    }
}

// ---------- comparisons ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareScale {
    /// Last 7 days vs the 7 before.
    Week,
    /// Last 30 days vs the 30 before.
    Month,
}

impl CompareScale {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    fn len(self) -> i64 {
        match self {
            Self::Week => 7,
            Self::Month => 30,
        }
    }

    /// §6 floor of days with data in each window.
    #[must_use]
    pub fn min_days(self) -> u32 {
        match self {
            Self::Week => 4,
            Self::Month => 14,
        }
    }
}

/// §6: days-with-data ratio between the two windows.
pub const MIN_COVERAGE_RATIO: f64 = 0.6;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowStat {
    pub first_day: LocalDay,
    pub last_day: LocalDay,
    pub days_with_data: u32,
    pub median: Option<f64>,
    /// Kind on most days. `None` when no kind (Focus) or empty.
    pub main_src_kind: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Similar,
    Higher,
    Lower,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub metric: TimelineMetric,
    pub scale: CompareScale,
    pub current: WindowStat,
    pub previous: WindowStat,
    /// `None` when §6 floors are not met. Then `text` is the honest empty status.
    pub delta: Option<f64>,
    pub direction: Option<Direction>,
    pub sources_differ: bool,
    pub text: String,
}

fn window_stat(
    totals: &[DailyTotal],
    metric: TimelineMetric,
    first: LocalDay,
    last: LocalDay,
) -> WindowStat {
    let rows: Vec<&DailyTotal> = totals
        .iter()
        .filter(|t| t.metric == metric && t.day >= first && t.day <= last)
        .collect();
    let values: Vec<f64> = rows.iter().map(|t| t.value).collect();
    let mut kinds: BTreeMap<&str, u32> = BTreeMap::new();
    for row in &rows {
        if let Some(kind) = row.src_kind.as_deref() {
            *kinds.entry(kind).or_default() += 1;
        }
    }
    let main_src_kind = kinds
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(kind, _)| kind.to_owned());
    WindowStat {
        first_day: first,
        last_day: last,
        days_with_data: rows.len() as u32,
        median: median(&values),
        main_src_kind,
    }
}

/// Last full day used as the window end. Nights end on today's wake date;
/// steps, energy, and Focus end yesterday so a half-finished day is not compared.
#[must_use]
pub fn window_end_day(metric: TimelineMetric, today: LocalDay) -> LocalDay {
    if metric.is_night() { today } else { today - 1 }
}

/// Is `delta` inside the §6 "similar" threshold?
#[must_use]
pub fn is_similar(metric: TimelineMetric, delta: f64, previous: f64) -> bool {
    let abs = delta.abs();
    match metric {
        TimelineMetric::Sleep | TimelineMetric::InBed => abs < 15.0,
        TimelineMetric::Steps => abs < (previous.abs() * 0.10).max(1_000.0),
        // Not in the spec table: same 10 % rule as steps, no absolute floor.
        TimelineMetric::ActiveEnergy => abs < previous.abs() * 0.10,
        TimelineMetric::Focus => abs < 5.0,
    }
}

/// §6 comparison of the latest window with the one before it.
#[must_use]
pub fn compare_windows(
    totals: &[DailyTotal],
    metric: TimelineMetric,
    scale: CompareScale,
    today: LocalDay,
) -> Comparison {
    let end = window_end_day(metric, today);
    let len = scale.len();
    let current = window_stat(totals, metric, end - len + 1, end);
    let previous = window_stat(totals, metric, end - 2 * len + 1, end - len);

    let floor = scale.min_days();
    let enough = current.days_with_data >= floor && previous.days_with_data >= floor;
    let ratio = if enough {
        let (lo, hi) = (
            current.days_with_data.min(previous.days_with_data),
            current.days_with_data.max(previous.days_with_data),
        );
        f64::from(lo) / f64::from(hi)
    } else {
        0.0
    };
    let sources_differ = matches!(
        (&current.main_src_kind, &previous.main_src_kind),
        (Some(a), Some(b)) if a != b
    );

    let (delta, direction) = match (
        enough && ratio >= MIN_COVERAGE_RATIO,
        current.median,
        previous.median,
    ) {
        (true, Some(cur), Some(prev)) => {
            let delta = cur - prev;
            let direction = if is_similar(metric, delta, prev) {
                Direction::Similar
            } else if delta > 0.0 {
                Direction::Higher
            } else {
                Direction::Lower
            };
            (Some(delta), Some(direction))
        }
        _ => (None, None),
    };

    let text = match (delta, direction) {
        (Some(delta), Some(direction)) => comparison_text(
            metric,
            scale,
            &current,
            &previous,
            delta,
            direction,
            sources_differ,
        ),
        _ => insufficient_text(scale).to_owned(),
    };

    Comparison {
        metric,
        scale,
        current,
        previous,
        delta,
        direction,
        sources_differ,
        text,
    }
}

/// Today (or last night) vs the median of the same weekday over the last 8 weeks.
#[derive(Debug, Clone, PartialEq)]
pub struct DayComparison {
    pub metric: TimelineMetric,
    pub day: LocalDay,
    pub value: Option<f64>,
    pub same_weekdays: u32,
    pub typical: Option<f64>,
    pub delta: Option<f64>,
    pub direction: Option<Direction>,
    pub text: String,
}

/// §6 day row: needs a value on `day` and ≥ 3 same weekdays in the 8 weeks before.
#[must_use]
pub fn compare_day_to_typical(
    totals: &[DailyTotal],
    metric: TimelineMetric,
    day: LocalDay,
) -> DayComparison {
    let rows: Vec<&DailyTotal> = totals.iter().filter(|t| t.metric == metric).collect();
    let value = rows.iter().find(|t| t.day == day).map(|t| t.value);
    let earlier: Vec<f64> = (1..=8)
        .filter_map(|week| {
            let d = day - 7 * week;
            rows.iter().find(|t| t.day == d).map(|t| t.value)
        })
        .collect();
    let typical = if earlier.len() >= 3 {
        median(&earlier)
    } else {
        None
    };
    let (delta, direction) = match (value, typical) {
        (Some(v), Some(t)) => {
            let delta = v - t;
            let direction = if is_similar(metric, delta, t) {
                Direction::Similar
            } else if delta > 0.0 {
                Direction::Higher
            } else {
                Direction::Lower
            };
            (Some(delta), Some(direction))
        }
        _ => (None, None),
    };
    let text = match (delta, direction) {
        (Some(delta), Some(direction)) => {
            let unit = days_word(metric, earlier.len() as u32);
            let what = metric_subject(metric);
            match direction {
                Direction::Similar => format!(
                    "{what} примерно как в обычный такой же день недели (медиана {} {unit}).",
                    earlier.len()
                ),
                _ => format!(
                    "{what} {} обычного такого же дня недели (медиана {} {unit}).",
                    delta_phrase(metric, delta, typical.unwrap_or(0.0)),
                    earlier.len()
                ),
            }
        }
        _ => "Недостаточно таких же дней недели, чтобы сравнить.".to_owned(),
    };
    DayComparison {
        metric,
        day,
        value,
        same_weekdays: earlier.len() as u32,
        typical,
        delta,
        direction,
        text,
    }
}

// ---------- templates (§8) ----------

/// Honest empty status for a scale.
#[must_use]
pub fn insufficient_text(scale: CompareScale) -> &'static str {
    match scale {
        CompareScale::Week => "Недостаточно сопоставимых дней, чтобы сравнить эти недели.",
        CompareScale::Month => "Недостаточно сопоставимых дней, чтобы сравнить эти месяцы.",
    }
}

/// Russian label for a `src.kind`.
#[must_use]
pub fn src_kind_label(kind: &str) -> &'static str {
    match kind {
        "apple_watch" => "Apple Watch",
        "iphone" => "iPhone",
        "xiaomi_mi_fitness" => "Mi Fitness",
        "zepp_life" => "Zepp Life",
        "other_app" => "другое приложение",
        "manual" => "ручной ввод",
        _ => "неизвестный источник",
    }
}

fn plural(n: u32, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    let n100 = n % 100;
    let n10 = n % 10;
    if (11..=14).contains(&n100) {
        many
    } else if n10 == 1 {
        one
    } else if (2..=4).contains(&n10) {
        few
    } else {
        many
    }
}

fn days_word(metric: TimelineMetric, n: u32) -> &'static str {
    if metric.is_night() {
        plural(n, "ночь", "ночи", "ночей")
    } else {
        plural(n, "день", "дня", "дней")
    }
}

fn metric_subject(metric: TimelineMetric) -> &'static str {
    match metric {
        TimelineMetric::Sleep => "Сон",
        TimelineMetric::InBed => "Время в постели",
        TimelineMetric::Steps => "Шаги",
        TimelineMetric::ActiveEnergy => "Активная энергия",
        TimelineMetric::Focus => "Focus",
    }
}

fn metric_genitive(metric: TimelineMetric) -> &'static str {
    match metric {
        TimelineMetric::Sleep => "сна",
        TimelineMetric::InBed => "времени в постели",
        TimelineMetric::Steps => "шагов",
        TimelineMetric::ActiveEnergy => "активной энергии",
        TimelineMetric::Focus => "Focus",
    }
}

fn format_minutes(minutes: f64) -> String {
    let total = minutes.abs().round() as i64;
    let (h, m) = (total / 60, total % 60);
    if h == 0 {
        format!("{m} {}", plural(m as u32, "минуту", "минуты", "минут"))
    } else if m == 0 {
        format!("{h} ч")
    } else {
        format!("{h} ч {m} мин")
    }
}

fn delta_phrase(metric: TimelineMetric, delta: f64, previous: f64) -> String {
    let up = delta > 0.0;
    match metric {
        TimelineMetric::Sleep | TimelineMetric::InBed => format!(
            "на {} {}",
            format_minutes(delta),
            if up { "длиннее" } else { "короче" }
        ),
        TimelineMetric::Steps => {
            let abs = delta.abs().round() as i64;
            let pct = if previous > 0.0 {
                format!(
                    " ({}{:.0}%)",
                    if up { "+" } else { "−" },
                    delta.abs() / previous * 100.0
                )
            } else {
                String::new()
            };
            format!(
                "на {abs} {} {}{pct}",
                plural(abs as u32, "шаг", "шага", "шагов"),
                if up { "больше" } else { "меньше" }
            )
        }
        TimelineMetric::ActiveEnergy => format!(
            "на {:.0} ккал {}",
            delta.abs(),
            if up { "больше" } else { "меньше" }
        ),
        TimelineMetric::Focus => {
            let abs = delta.abs().round() as u32;
            format!(
                "на {abs} {} {}",
                plural(abs, "пункт", "пункта", "пунктов"),
                if up { "выше" } else { "ниже" }
            )
        }
    }
}

fn comparison_text(
    metric: TimelineMetric,
    scale: CompareScale,
    current: &WindowStat,
    previous: &WindowStat,
    delta: f64,
    direction: Direction,
    sources_differ: bool,
) -> String {
    let (lead, other) = match scale {
        CompareScale::Week => ("За последние 7 дней", "за 7 дней до этого"),
        CompareScale::Month => ("За последние 30 дней", "за 30 дней до этого"),
    };
    let counts = format!(
        "{} {} против {}",
        current.days_with_data,
        days_word(metric, current.days_with_data),
        previous.days_with_data
    );
    let what = metric_genitive(metric);
    let mut text = match direction {
        Direction::Similar => {
            format!("{lead} медиана {what} была примерно такой же, как {other} ({counts}).")
        }
        _ => format!(
            "{lead} медиана {what} была {}, чем {other} ({counts}).",
            delta_phrase(metric, delta, previous.median.unwrap_or(0.0))
        ),
    };
    if sources_differ {
        let a = current
            .main_src_kind
            .as_deref()
            .map(src_kind_label)
            .unwrap_or("");
        let b = previous
            .main_src_kind
            .as_deref()
            .map(src_kind_label)
            .unwrap_or("");
        text.push_str(&format!(" Источники различаются: сейчас {a}, раньше {b}."));
    } else if let Some(kind) = current.main_src_kind.as_deref() {
        if kind != "apple_watch" {
            text.push_str(&format!(" Источник в основном {}.", src_kind_label(kind)));
        }
    }
    text
}

#[cfg(test)]
mod tests;
