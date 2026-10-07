//! Chart Feature series host (ADR-018 / P17-E3-T1).
//!
//! Recompute-on-read: load Observations for the selected span, run
//! `FeatureEngine` with the range's default `stepSecs`, optional in-process
//! memo. Soft-fails to an empty series. No Feature-history SQLite. UI ↛ DB.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use bio_spec::{Feature, Observation, UnixTimestamp};
use feature_engine::{register_catalog_v1, FeatureEngine};
use storage::{Database, ObservationRepository};
use tracing::warn;

/// Closed-set chart ranges (ADR-018).
pub const CHART_RANGES: &[&str] = &["1h", "8h", "12h", "1d", "1w"];

/// Process-local memo TTL for series recompute (session cache only).
const SERIES_MEMO_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartRangeSpec {
    pub range: &'static str,
    pub span_secs: i64,
    pub step_secs: i64,
}

/// Resolve a closed-set range string to span + default step (ADR-018 table).
#[must_use]
pub fn parse_chart_range(range: &str) -> Option<ChartRangeSpec> {
    match range {
        "1h" => Some(ChartRangeSpec {
            range: "1h",
            span_secs: 3600,
            step_secs: 60,
        }),
        "8h" => Some(ChartRangeSpec {
            range: "8h",
            span_secs: 8 * 3600,
            step_secs: 300,
        }),
        "12h" => Some(ChartRangeSpec {
            range: "12h",
            span_secs: 12 * 3600,
            step_secs: 300,
        }),
        "1d" => Some(ChartRangeSpec {
            range: "1d",
            span_secs: 24 * 3600,
            step_secs: 900,
        }),
        "1w" => Some(ChartRangeSpec {
            range: "1w",
            span_secs: 7 * 24 * 3600,
            step_secs: 3600,
        }),
        _ => None,
    }
}

#[derive(Debug, Default)]
struct SeriesMemoEntry {
    key: String,
    watermark: i64,
    features: Vec<Feature>,
    computed_at: Option<Instant>,
}

/// In-process memo for recomputed Feature series (not persisted).
#[derive(Debug, Default)]
pub struct SeriesMemoState {
    inner: Mutex<SeriesMemoEntry>,
}

impl SeriesMemoState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn get_if_fresh(&self, key: &str, watermark: i64) -> Option<Vec<Feature>> {
        let guard = self.inner.lock().ok()?;
        let computed_at = guard.computed_at?;
        if guard.key != key || guard.watermark != watermark {
            return None;
        }
        if computed_at.elapsed() > SERIES_MEMO_TTL {
            return None;
        }
        Some(guard.features.clone())
    }

    fn store(&self, key: String, watermark: i64, features: Vec<Feature>) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = SeriesMemoEntry {
                key,
                watermark,
                features,
                computed_at: Some(Instant::now()),
            };
        }
    }
}

/// Result of a series recompute (host → IPC DTO mapping).
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureSeriesResult {
    pub range: String,
    pub step_secs: i64,
    pub window_start: i64,
    pub window_end: i64,
    pub features: Vec<Feature>,
}

impl FeatureSeriesResult {
    #[must_use]
    pub fn empty(range: &str, step_secs: i64, window_start: i64, window_end: i64) -> Self {
        Self {
            range: range.to_owned(),
            step_secs,
            window_start,
            window_end,
            features: Vec::new(),
        }
    }
}

