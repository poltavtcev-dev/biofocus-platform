//! «Динамика» — daily totals, coverage calendar, and week / month cards (timeline slice A).
//!
//! Recompute on read, nothing persisted. Source choice is the P3
//! [`pipeline::select_sources`] pass inside [`pipeline::timeline::daily_totals`]
//! with the installed [`SourcePriority`]. Deleted wearable rows are hidden by the
//! storage `source_deletion` filter, then again by `select_sources`.
//! UI never opens SQLite.

use std::time::{SystemTime, UNIX_EPOCH};

use bio_spec::{
    DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_SLEEP_INTERVAL, DATA_TYPE_STEP_COUNT, FeatureValue,
    Observation, UnixTimestamp,
};
use feature_engine::{FeatureEngine, register_catalog_v1};
use pipeline::SourcePriority;
use pipeline::timeline::{
    self, CompareScale, Comparison, CoverageCalendar, DailyTotal, Direction,
    FOCUS_MIN_CONTEXT_SECS, FocusWindow, LocalDay, TimelineMetric,
};
use serde::Serialize;
use storage::{Database, ObservationRepository};
use tracing::warn;

const SECS_PER_DAY: i64 = 86_400;
/// Coverage calendar span.
const CALENDAR_DAYS: i64 = 182;
/// Two 30-day windows plus the partial day.
const FOCUS_LOOKBACK_DAYS: i64 = 61;
const FOCUS_STEP_SECS: i64 = 900;
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";
const FOCUS_FEATURE_ID: &str = "FocusScore";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoverageDayDto {
    pub date: String,
    pub value: Option<f64>,
    pub src_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoverageMonthDto {
    pub year: i64,
    pub month: u32,
    pub days_with_data: u32,
    pub days_in_range: u32,
    pub median: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowDto {
    pub first_date: String,
    pub last_date: String,
    pub days_with_data: u32,
    pub median: Option<f64>,
    pub main_src_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonDto {
    pub scale: String,
    pub current: WindowDto,
    pub previous: WindowDto,
    pub delta: Option<f64>,
    /// `similar` / `higher` / `lower`, or `None` when withheld.
    pub direction: Option<String>,
    pub sources_differ: bool,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDto {
    pub metric: String,
    pub timezone: String,
    pub today: String,
    pub days: Vec<CoverageDayDto>,
    pub months: Vec<CoverageMonthDto>,
    pub comparisons: Vec<ComparisonDto>,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

fn window_dto(stat: &timeline::WindowStat) -> WindowDto {
    WindowDto {
        first_date: timeline::day_label(stat.first_day),
        last_date: timeline::day_label(stat.last_day),
        days_with_data: stat.days_with_data,
        median: stat.median,
        main_src_kind: stat.main_src_kind.clone(),
    }
}

fn comparison_dto(c: &Comparison) -> ComparisonDto {
    ComparisonDto {
        scale: c.scale.id().to_owned(),
        current: window_dto(&c.current),
        previous: window_dto(&c.previous),
        delta: c.delta,
        direction: c.direction.map(|d| {
            match d {
                Direction::Similar => "similar",
                Direction::Higher => "higher",
                Direction::Lower => "lower",
            }
            .to_owned()
        }),
        sources_differ: c.sources_differ,
        text: c.text.clone(),
    }
}

/// Pure mapping from daily totals to the DTO (tested without SQLite).
#[must_use]
pub fn build_dto(totals: &[DailyTotal], metric: TimelineMetric, today: LocalDay) -> TimelineDto {
    let CoverageCalendar { days, months, .. } =
        timeline::coverage_calendar(totals, metric, today - CALENDAR_DAYS + 1, today);
    let comparisons = [CompareScale::Week, CompareScale::Month]
        .into_iter()
        .map(|scale| comparison_dto(&timeline::compare_windows(totals, metric, scale, today)))
        .collect();
    TimelineDto {
        metric: metric.id().to_owned(),
        timezone: timeline::TIMELINE_TIMEZONE.to_owned(),
        today: timeline::day_label(today),
        days: days
            .into_iter()
            .map(|d| CoverageDayDto {
                date: timeline::day_label(d.day),
                value: d.value,
                src_kind: d.src_kind,
            })
            .collect(),
        months: months
            .into_iter()
            .map(|m| CoverageMonthDto {
                year: m.year,
                month: m.month,
                days_with_data: m.days_with_data,
                days_in_range: m.days_in_range,
                median: m.median,
            })
            .collect(),
        comparisons,
    }
}

fn types_for(metric: TimelineMetric) -> &'static [&'static str] {
    match metric {
        TimelineMetric::Sleep | TimelineMetric::InBed => &[DATA_TYPE_SLEEP_INTERVAL],
        TimelineMetric::Steps => &[DATA_TYPE_STEP_COUNT],
        TimelineMetric::ActiveEnergy => &[DATA_TYPE_ACTIVE_ENERGY],
        TimelineMetric::Focus => &[DATA_TYPE_CONTEXT_WINDOW],
    }
}

/// FocusScore windows for days that already have ≥ 2 h of `context_window`.
fn focus_windows(
    repo: &ObservationRepository<'_>,
    context: &[Observation],
    since: i64,
) -> Vec<FocusWindow> {
    let ts: Vec<i64> = context.iter().map(|o| o.timestamp.as_secs()).collect();
    let mut out = Vec::new();
    for (day, secs) in timeline::context_seconds_by_day(&ts) {
        if secs < FOCUS_MIN_CONTEXT_SECS || day * SECS_PER_DAY < since - SECS_PER_DAY {
            continue;
        }
        // Local day bounds (offset of noon is enough for Belgrade).
        let noon = day * SECS_PER_DAY + 12 * 3_600;
        let start = day * SECS_PER_DAY - timeline::belgrade_offset_secs(noon);
        let end = start + SECS_PER_DAY - 1;
        let Ok(observations) = repo.list_by_time_range(
            UnixTimestamp::from_secs(start),
            UnixTimestamp::from_secs(end),
        ) else {
            continue;
        };
        let mut engine = FeatureEngine::new();
        if register_catalog_v1(&mut engine).is_err() {
            return out;
        }
        let Ok(output) = engine.run_with_step(&observations, FOCUS_STEP_SECS) else {
            continue;
        };
        for feature in output.features {
            if feature.feature_id != FOCUS_FEATURE_ID {
                continue;
            }
            if let FeatureValue::Scalar(score) = feature.value {
                out.push(FocusWindow {
                    end: feature.time_window.end.as_secs(),
                    score,
                    confidence: feature.confidence.get(),
                });
            }
        }
    }
    out
}

fn load_totals(metric: TimelineMetric, now: i64) -> Result<Vec<DailyTotal>, String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    let span = if metric == TimelineMetric::Focus {
        FOCUS_LOOKBACK_DAYS
    } else {
        CALENDAR_DAYS + 1
    };
    let start = now - span * SECS_PER_DAY;
    let mut rows = Vec::new();
    for data_type in types_for(metric) {
        rows.extend(
            repo.list_by_data_type_in_range(
                data_type,
                UnixTimestamp::from_secs(start),
                UnixTimestamp::from_secs(now),
            )
            .map_err(|err| err.public_message())?,
        );
    }
    let focus = if metric == TimelineMetric::Focus {
        focus_windows(&repo, &rows, start)
    } else {
        Vec::new()
    };
    let priority = SourcePriority::load_installed();
    Ok(timeline::daily_totals(rows, &focus, &priority))
}

/// One metric's calendar and cards. Unknown metric or DB failure → empty calendar.
pub fn get_timeline(metric: &str) -> TimelineDto {
    let metric = TimelineMetric::from_id(metric).unwrap_or(TimelineMetric::Sleep);
    let now = now_secs();
    let totals = load_totals(metric, now).unwrap_or_else(|err| {
        warn!(error = %err, "timeline: load failed; empty");
        Vec::new()
    });
    build_dto(&totals, metric, timeline::local_day(now))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_totals_give_empty_days_and_withheld_cards() {
        let today = timeline::days_from_civil(2026, 11, 6);
        let dto = build_dto(&[], TimelineMetric::Sleep, today);
        assert_eq!(dto.days.len(), CALENDAR_DAYS as usize);
        assert!(dto.days.iter().all(|d| d.value.is_none()));
        assert!(dto.months.iter().all(|m| m.median.is_none()));
        assert_eq!(dto.comparisons.len(), 2);
        assert!(dto.comparisons.iter().all(|c| c.delta.is_none()));
        assert_eq!(dto.today, "2026-11-06");
        assert_eq!(dto.timezone, "Europe/Belgrade");
    }
}
