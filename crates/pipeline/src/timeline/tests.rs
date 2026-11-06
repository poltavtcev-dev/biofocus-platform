use bio_spec::{Observation, UnixTimestamp};
use serde_json::json;
use uuid::Uuid;

use super::*;

const PROVIDER: &str = "com.biofocus.applehealth";

fn obs(n: u128, ts: i64, data_type: &str, payload: serde_json::Value) -> Observation {
    Observation::try_new(
        Uuid::from_u128(n),
        UnixTimestamp::from_secs(ts),
        PROVIDER,
        data_type,
        payload,
        1.0,
    )
    .expect("valid observation")
}

/// Local noon (UTC 11:00 in winter) of a civil date.
fn noon(y: i64, m: u32, d: u32) -> i64 {
    days_from_civil(y, m, d) * SECS_PER_DAY + 11 * 3_600
}

fn sleep(n: u128, end: i64, minutes: i64, stage: &str, kind: &str) -> Observation {
    let start = end - minutes * 60;
    obs(
        n,
        start,
        DATA_TYPE_SLEEP_INTERVAL,
        json!({ "start": start, "end": end, "stage": stage, "src": { "kind": kind } }),
    )
}

fn steps(n: u128, ts: i64, count: u64, kind: &str) -> Observation {
    obs(
        n,
        ts,
        DATA_TYPE_STEP_COUNT,
        json!({ "count": count, "src": { "kind": kind } }),
    )
}

fn deletion(n: u128, target: u128) -> Observation {
    obs(
        n,
        noon(2026, 11, 5),
        "source_deletion",
        json!({ "target_id": Uuid::from_u128(target).to_string() }),
    )
}

fn total(totals: &[DailyTotal], metric: TimelineMetric, day: LocalDay) -> Option<&DailyTotal> {
    totals.iter().find(|t| t.metric == metric && t.day == day)
}

fn synthetic(metric: TimelineMetric, day: LocalDay, value: f64, kind: Option<&str>) -> DailyTotal {
    DailyTotal {
        day,
        metric,
        value,
        samples: 1,
        coverage_minutes: None,
        src_kind: kind.map(str::to_owned),
        confidence: 1.0,
    }
}

#[test]
fn belgrade_days_follow_dst() {
    // 2026-07-01 22:30 UTC is 00:30 on 2 July in Belgrade (CEST).
    let summer = days_from_civil(2026, 7, 1) * SECS_PER_DAY + 22 * 3_600 + 1_800;
    assert_eq!(day_label(local_day(summer)), "2026-07-02");
    // 2026-01-01 22:30 UTC is 23:30 on 1 January (CET).
    let winter = days_from_civil(2026, 1, 1) * SECS_PER_DAY + 22 * 3_600 + 1_800;
    assert_eq!(day_label(local_day(winter)), "2026-01-01");
    assert_eq!(
        weekday(days_from_civil(2026, 11, 2)),
        0,
        "2 Nov 2026 is Monday"
    );
}

#[test]
fn asleep_stages_sum_and_in_bed_stays_separate() {
    let wake = noon(2026, 11, 3) - 5 * 3_600; // 07:00 local
    let rows = vec![
        sleep(1, wake - 4 * 3_600, 120, "asleep_core", "apple_watch"),
        sleep(2, wake - 2 * 3_600, 60, "asleep_deep", "apple_watch"),
        sleep(3, wake - 3_600, 60, "asleep_rem", "apple_watch"),
        sleep(4, wake, 30, "asleep", "apple_watch"),
        sleep(5, wake, 540, "in_bed", "apple_watch"),
        sleep(6, wake - 1_800, 20, "awake", "apple_watch"),
    ];
    let totals = daily_totals(rows, &[], &SourcePriority::default());
    let day = days_from_civil(2026, 11, 3);
    let asleep = total(&totals, TimelineMetric::Sleep, day).expect("sleep day");
    assert_eq!(asleep.value, 270.0, "120+60+60+30, no in_bed, no awake");
    let in_bed = total(&totals, TimelineMetric::InBed, day).expect("in bed day");
    assert_eq!(in_bed.value, 540.0);
}