/// Load Observations for `range`, run catalog FeatureEngine with the range step.
///
/// Soft-fails (empty features) on unknown range / DB / engine errors.
/// Optional `feature_ids` filters the emitted series (empty filter = all).
pub fn load_feature_series(
    memo: Option<&SeriesMemoState>,
    range: &str,
    feature_ids: Option<&[String]>,
    reference_ts: i64,
) -> FeatureSeriesResult {
    let Some(spec) = parse_chart_range(range) else {
        return FeatureSeriesResult::empty(range, 60, reference_ts, reference_ts);
    };

    let window_end = reference_ts;
    let window_start = reference_ts.saturating_sub(spec.span_secs);

    let observations = match load_span_observations(window_start, window_end) {
        Ok(obs) => obs,
        Err(err) => {
            warn!(error = %err, "series: observation load failed; empty series");
            return FeatureSeriesResult::empty(spec.range, spec.step_secs, window_start, window_end);
        }
    };

    let watermark = observations
        .iter()
        .map(|o| o.timestamp.as_secs())
        .max()
        .unwrap_or(0);

    let filter_key = feature_ids
        .map(|ids| {
            let mut sorted = ids.to_vec();
            sorted.sort();
            sorted.join(",")
        })
        .unwrap_or_default();
    // Row count in the key: retracting an older Life Event leaves the max
    // timestamp unchanged but must still invalidate the memo.
    let memo_key = format!(
        "{}|{}|{}|{}",
        spec.range,
        spec.step_secs,
        filter_key,
        observations.len()
    );

    if let Some(memo) = memo {
        if let Some(cached) = memo.get_if_fresh(&memo_key, watermark) {
            return FeatureSeriesResult {
                range: spec.range.to_owned(),
                step_secs: spec.step_secs,
                window_start,
                window_end,
                features: cached,
            };
        }
    }

    let features = match recompute_series(&observations, spec.step_secs, feature_ids) {
        Ok(features) => {
            if let Some(memo) = memo {
                memo.store(memo_key, watermark, features.clone());
            }
            features
        }
        Err(err) => {
            warn!(error = %err, "series: FeatureEngine failed; empty series");
            Vec::new()
        }
    };

    FeatureSeriesResult {
        range: spec.range.to_owned(),
        step_secs: spec.step_secs,
        window_start,
        window_end,
        features,
    }
}

fn recompute_series(
    observations: &[Observation],
    step_secs: i64,
    feature_ids: Option<&[String]>,
) -> Result<Vec<Feature>, String> {
    if observations.is_empty() {
        return Ok(Vec::new());
    }
    let mut engine = FeatureEngine::new();
    register_catalog_v1(&mut engine).map_err(|err| err.to_string())?;
    let output = engine
        .run_with_step(observations, step_secs)
        .map_err(|err| err.to_string())?;
    let features = match feature_ids {
        None | Some([]) => output.features,
        Some(ids) => output
            .features
            .into_iter()
            .filter(|f| ids.iter().any(|id| id == &f.feature_id))
            .collect(),
    };
    Ok(features)
}

fn load_span_observations(start: i64, end: i64) -> Result<Vec<Observation>, String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    repo.list_by_time_range(UnixTimestamp::from_secs(start), UnixTimestamp::from_secs(end))
        .map_err(|err| err.public_message())
}

