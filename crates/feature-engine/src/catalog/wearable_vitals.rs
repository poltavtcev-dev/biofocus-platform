//! Daily wearable vitals (ADR-030). Display next to the person's own days.
//!
//! Resting heart rate, HRV versus a personal median, sleep stages, night SpO2,
//! breathing rate, and wrist temperature. HRV emits only after five earlier
//! days of the same method, and SDNN is never averaged with RMSSD. No alerts.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use bio_spec::{
    Confidence, ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, UnixTimestamp,
    DATA_TYPE_HRV, DATA_TYPE_OXYGEN_SATURATION, DATA_TYPE_RESPIRATORY_RATE,
    DATA_TYPE_RESTING_HEART_RATE, DATA_TYPE_SLEEPING_WRIST_TEMPERATURE, DATA_TYPE_SLEEP_INTERVAL,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::hrv::{mean_method, sample_method, HrvMethod};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Latest resting heart rate for the day, plus a delta once five earlier days exist.
pub const RESTING_HEART_RATE_ID: &str = "RestingHeartRate";
/// One HRV method versus the median of earlier days of that same method.
pub const HRV_VS_BASELINE_ID: &str = "HrvVsBaseline";
/// Asleep time, and deep / REM / core shares when the source wrote stages.
pub const SLEEP_STAGES_ID: &str = "SleepStages";
/// Night oxygen saturation: mean, minimum, and a personal delta when available.
pub const NIGHT_SPO2_ID: &str = "NightSpO2";
/// Breathing rate: mean, minimum, and a personal delta when available.
pub const RESPIRATORY_RATE_ID: &str = "RespiratoryRate";
/// Wrist-temperature change versus the person's recent nights.
pub const WRIST_TEMPERATURE_ID: &str = "WristTemperature";

/// Earlier days required before a personal median is attached or, for HRV, before any emit.
pub const BASELINE_MIN_DAYS: usize = 5;
const RHR_LOOKBACK_DAYS: i64 = 14;
const HRV_LOOKBACK_DAYS: i64 = 28;
const NIGHT_LOOKBACK_DAYS: i64 = 14;
const BEDTIME_MIN_NIGHTS: usize = 3;
const SECS_PER_DAY: i64 = 86_400;

/// DAG node that emits the wearable vital Features. No Feature dependencies.
#[derive(Debug, Default, Clone)]
pub struct WearableVitalsNode;

impl WearableVitalsNode {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for WearableVitalsNode {
    fn id(&self) -> &str {
        "WearableVitals"
    }