#[test]
fn only_in_bed_is_time_in_bed_not_sleep() {
    let wake = noon(2026, 11, 3) - 5 * 3_600;
    let totals = daily_totals(
        vec![sleep(1, wake, 480, "in_bed", "zepp_life")],
        &[],
        &SourcePriority::default(),
    );
    let day = days_from_civil(2026, 11, 3);
    assert!(total(&totals, TimelineMetric::Sleep, day).is_none());
    assert_eq!(
        total(&totals, TimelineMetric::InBed, day).unwrap().value,
        480.0
    );
}

#[test]
fn sleep_belongs_to_the_wake_date() {
    // 23:00 → 07:00 local: belongs to the morning date.
    let wake = noon(2026, 11, 4) - 4 * 3_600;
    let totals = daily_totals(
        vec![sleep(1, wake, 480, "asleep", "apple_watch")],
        &[],
        &SourcePriority::default(),
    );
    assert!(total(&totals, TimelineMetric::Sleep, days_from_civil(2026, 11, 4)).is_some());
    assert!(total(&totals, TimelineMetric::Sleep, days_from_civil(2026, 11, 3)).is_none());
}

#[test]
fn steps_use_p3_source_select_and_keep_src_kind() {
    let ts = noon(2026, 11, 3);
    let rows = vec![
        steps(1, ts, 6_000, "apple_watch"),
        steps(2, ts + 600, 2_000, "apple_watch"),
        steps(3, ts + 60, 9_000, "xiaomi_mi_fitness"),
    ];
    let day = days_from_civil(2026, 11, 3);

    let totals = daily_totals(rows.clone(), &[], &SourcePriority::default());
    let t = total(&totals, TimelineMetric::Steps, day).unwrap();
    assert_eq!(t.value, 8_000.0, "Watch only, never Watch + Mi Fitness");
    assert_eq!(t.src_kind.as_deref(), Some("apple_watch"));

    // Changing the P3 priority changes the daily kind: no second policy here.
    let priority = SourcePriority::from_order(["xiaomi_mi_fitness", "apple_watch"]);
    let totals = daily_totals(rows.clone(), &[], &priority);
    let t = total(&totals, TimelineMetric::Steps, day).unwrap();
    assert_eq!(t.value, 9_000.0);
    assert_eq!(t.src_kind.as_deref(), Some("xiaomi_mi_fitness"));

    // Same result as feeding the P3 selection output directly.
    let selected = select_sources(rows, &priority);
    assert!(
        selected
            .iter()
            .all(|o| o.payload["src"]["kind"] == "xiaomi_mi_fitness")
    );
}

#[test]
fn sleep_uses_p3_source_select() {
    let wake = noon(2026, 11, 3) - 5 * 3_600;
    let rows = vec![
        sleep(1, wake, 420, "asleep", "apple_watch"),
        sleep(2, wake + 60, 400, "asleep", "zepp_life"),
    ];
    let totals = daily_totals(rows, &[], &SourcePriority::default());
    let t = total(&totals, TimelineMetric::Sleep, days_from_civil(2026, 11, 3)).unwrap();
    assert_eq!(t.value, 420.0);
    assert_eq!(t.src_kind.as_deref(), Some("apple_watch"));
}