/// Keep the latest Feature per `feature_id` (by `time_window.end`, then last wins).
#[must_use]
pub fn latest_features_per_id(features: &[Feature]) -> Vec<Feature> {
    let mut best: Vec<Feature> = Vec::new();
    for feature in features {
        if let Some(existing) = best
            .iter_mut()
            .find(|f| f.feature_id == feature.feature_id)
        {
            if feature.time_window.end.as_secs() >= existing.time_window.end.as_secs() {
                *existing = feature.clone();
            }
        } else {
            best.push(feature.clone());
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use bio_spec::{Confidence, FeatureValue, TimeWindow};
    use uuid::Uuid;

    #[test]
    fn parse_all_closed_set_ranges() {
        for range in CHART_RANGES {
            let spec = parse_chart_range(range).expect("range");
            assert_eq!(spec.range, *range);
            assert!(spec.span_secs > 0);
            assert!(spec.step_secs > 0);
        }
        assert!(parse_chart_range("2d").is_none());
    }

    #[test]
    fn default_steps_match_adr_018() {
        assert_eq!(parse_chart_range("1h").unwrap().step_secs, 60);
        assert_eq!(parse_chart_range("8h").unwrap().step_secs, 300);
        assert_eq!(parse_chart_range("12h").unwrap().step_secs, 300);
        assert_eq!(parse_chart_range("1d").unwrap().step_secs, 900);
        assert_eq!(parse_chart_range("1w").unwrap().step_secs, 3600);
    }

    #[test]
    fn memo_misses_on_key_or_watermark() {
        let memo = SeriesMemoState::new();
        memo.store("1h|60|".into(), 100, Vec::new());
        assert!(memo.get_if_fresh("1h|60|", 100).is_some());
        assert!(memo.get_if_fresh("1h|60|", 101).is_none());
        assert!(memo.get_if_fresh("1d|900|", 100).is_none());
    }

    #[test]
    fn latest_per_id_keeps_newest_end() {
        let window = |end| {
            TimeWindow::try_new(UnixTimestamp::from_secs(end - 900), UnixTimestamp::from_secs(end))
                .expect("tw")
        };
        let a = Feature {
            feature_id: "FocusScore".into(),
            time_window: window(1000),
            value: FeatureValue::Scalar(10.0),
            provenance: vec![Uuid::from_u128(1)],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        };
        let b = Feature {
            feature_id: "FocusScore".into(),
            time_window: window(2000),
            value: FeatureValue::Scalar(90.0),
            provenance: vec![Uuid::from_u128(2)],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        };
        let c = Feature {
            feature_id: "StressIndex".into(),
            time_window: window(1500),
            value: FeatureValue::Scalar(40.0),
            provenance: vec![Uuid::from_u128(3)],
            confidence: Confidence::ONE,
            factors: Vec::new(),
        };
        let latest = latest_features_per_id(&[a, b, c]);
        assert_eq!(latest.len(), 2);
        let focus = latest.iter().find(|f| f.feature_id == "FocusScore").unwrap();
        assert_eq!(focus.time_window.end.as_secs(), 2000);
        match &focus.value {
            FeatureValue::Scalar(v) => assert!((*v - 90.0).abs() < 0.01),
            _ => panic!("scalar"),
        }
    }

    #[test]
    fn recompute_empty_observations_is_empty() {
        let features = recompute_series(&[], 60, None).expect("ok");
        assert!(features.is_empty());
    }

    #[test]
    fn recompute_synthetic_steps_emits_activity_balance() {
        use bio_spec::{Observation, DATA_TYPE_STEP_COUNT};
        use serde_json::json;
        let obs = Observation::try_new(
            Uuid::from_u128(42),
            UnixTimestamp::from_secs(3_600),
            "test.provider",
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 800 }),
            1.0,
        )
        .expect("obs");
        let features = recompute_series(&[obs], 300, Some(&["ActivityBalance".into()])).expect("ok");
        assert!(
            features.iter().any(|f| f.feature_id == "ActivityBalance"),
            "expected ActivityBalance in synthetic series"
        );
        assert!(features.iter().all(|f| f.feature_id == "ActivityBalance"));
    }

    #[test]
    fn retracted_life_event_does_not_reach_feature_series() {
        use bio_spec::{Observation, DATA_TYPE_LIFE_EVENT, DATA_TYPE_LIFE_EVENT_RETRACTION, DATA_TYPE_STEP_COUNT};
        use serde_json::json;
        use storage::{Database, ObservationRepository};

        let db = Database::open_in_memory().expect("db");
        let repo = ObservationRepository::new(&db);
        let steps = Observation::try_new(
            Uuid::from_u128(1),
            UnixTimestamp::from_secs(3_000),
            "test.provider",
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 200 }),
            1.0,
        )
        .expect("steps");
        let workout = Observation::try_new(
            Uuid::from_u128(2),
            UnixTimestamp::from_secs(3_300),
            "com.biofocus.desktop",
            DATA_TYPE_LIFE_EVENT,
            json!({ "kind": "workout", "duration_secs": 1800 }),
            1.0,
        )
        .expect("workout");
        repo.insert(&steps).expect("i1");
        repo.insert(&workout).expect("i2");
        let with_workout = recompute_series(
            &repo.list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(4_000)).expect("q"),
            300,
            None,
        )
        .expect("series");
        let baseline = recompute_series(std::slice::from_ref(&steps), 300, None).expect("baseline");
        assert_ne!(with_workout, baseline, "workout must influence features (sanity)");

        let marker = Observation::try_new(
            Uuid::from_u128(3),
            UnixTimestamp::from_secs(3_500),
            "com.biofocus.desktop",
            DATA_TYPE_LIFE_EVENT_RETRACTION,
            json!({ "target_id": Uuid::from_u128(2).to_string() }),
            1.0,
        )
        .expect("marker");
        repo.insert(&marker).expect("i3");
        let after = recompute_series(
            &repo.list_by_time_range(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(4_000)).expect("q"),
            300,
            None,
        )
        .expect("series");
        assert_eq!(after, baseline, "retracted workout must not change any Feature");
    }
}