    fn depends_on(&self) -> &[NodeId] {
        &[]
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let observations = ctx.observations();
        let mut features = Vec::new();
        features.extend(resting_features(observations));
        features.extend(hrv_features(observations));
        features.extend(sleep_features(observations));
        features.extend(nightly_features(
            observations,
            DATA_TYPE_OXYGEN_SATURATION,
            "spo2_percent",
            NIGHT_SPO2_ID,
            "mean_percent",
            Some("min_percent"),
            "baseline_mean",
            "delta_mean",
        ));
        features.extend(nightly_features(
            observations,
            DATA_TYPE_RESPIRATORY_RATE,
            "breaths_per_min",
            RESPIRATORY_RATE_ID,
            "mean_per_min",
            Some("min_per_min"),
            "baseline_mean",
            "delta_mean",
        ));
        features.extend(nightly_features(
            observations,
            DATA_TYPE_SLEEPING_WRIST_TEMPERATURE,
            "celsius",
            WRIST_TEMPERATURE_ID,
            "celsius",
            None,
            "baseline_celsius",
            "delta_celsius",
        ));
        Ok(NodeOutput::features(features))
    }
}

struct DayStat {
    day: i64,
    value: f64,
}

fn resting_features(observations: &[Observation]) -> Vec<Feature> {
    let samples = of_type(observations, DATA_TYPE_RESTING_HEART_RATE);
    let days = by_utc_day(&samples);
    let stats: Vec<DayStat> = days
        .iter()
        .filter_map(|(day, day_samples)| {
            last_f64(day_samples, "bpm").map(|bpm| DayStat {
                day: *day,
                value: bpm,
            })
        })
        .collect();

    let mut features = Vec::new();
    for stat in &stats {
        let Some(day_samples) = days.get(&stat.day) else {
            continue;
        };
        let mut value = serde_json::Map::new();
        value.insert("bpm".to_owned(), serde_json::json!(stat.value));
        if let Some((median, n)) = baseline_median(&stats, stat.day, RHR_LOOKBACK_DAYS) {
            value.insert("baseline_bpm".to_owned(), serde_json::json!(median));
            value.insert(
                "delta_bpm".to_owned(),
                serde_json::json!(stat.value - median),
            );
            value.insert("baseline_days".to_owned(), serde_json::json!(n));
        }
        let end = day_samples
            .last()
            .map(|o| o.timestamp.as_secs())
            .unwrap_or(stat.day);
        features.push(emit(
            RESTING_HEART_RATE_ID,
            stat.day,
            end,
            serde_json::Value::Object(value),
            day_samples,
        ));
    }
    features
}

fn hrv_features(observations: &[Observation]) -> Vec<Feature> {
    let samples = of_type(observations, DATA_TYPE_HRV);
    let mut grouped: HashMap<HrvMethod, Vec<&Observation>> = HashMap::new();
    for sample in &samples {
        if let Some(method) = sample_method(sample) {
            grouped.entry(method).or_default().push(*sample);
        }
    }

    let mut daily: HashMap<(HrvMethod, i64), (f64, Vec<&Observation>)> = HashMap::new();
    let mut stats: HashMap<HrvMethod, Vec<DayStat>> = HashMap::new();
    for (method, method_samples) in &grouped {
        let days = by_utc_day(method_samples);
        let mut method_stats = Vec::new();
        for (day, day_samples) in days {
            if let Some(ms) = mean_method(&day_samples, *method) {
                daily.insert((*method, day), (ms, day_samples));
                method_stats.push(DayStat { day, value: ms });
            }
        }
        stats.insert(*method, method_stats);
    }

    let mut day_keys: BTreeSet<i64> = BTreeSet::new();
    for ((_method, day), _) in &daily {
        day_keys.insert(*day);
    }

    let mut features = Vec::new();
    for day in day_keys {
        let mut best: Option<(i64, HrvMethod)> = None;
        for method in [HrvMethod::Sdnn, HrvMethod::Rmssd] {
            let Some(method_stats) = stats.get(&method) else {
                continue;
            };
            if baseline_median(method_stats, day, HRV_LOOKBACK_DAYS).is_none() {
                continue;
            }
            let Some((_, day_samples)) = daily.get(&(method, day)) else {
                continue;
            };
            let ts = day_samples
                .last()
                .map(|o| o.timestamp.as_secs())
                .unwrap_or(day);
            let replace = best.map(|(prev, _)| ts >= prev).unwrap_or(true);
            if replace {
                best = Some((ts, method));
            }
        }
        let Some((_, method)) = best else {
            continue;
        };
        let Some(method_stats) = stats.get(&method) else {
            continue;
        };
        let Some((median, n)) = baseline_median(method_stats, day, HRV_LOOKBACK_DAYS) else {
            continue;
        };
        let Some((ms, day_samples)) = daily.get(&(method, day)) else {
            continue;
        };
        let end = day_samples
            .last()
            .map(|o| o.timestamp.as_secs())
            .unwrap_or(day);
        let mut value = serde_json::Map::new();
        value.insert("method".to_owned(), serde_json::json!(method.as_str()));
        value.insert("ms".to_owned(), serde_json::json!(*ms));
        value.insert("baseline_ms".to_owned(), serde_json::json!(median));
        value.insert("delta_ms".to_owned(), serde_json::json!(*ms - median));
        value.insert("baseline_days".to_owned(), serde_json::json!(n));
        features.push(emit(
            HRV_VS_BASELINE_ID,
            day,
            end,
            serde_json::Value::Object(value),
            day_samples,
        ));
    }
    features
}

fn sleep_features(observations: &[Observation]) -> Vec<Feature> {
    let samples = of_type(observations, DATA_TYPE_SLEEP_INTERVAL);
    let mut by_day: BTreeMap<i64, Vec<&Observation>> = BTreeMap::new();
    for sample in &samples {
        let Some((_, end)) = interval_bounds(sample) else {
            continue;
        };
        by_day.entry(utc_day(end)).or_default().push(*sample);
    }

    let bedtimes: Vec<(i64, f64)> = by_day
        .iter()
        .filter_map(|(day, day_samples)| {
            earliest_start(day_samples).map(|start| (*day, hour_of(start)))
        })
        .collect();

    let mut features = Vec::new();
    for (day, day_samples) in &by_day {
        let mut unstaged = Vec::new();
        let mut core = Vec::new();
        let mut deep = Vec::new();
        let mut rem = Vec::new();
        for sample in day_samples {
            let Some((start, end)) = interval_bounds(sample) else {
                continue;
            };
            match sample.payload.get("stage").and_then(|v| v.as_str()) {
                Some("asleep_core") => core.push((start, end)),
                Some("asleep_deep") => deep.push((start, end)),
                Some("asleep_rem") => rem.push((start, end)),
                Some("asleep") | None => unstaged.push((start, end)),
                _ => {}
            }
        }
        let core_secs = merged_secs(&mut core);
        let deep_secs = merged_secs(&mut deep);
        let rem_secs = merged_secs(&mut rem);
        let unstaged_secs = merged_secs(&mut unstaged);
        let staged = core_secs + deep_secs + rem_secs;
        let total = staged + unstaged_secs;
        if total <= 0.0 {
            continue;
        }
        let mut value = serde_json::Map::new();
        value.insert("total_min".to_owned(), serde_json::json!(total / 60.0));
        if staged > 0.0 {
            value.insert("stages".to_owned(), serde_json::json!(true));
            value.insert(
                "deep_share".to_owned(),
                serde_json::json!(deep_secs / staged),
            );
            value.insert("rem_share".to_owned(), serde_json::json!(rem_secs / staged));
            value.insert(
                "core_share".to_owned(),
                serde_json::json!(core_secs / staged),
            );
        } else {
            value.insert("stages".to_owned(), serde_json::json!(false));
        }
        if let Some(regularity) = bedtime_regularity(&bedtimes, *day) {
            value.insert(
                "bedtime_regularity".to_owned(),
                serde_json::json!(regularity),
            );
        }
        let end = day_samples
            .iter()
            .filter_map(|o| interval_bounds(o).map(|(_, e)| e))
            .max()
            .unwrap_or(*day);
        features.push(emit(
            SLEEP_STAGES_ID,
            *day,
            end,
            serde_json::Value::Object(value),
            day_samples,
        ));
    }
    features
}

fn nightly_features(
    observations: &[Observation],
    data_type: &str,
    field: &str,
    feature_id: &str,
    mean_key: &str,
    min_key: Option<&str>,
    baseline_key: &str,
    delta_key: &str,
) -> Vec<Feature> {
    let samples = of_type(observations, data_type);
    let sleep = of_type(observations, DATA_TYPE_SLEEP_INTERVAL);
    let nights = nights_for(&samples, &sleep);

    let stats: Vec<DayStat> = nights
        .iter()
        .filter_map(|(day, day_samples)| {
            mean_min(day_samples, field).map(|(mean, _)| DayStat {
                day: *day,
                value: mean,
            })
        })
        .collect();

    let mut features = Vec::new();
    for (day, day_samples) in &nights {
        let Some((mean, min)) = mean_min(day_samples, field) else {
            continue;
        };
        let mut value = serde_json::Map::new();
        value.insert(mean_key.to_owned(), serde_json::json!(mean));
        if let Some(min_key) = min_key {
            value.insert(min_key.to_owned(), serde_json::json!(min));
        }
        if let Some((median, n)) = baseline_median(&stats, *day, NIGHT_LOOKBACK_DAYS) {
            value.insert(baseline_key.to_owned(), serde_json::json!(median));
            value.insert(delta_key.to_owned(), serde_json::json!(mean - median));
            value.insert("baseline_days".to_owned(), serde_json::json!(n));
        }
        let end = day_samples
            .last()
            .map(|o| o.timestamp.as_secs())
            .unwrap_or(*day);
        features.push(emit(
            feature_id,
            *day,
            end,
            serde_json::Value::Object(value),
            day_samples,
        ));
    }
    features
}

fn nights_for<'a>(
    samples: &[&'a Observation],
    sleep: &[&'a Observation],
) -> BTreeMap<i64, Vec<&'a Observation>> {
    let mut claimed: std::collections::HashSet<bio_spec::ObservationId> =
        std::collections::HashSet::new();
    let mut nights: BTreeMap<i64, Vec<&Observation>> = BTreeMap::new();
    let mut wake_days: BTreeSet<i64> = BTreeSet::new();
    for sample in sleep {
        if let Some((_, end)) = interval_bounds(sample) {
            wake_days.insert(utc_day(end));
        }
    }
    for day in wake_days {
        let Some((start, end)) = night_bounds(sleep, day) else {
            continue;
        };
        let inside: Vec<&Observation> = samples
            .iter()
            .copied()
            .filter(|o| {
                let t = o.timestamp.as_secs();
                t >= start && t <= end
            })
            .collect();
        for sample in &inside {
            claimed.insert(sample.id);
        }
        if !inside.is_empty() {
            nights.insert(day, inside);
        }
    }
    let mut leftovers: BTreeMap<i64, Vec<&Observation>> = BTreeMap::new();
    for sample in samples {
        if claimed.contains(&sample.id) {
            continue;
        }
        leftovers
            .entry(utc_day(sample.timestamp.as_secs()))
            .or_default()
            .push(*sample);
    }
    for (day, day_samples) in leftovers {
        nights.entry(day).or_insert(day_samples);
    }
    nights
}

fn night_bounds(sleep: &[&Observation], day: i64) -> Option<(i64, i64)> {
    let mut start = i64::MAX;
    let mut end = i64::MIN;
    let mut any = false;
    for sample in sleep {
        let Some((s, e)) = interval_bounds(sample) else {
            continue;
        };
        if utc_day(e) != day {
            continue;
        }
        any = true;
        start = start.min(s);
        end = end.max(e);
    }
    if any {
        Some((start, end))
    } else {
        None
    }
}

fn of_type<'a>(observations: &'a [Observation], data_type: &str) -> Vec<&'a Observation> {
    observations
        .iter()
        .filter(|o| o.data_type == data_type)
        .collect()
}