#[test]
fn source_deletion_removes_totals_coverage_and_comparisons() {
    let first = days_from_civil(2026, 10, 1);
    let mut rows = Vec::new();
    for i in 0..30u128 {
        let day = first + i as i64;
        rows.push(steps(
            100 + i,
            day * SECS_PER_DAY + 11 * 3_600,
            5_000,
            "apple_watch",
        ));
    }
    // Delete every Watch row from the last 7 days.
    let today = first + 30;
    let mut deleted = rows.clone();
    for i in 23..30u128 {
        deleted.push(deletion(500 + i, 100 + i));
    }
    let before = daily_totals(rows, &[], &SourcePriority::default());
    let after = daily_totals(deleted, &[], &SourcePriority::default());
    assert_eq!(before.len(), 30);
    assert_eq!(after.len(), 23);

    let deleted_day = first + 25;
    assert!(total(&after, TimelineMetric::Steps, deleted_day).is_none());
    let cal = coverage_calendar(&after, TimelineMetric::Steps, first, first + 29);
    assert_eq!(cal.days.iter().filter(|d| d.value.is_some()).count(), 23);
    assert!(
        cal.days
            .iter()
            .find(|d| d.day == deleted_day)
            .unwrap()
            .value
            .is_none()
    );

    let week_before = compare_windows(&before, TimelineMetric::Steps, CompareScale::Week, today);
    assert!(week_before.delta.is_some());
    let week_after = compare_windows(&after, TimelineMetric::Steps, CompareScale::Week, today);
    assert_eq!(week_after.current.days_with_data, 0);
    assert!(week_after.delta.is_none());
}

#[test]
fn deletion_from_another_provider_does_not_hide() {
    let ts = noon(2026, 11, 3);
    let mut marker = deletion(9, 1);
    marker.provider_id = "com.example.other".to_owned();
    let totals = daily_totals(
        vec![steps(1, ts, 1_000, "apple_watch"), marker],
        &[],
        &SourcePriority::default(),
    );
    assert_eq!(totals.len(), 1);
}

#[test]
fn empty_days_and_months_stay_empty() {
    let totals = vec![
        synthetic(
            TimelineMetric::Sleep,
            days_from_civil(2026, 1, 10),
            400.0,
            Some("zepp_life"),
        ),
        synthetic(
            TimelineMetric::Sleep,
            days_from_civil(2026, 3, 5),
            420.0,
            Some("zepp_life"),
        ),
    ];
    let cal = coverage_calendar(
        &totals,
        TimelineMetric::Sleep,
        days_from_civil(2026, 1, 1),
        days_from_civil(2026, 3, 31),
    );
    assert_eq!(cal.days.len(), 90);
    assert_eq!(cal.days.iter().filter(|d| d.value.is_some()).count(), 2);
    assert_eq!(cal.months.len(), 3);
    let feb = &cal.months[1];
    assert_eq!(
        (feb.month, feb.days_with_data, feb.days_in_range),
        (2, 0, 28)
    );
    assert_eq!(feb.median, None, "no zero, no interpolation");
    assert_eq!(cal.months[0].median, Some(400.0));
}

#[test]
fn week_withholds_delta_below_four_days() {
    let today = days_from_civil(2026, 11, 6);
    let mut totals = Vec::new();
    for d in 0..3 {
        totals.push(synthetic(
            TimelineMetric::Sleep,
            today - d,
            400.0,
            Some("apple_watch"),
        ));
    }
    for d in 7..13 {
        totals.push(synthetic(
            TimelineMetric::Sleep,
            today - d,
            450.0,
            Some("apple_watch"),
        ));
    }
    let c = compare_windows(&totals, TimelineMetric::Sleep, CompareScale::Week, today);
    assert_eq!(c.current.days_with_data, 3);
    assert!(c.delta.is_none());
    assert_eq!(
        c.text,
        "Недостаточно сопоставимых дней, чтобы сравнить эти недели."
    );

    totals.push(synthetic(
        TimelineMetric::Sleep,
        today - 3,
        400.0,
        Some("apple_watch"),
    ));
    let c = compare_windows(&totals, TimelineMetric::Sleep, CompareScale::Week, today);
    assert_eq!(c.delta, Some(-50.0));
    assert_eq!(c.direction, Some(Direction::Lower));
    assert_eq!(
        c.text,
        "За последние 7 дней медиана сна была на 50 минут короче, чем за 7 дней до этого (4 ночи против 6)."
    );
}

#[test]
fn week_withholds_delta_when_coverage_ratio_is_low() {
    // 4 vs 7 days → ratio 0.57 < 0.6.
    let today = days_from_civil(2026, 11, 6);
    let mut totals = Vec::new();
    for d in 0..4 {
        totals.push(synthetic(TimelineMetric::Sleep, today - d, 400.0, None));
    }
    for d in 7..14 {
        totals.push(synthetic(TimelineMetric::Sleep, today - d, 450.0, None));
    }
    let c = compare_windows(&totals, TimelineMetric::Sleep, CompareScale::Week, today);
    // 4/7 = 0.571
    assert!(c.delta.is_none());
}

#[test]
fn month_withholds_delta_below_fourteen_days() {
    let today = days_from_civil(2026, 11, 6);
    let end = window_end_day(TimelineMetric::Steps, today);
    let mut totals = Vec::new();
    for d in 0..13 {
        totals.push(synthetic(
            TimelineMetric::Steps,
            end - d,
            8_000.0,
            Some("iphone"),
        ));
    }
    for d in 30..50 {
        totals.push(synthetic(
            TimelineMetric::Steps,
            end - d,
            4_000.0,
            Some("iphone"),
        ));
    }
    let c = compare_windows(&totals, TimelineMetric::Steps, CompareScale::Month, today);
    assert!(c.delta.is_none());
    assert_eq!(
        c.text,
        "Недостаточно сопоставимых дней, чтобы сравнить эти месяцы."
    );

    for d in 13..16 {
        totals.push(synthetic(
            TimelineMetric::Steps,
            end - d,
            8_000.0,
            Some("iphone"),
        ));
    }
    let c = compare_windows(&totals, TimelineMetric::Steps, CompareScale::Month, today);
    assert_eq!(c.delta, Some(4_000.0));
    assert!(c.text.contains("Источник в основном iPhone."));
}

#[test]
fn steps_and_focus_windows_end_yesterday() {
    let today = days_from_civil(2026, 11, 6);
    assert_eq!(window_end_day(TimelineMetric::Steps, today), today - 1);
    assert_eq!(window_end_day(TimelineMetric::Focus, today), today - 1);
    assert_eq!(window_end_day(TimelineMetric::Sleep, today), today);
}

#[test]
fn sources_differ_is_marked() {
    let today = days_from_civil(2026, 11, 6);
    let mut totals = Vec::new();
    for d in 0..7 {
        totals.push(synthetic(
            TimelineMetric::Sleep,
            today - d,
            400.0,
            Some("apple_watch"),
        ));
        totals.push(synthetic(
            TimelineMetric::Sleep,
            today - 7 - d,
            300.0,
            Some("zepp_life"),
        ));
    }
    let c = compare_windows(&totals, TimelineMetric::Sleep, CompareScale::Week, today);
    assert!(c.sources_differ);
    assert!(
        c.text
            .contains("Источники различаются: сейчас Apple Watch, раньше Zepp Life.")
    );
}

#[test]
fn period_aggregate_is_median_not_mean() {
    let today = days_from_civil(2026, 11, 6);
    let values = [400.0, 410.0, 420.0, 430.0, 900.0];
    let totals: Vec<DailyTotal> = values
        .iter()
        .enumerate()
        .map(|(i, v)| synthetic(TimelineMetric::Sleep, today - i as i64, *v, None))
        .collect();
    let c = compare_windows(&totals, TimelineMetric::Sleep, CompareScale::Week, today);
    assert_eq!(c.current.median, Some(420.0), "mean would be 512");
    assert_eq!(median(&[1.0, 2.0, 3.0, 10.0]), Some(2.5));
    assert_eq!(median(&[]), None);
}