fn by_utc_day<'a>(samples: &[&'a Observation]) -> BTreeMap<i64, Vec<&'a Observation>> {
    let mut days: BTreeMap<i64, Vec<&Observation>> = BTreeMap::new();
    for sample in samples {
        days.entry(utc_day(sample.timestamp.as_secs()))
            .or_default()
            .push(*sample);
    }
    for day_samples in days.values_mut() {
        day_samples.sort_by_key(|o| o.timestamp.as_secs());
    }
    days
}

fn baseline_median(stats: &[DayStat], day: i64, lookback_days: i64) -> Option<(f64, usize)> {
    let start = day - lookback_days * SECS_PER_DAY;
    let mut values: Vec<f64> = stats
        .iter()
        .filter(|stat| stat.day >= start && stat.day < day)
        .map(|stat| stat.value)
        .filter(|v| v.is_finite())
        .collect();
    if values.len() < BASELINE_MIN_DAYS {
        return None;
    }
    let n = values.len();
    median(&mut values).map(|m| (m, n))
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    if n % 2 == 1 {
        Some(values[n / 2])
    } else {
        Some((values[n / 2 - 1] + values[n / 2]) / 2.0)
    }
}

fn last_f64(samples: &[&Observation], key: &str) -> Option<f64> {
    samples.iter().rev().find_map(|o| finite_field(o, key))
}

fn mean_min(samples: &[&Observation], key: &str) -> Option<(f64, f64)> {
    let values: Vec<f64> = samples
        .iter()
        .filter_map(|o| finite_field(o, key))
        .collect();
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    Some((mean, min))
}

fn finite_field(obs: &Observation, key: &str) -> Option<f64> {
    obs.payload
        .get(key)
        .and_then(|v| v.as_f64())
        .filter(|n| n.is_finite())
}

fn interval_bounds(obs: &Observation) -> Option<(i64, i64)> {
    let start = json_i64(obs.payload.get("start")?)?;
    let end = json_i64(obs.payload.get("end")?)?;
    if end < start {
        return None;
    }
    Some((start, end))
}

fn json_i64(v: &serde_json::Value) -> Option<i64> {
    v.as_i64().or_else(|| {
        v.as_f64()
            .filter(|f| f.is_finite() && f.fract() == 0.0)
            .map(|f| f as i64)
    })
}

fn earliest_start(samples: &[&Observation]) -> Option<i64> {
    samples
        .iter()
        .filter_map(|o| interval_bounds(o).map(|(s, _)| s))
        .min()
}

fn hour_of(ts: i64) -> f64 {
    let day = utc_day(ts);
    (ts - day) as f64 / 3_600.0
}