#[test]
fn similar_thresholds() {
    assert!(is_similar(TimelineMetric::Sleep, 14.0, 400.0));
    assert!(!is_similar(TimelineMetric::Sleep, 15.0, 400.0));
    assert!(is_similar(TimelineMetric::Steps, 999.0, 3_000.0));
    assert!(is_similar(TimelineMetric::Steps, 1_900.0, 20_000.0));
    assert!(!is_similar(TimelineMetric::Steps, 2_100.0, 20_000.0));
    assert!(is_similar(TimelineMetric::Focus, 4.9, 60.0));
    assert!(!is_similar(TimelineMetric::Focus, 5.0, 60.0));
}

fn context(n: u128, ts: i64) -> Observation {
    obs(
        n,
        ts,
        "context_window",
        json!({ "bundle_id": "com.apple.Terminal", "app_name": "Terminal" }),
    )
}

#[test]
fn focus_day_needs_two_hours_of_context_window() {
    let start = noon(2026, 11, 3) - 3 * 3_600; // 09:00 local
    let windows: Vec<FocusWindow> = (0..8)
        .map(|i| FocusWindow {
            end: start + (i + 1) * 900,
            score: 50.0 + i as f64,
            confidence: 0.8,
        })
        .collect();

    // Switches every 10 min for 1 h 50 min → 110 min + capped 15 for the last = 125 min.
    let enough: Vec<Observation> = (0..12)
        .map(|i| context(i as u128 + 1, start + i * 600))
        .collect();
    let totals = daily_totals(enough, &windows, &SourcePriority::default());
    let day = days_from_civil(2026, 11, 3);
    let focus = total(&totals, TimelineMetric::Focus, day).expect("≥ 2 h → Focus day");
    assert_eq!(focus.value, 53.5, "median of 50..57");
    assert!(focus.coverage_minutes.unwrap() >= 120);
    assert_eq!(focus.src_kind, None);

    // Ten switches 10 min apart → 90 + 15 = 105 min. Not a Focus day.
    let thin: Vec<Observation> = (0..10)
        .map(|i| context(i as u128 + 1, start + i * 600))
        .collect();
    let totals = daily_totals(thin, &windows, &SourcePriority::default());
    assert!(total(&totals, TimelineMetric::Focus, day).is_none());
}

#[test]
fn long_idle_gaps_are_capped() {
    let start = noon(2026, 11, 3);
    // Two switches 5 h apart count only 15 + 15 min.
    let secs = context_seconds_by_day(&[start, start + 5 * 3_600]);
    assert_eq!(secs.values().sum::<i64>(), 30 * 60);
}

#[test]
fn day_vs_typical_needs_three_same_weekdays() {
    let today = days_from_civil(2026, 11, 6);
    let mut totals = vec![synthetic(TimelineMetric::Sleep, today, 360.0, None)];
    totals.push(synthetic(TimelineMetric::Sleep, today - 7, 420.0, None));
    totals.push(synthetic(TimelineMetric::Sleep, today - 14, 430.0, None));
    let c = compare_day_to_typical(&totals, TimelineMetric::Sleep, today);
    assert!(c.delta.is_none());
    totals.push(synthetic(TimelineMetric::Sleep, today - 21, 440.0, None));
    let c = compare_day_to_typical(&totals, TimelineMetric::Sleep, today);
    assert_eq!(c.typical, Some(430.0));
    assert_eq!(c.delta, Some(-70.0));
}

#[test]
fn templates_have_no_forbidden_words() {
    let src = include_str!("../timeline.rs").to_lowercase();
    for word in [
        "организм",
        "диагноз",
        "болезн",
        "здоров",
        "нездоров",
        "норма",
        "для вашего возраста",
        "апноэ",
        "аритми",
        "выгоран",
        "депресс",
        "stress index",
    ] {
        assert!(!src.contains(word), "forbidden word in templates: {word}");
    }
}