fn bedtime_regularity(bedtimes: &[(i64, f64)], day: i64) -> Option<f64> {
    let start = day - NIGHT_LOOKBACK_DAYS * SECS_PER_DAY;
    let hours: Vec<f64> = bedtimes
        .iter()
        .filter(|(d, _)| *d >= start && *d < day)
        .map(|(_, hour)| *hour)
        .collect();
    if hours.len() < BEDTIME_MIN_NIGHTS {
        return None;
    }
    let mean = hours.iter().sum::<f64>() / hours.len() as f64;
    let var = hours.iter().map(|h| (h - mean).powi(2)).sum::<f64>() / hours.len() as f64;
    let score = 100.0 * (1.0 - (var.sqrt() / 3.0)).clamp(0.0, 1.0);
    Some(score)
}

fn merged_secs(spans: &mut [(i64, i64)]) -> f64 {
    if spans.is_empty() {
        return 0.0;
    }
    spans.sort_by_key(|(start, _)| *start);
    let mut total = 0.0;
    let mut cur_s = spans[0].0;
    let mut cur_e = spans[0].1;
    for (start, end) in spans.iter().skip(1) {
        if *start <= cur_e {
            cur_e = cur_e.max(*end);
        } else {
            total += (cur_e - cur_s) as f64;
            cur_s = *start;
            cur_e = *end;
        }
    }
    total + (cur_e - cur_s) as f64
}

fn utc_day(ts: i64) -> i64 {
    ts.div_euclid(SECS_PER_DAY) * SECS_PER_DAY
}

fn emit(
    feature_id: &str,
    start: i64,
    end: i64,
    value: serde_json::Value,
    evidence: &[&Observation],
) -> Feature {
    let end = end.max(start);
    let base = single_family_confidence(evidence).get();
    let confidence = Confidence::saturating_from(base * mean_source_quality(evidence));
    Feature {
        feature_id: feature_id.to_owned(),
        time_window: TimeWindow {
            start: UnixTimestamp::from_secs(start),
            end: UnixTimestamp::from_secs(end),
        },
        value: FeatureValue::Object(value),
        provenance: evidence.iter().map(|o| o.id).collect(),
        confidence,
        factors: source_factor(evidence),
    }
}

fn source_kind(obs: &Observation) -> Option<&str> {
    obs.payload
        .get("src")
        .and_then(|src| src.get("kind"))
        .and_then(|kind| kind.as_str())
        .filter(|kind| !kind.is_empty())
}

fn source_quality(kind: &str) -> f64 {
    match kind {
        "apple_watch" => 1.0,
        "manual" => 0.3,
        _ => 0.6,
    }
}

fn mean_source_quality(evidence: &[&Observation]) -> f64 {
    if evidence.is_empty() {
        return 0.0;
    }
    let sum: f64 = evidence
        .iter()
        .map(|o| source_kind(o).map(source_quality).unwrap_or(0.6))
        .sum();
    sum / evidence.len() as f64
}

fn source_label(kind: &str) -> &'static str {
    match kind {
        "apple_watch" => "Apple Watch",
        "xiaomi_mi_fitness" => "Mi Fitness",
        "zepp_life" => "Zepp Life",
        "iphone" => "iPhone",
        "manual" => "Manual entry",
        "other_app" => "Other app",
        _ => "Source",
    }
}

fn source_factor(evidence: &[&Observation]) -> Vec<ExplanationFactor> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for obs in evidence {
        if let Some(kind) = source_kind(obs) {
            *counts.entry(kind).or_default() += 1;
        }
    }
    let Some((kind, _)) = counts.into_iter().max_by_key(|(_, n)| *n) else {
        return Vec::new();
    };
    vec![ExplanationFactor {
        id: "source".to_owned(),
        label: source_label(kind).to_owned(),
        share: 1.0,
    }]
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::{json, Value};
    use uuid::Uuid;

    use super::*;
    use crate::FeatureEngine;

    fn day(offset: i64) -> i64 {
        let base = 1_700_000_000i64.div_euclid(SECS_PER_DAY) * SECS_PER_DAY;
        base + offset * SECS_PER_DAY
    }

    fn obs(id: u128, ts: i64, data_type: &str, payload: Value) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            data_type,
            payload,
            1.0,
        )
        .expect("obs")
    }

    fn run(batch: Vec<Observation>) -> Vec<Feature> {
        let mut engine = FeatureEngine::new();
        engine.register(WearableVitalsNode::new()).expect("reg");
        engine.run(&batch).expect("run").features
    }

    fn object<'a>(features: &'a [Feature], id: &str) -> &'a serde_json::Map<String, Value> {
        let feature = features
            .iter()
            .filter(|f| f.feature_id == id)
            .next_back()
            .unwrap_or_else(|| panic!("missing {id}"));
        match &feature.value {
            FeatureValue::Object(Value::Object(map)) => map,
            other => panic!("expected object, got {other:?}"),
        }
    }

    fn num(map: &serde_json::Map<String, Value>, key: &str) -> f64 {
        map.get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("missing {key}"))
    }

    #[test]
    fn resting_heart_rate_uses_personal_median() {
        let mut batch = Vec::new();
        for offset in 5..10 {
            batch.push(obs(
                offset as u128,
                day(offset) + 8 * 3_600,
                DATA_TYPE_RESTING_HEART_RATE,
                json!({ "bpm": 60.0, "src": { "kind": "apple_watch" } }),
            ));
        }
        batch.push(obs(
            10,
            day(10) + 8 * 3_600,
            DATA_TYPE_RESTING_HEART_RATE,
            json!({ "bpm": 54.0, "src": { "kind": "apple_watch" } }),
        ));
        let features = run(batch);
        let map = object(&features, RESTING_HEART_RATE_ID);
        assert!((num(map, "bpm") - 54.0).abs() < 1e-9);
        assert!((num(map, "baseline_bpm") - 60.0).abs() < 1e-9);
        assert!((num(map, "delta_bpm") + 6.0).abs() < 1e-9);
        assert_eq!(num(map, "baseline_days") as usize, 5);
    }

    #[test]
    fn resting_heart_rate_without_baseline_still_shows_bpm() {
        let features = run(vec![obs(
            1,
            day(3) + 100,
            DATA_TYPE_RESTING_HEART_RATE,
            json!({ "bpm": 58.0, "src": { "kind": "apple_watch" } }),
        )]);
        let map = object(&features, RESTING_HEART_RATE_ID);
        assert!((num(map, "bpm") - 58.0).abs() < 1e-9);
        assert!(!map.contains_key("delta_bpm"));
    }

    #[test]
    fn hrv_omits_until_five_baseline_days() {
        let mut batch = Vec::new();
        for offset in 6..10 {
            batch.push(obs(
                offset as u128,
                day(offset) + 100,
                DATA_TYPE_HRV,
                json!({ "method": "sdnn", "sdnn_ms": 50.0 }),
            ));
        }
        let features = run(batch);
        assert!(
            features.iter().all(|f| f.feature_id != HRV_VS_BASELINE_ID),
            "four earlier days must omit HRV, not emit 0"
        );
    }

    #[test]
    fn hrv_sdnn_delta_and_methods_do_not_mix() {
        let mut batch = Vec::new();
        for offset in 5..10 {
            let ts = day(offset) + 12 * 3_600;
            batch.push(obs(
                offset as u128,
                ts,
                DATA_TYPE_HRV,
                json!({ "method": "sdnn", "sdnn_ms": 40.0, "src": { "kind": "apple_watch" } }),
            ));
            batch.push(obs(
                100 + offset as u128,
                ts + 60,
                DATA_TYPE_HRV,
                json!({ "method": "rmssd", "rmssd_ms": 80.0, "src": { "kind": "apple_watch" } }),
            ));
        }
        let today = day(10) + 12 * 3_600;
        batch.push(obs(
            10,
            today,
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": 40.0, "src": { "kind": "apple_watch" } }),
        ));
        batch.push(obs(
            110,
            today + 60,
            DATA_TYPE_HRV,
            json!({ "method": "rmssd", "rmssd_ms": 80.0, "src": { "kind": "apple_watch" } }),
        ));
        let features = run(batch);
        let latest = features
            .iter()
            .filter(|f| f.feature_id == HRV_VS_BASELINE_ID)
            .next_back()
            .expect("hrv");
        let map = match &latest.value {
            FeatureValue::Object(Value::Object(map)) => map,
            other => panic!("{other:?}"),
        };
        assert_eq!(map["method"], json!("rmssd"));
        assert!(
            (num(map, "ms") - 80.0).abs() < 1e-9,
            "must not average SDNN with RMSSD"
        );
        assert!((num(map, "delta_ms") - 0.0).abs() < 1e-9);
        assert!(features
            .iter()
            .filter(|f| f.feature_id == HRV_VS_BASELINE_ID)
            .all(|f| {
                match &f.value {
                    FeatureValue::Object(Value::Object(m)) => {
                        m["method"] == json!("rmssd") || m["method"] == json!("sdnn")
                    }
                    _ => false,
                }
            }));
    }

    #[test]
    fn unlabeled_both_fields_are_skipped() {
        let mut batch = Vec::new();
        for offset in 5..=10 {
            batch.push(obs(
                offset as u128,
                day(offset) + 100,
                DATA_TYPE_HRV,
                json!({ "rmssd_ms": 30.0, "sdnn_ms": 90.0 }),
            ));
        }
        let features = run(batch);
        assert!(features.iter().all(|f| f.feature_id != HRV_VS_BASELINE_ID));
    }

    #[test]
    fn sleep_stages_report_shares_or_total_only() {
        let wake = day(10) + 7 * 3_600;
        let mut batch = Vec::new();
        for offset in 7..10 {
            let start = day(offset) + 3 * 3_600;
            batch.push(obs(
                offset as u128,
                start + 3_600,
                DATA_TYPE_SLEEP_INTERVAL,
                json!({
                    "start": start,
                    "end": start + 3_600,
                    "stage": "asleep",
                    "src": { "kind": "apple_watch" }
                }),
            ));
        }
        batch.push(obs(
            21,
            wake - 2 * 3_600,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({ "start": wake - 4 * 3_600, "end": wake - 2 * 3_600, "stage": "asleep_core", "src": { "kind": "apple_watch" } }),
        ));
        batch.push(obs(
            22,
            wake - 3_600,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({ "start": wake - 2 * 3_600, "end": wake - 3_600, "stage": "asleep_deep", "src": { "kind": "apple_watch" } }),
        ));
        batch.push(obs(
            23,
            wake,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({ "start": wake - 3_600, "end": wake, "stage": "asleep_rem", "src": { "kind": "apple_watch" } }),
        ));
        let features = run(batch);
        let staged = features
            .iter()
            .filter(|f| f.feature_id == SLEEP_STAGES_ID)
            .next_back()
            .expect("stages");
        let map = match &staged.value {
            FeatureValue::Object(Value::Object(map)) => map,
            other => panic!("{other:?}"),
        };
        assert_eq!(map["stages"], json!(true));
        assert!((num(map, "total_min") - 240.0).abs() < 1e-6);
        assert!((num(map, "core_share") - 0.5).abs() < 1e-9);
        assert!((num(map, "deep_share") - 0.25).abs() < 1e-9);
        assert!((num(map, "rem_share") - 0.25).abs() < 1e-9);
        assert!((num(map, "bedtime_regularity") - 100.0).abs() < 1e-6);

        let total_only = run(vec![obs(
            1,
            day(4) + 8 * 3_600,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({
                "start": day(4),
                "end": day(4) + 7 * 3_600,
                "stage": "asleep",
                "src": { "kind": "xiaomi_mi_fitness" }
            }),
        )]);
        let plain = object(&total_only, SLEEP_STAGES_ID);
        assert_eq!(plain["stages"], json!(false));
        assert!((num(plain, "total_min") - 420.0).abs() < 1e-6);
        assert!(!plain.contains_key("deep_share"));
    }

    #[test]
    fn night_spo2_uses_sleep_window_mean_and_min() {
        let start = day(10) + 3_600;
        let end = day(10) + 8 * 3_600;
        let features = run(vec![
            obs(
                1,
                end,
                DATA_TYPE_SLEEP_INTERVAL,
                json!({ "start": start, "end": end, "stage": "asleep", "src": { "kind": "apple_watch" } }),
            ),
            obs(
                2,
                start + 3_600,
                DATA_TYPE_OXYGEN_SATURATION,
                json!({ "spo2_percent": 98.0, "src": { "kind": "apple_watch" } }),
            ),
            obs(
                3,
                start + 2 * 3_600,
                DATA_TYPE_OXYGEN_SATURATION,
                json!({ "spo2_percent": 94.0, "src": { "kind": "apple_watch" } }),
            ),
            obs(
                4,
                start + 3 * 3_600,
                DATA_TYPE_OXYGEN_SATURATION,
                json!({ "spo2_percent": 96.0, "src": { "kind": "apple_watch" } }),
            ),
            obs(
                5,
                day(10) + 20 * 3_600,
                DATA_TYPE_OXYGEN_SATURATION,
                json!({ "spo2_percent": 80.0, "src": { "kind": "apple_watch" } }),
            ),
        ]);
        let map = object(&features, NIGHT_SPO2_ID);
        assert!((num(map, "mean_percent") - 96.0).abs() < 1e-9);
        assert!((num(map, "min_percent") - 94.0).abs() < 1e-9);
    }

    #[test]
    fn respiratory_rate_reports_mean_and_min() {
        let ts = day(2) + 4 * 3_600;
        let features = run(vec![
            obs(
                1,
                ts,
                DATA_TYPE_RESPIRATORY_RATE,
                json!({ "breaths_per_min": 16.0, "src": { "kind": "apple_watch" } }),
            ),
            obs(
                2,
                ts + 60,
                DATA_TYPE_RESPIRATORY_RATE,
                json!({ "breaths_per_min": 12.0, "src": { "kind": "apple_watch" } }),
            ),
            obs(
                3,
                ts + 120,
                DATA_TYPE_RESPIRATORY_RATE,
                json!({ "breaths_per_min": 14.0, "src": { "kind": "apple_watch" } }),
            ),
        ]);
        let map = object(&features, RESPIRATORY_RATE_ID);
        assert!((num(map, "mean_per_min") - 14.0).abs() < 1e-9);
        assert!((num(map, "min_per_min") - 12.0).abs() < 1e-9);
    }

    #[test]
    fn wrist_temperature_delta_needs_personal_days() {
        let mut batch = Vec::new();
        for offset in 5..10 {
            batch.push(obs(
                offset as u128,
                day(offset) + 5 * 3_600,
                DATA_TYPE_SLEEPING_WRIST_TEMPERATURE,
                json!({ "celsius": 0.0, "src": { "kind": "apple_watch" } }),
            ));
        }
        batch.push(obs(
            10,
            day(10) + 5 * 3_600,
            DATA_TYPE_SLEEPING_WRIST_TEMPERATURE,
            json!({ "celsius": 0.4, "src": { "kind": "apple_watch" } }),
        ));
        let features = run(batch);
        let map = object(&features, WRIST_TEMPERATURE_ID);
        assert!((num(map, "celsius") - 0.4).abs() < 1e-9);
        assert!((num(map, "delta_celsius") - 0.4).abs() < 1e-9);
    }

    #[test]
    fn source_quality_scales_confidence() {
        let watch = run(vec![obs(
            1,
            day(1) + 100,
            DATA_TYPE_RESTING_HEART_RATE,
            json!({ "bpm": 60.0, "src": { "kind": "apple_watch" } }),
        )]);
        let manual = run(vec![obs(
            1,
            day(1) + 100,
            DATA_TYPE_RESTING_HEART_RATE,
            json!({ "bpm": 60.0, "src": { "kind": "manual" } }),
        )]);
        let watch_c = watch
            .iter()
            .find(|f| f.feature_id == RESTING_HEART_RATE_ID)
            .expect("watch")
            .confidence
            .get();
        let manual_c = manual
            .iter()
            .find(|f| f.feature_id == RESTING_HEART_RATE_ID)
            .expect("manual")
            .confidence
            .get();
        assert!((watch_c - 1.0).abs() < 1e-9, "{watch_c}");
        assert!((manual_c - 0.3).abs() < 1e-9, "{manual_c}");
        let label = &watch
            .iter()
            .find(|f| f.feature_id == RESTING_HEART_RATE_ID)
            .expect("watch")
            .factors[0]
            .label;
        assert_eq!(label, "Apple Watch");
        for word in [
            "apnea",
            "arrhythmia",
            "disease",
            "illness",
            "burnout",
            "stress",
        ] {
            assert!(!label.to_lowercase().contains(word));
        }
    }
}
