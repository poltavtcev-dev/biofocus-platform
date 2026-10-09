//! `StressIndex` v1 + transient `High_Stress` Signal (`docs/06-feature-catalog.md`).
//!
//! # v1 formula (documented simplifications)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus catalog).
//! - **Inputs:** normalized `hrv` payloads with RMSSD (`rmssd_ms`). SDNN is a
//!   different method and does not enter this map. Optional `pnn50` may
//!   average in when present.
//! - **RMSSD → stress:** linear map — **100** at RMSSD ≤ 15 ms, **0** at
//!   RMSSD ≥ 70 ms (low RMSSD ⇒ higher index). Those anchors are not applied
//!   to SDNN. `pnn50` (0–100) contributes `100 - pnn50`.
//! - **Provenance:** Observation IDs of `hrv` inside the window.
//! - **Confidence (ADR-007):** single family (HRV); when emitted,
//!   `confidence = mean(hrv Observation.confidence)`. Empty HRV → omit.
//! - Empty HRV window → no Feature for that step.
//!
//! # High_Stress Signal
//!
//! Catalog threshold: `StressIndex` > 75 for **longer than** 5 minutes.
//! On consecutive minute-aligned samples all `> 75`, if
//! `(last_end - first_end) > 300` secs → one transient
//! [`Signal`](bio_spec::Signal) with `signal_type = "High_Stress"` and
//! [`Severity::High`](bio_spec::Severity::High).

use bio_spec::{Feature, FeatureValue, Observation, Severity, Signal, UnixTimestamp};
use uuid::Uuid;

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at, STEP_SECS,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "StressIndex";

/// Transient Signal type when stress stays elevated (`docs/06-feature-catalog.md`).
pub const HIGH_STRESS_SIGNAL_TYPE: &str = "High_Stress";

const DATA_TYPE_HRV: &str = "hrv";

/// Catalog trigger: StressIndex must exceed this value.
pub const HIGH_STRESS_THRESHOLD: f64 = 75.0;

/// Catalog: elevated longer than 5 minutes (strict `>`).
pub const HIGH_STRESS_MIN_DURATION_SECS: i64 = 5 * 60;

/// RMSSD (ms) at which StressIndex saturates at 100.
const RMSSD_STRESS_MAX_MS: f64 = 15.0;
/// RMSSD (ms) at which StressIndex reaches 0.
const RMSSD_STRESS_MIN_MS: f64 = 70.0;

/// DAG node computing [`FEATURE_ID`] and optional `High_Stress` Signals.
#[derive(Debug, Default, Clone)]
pub struct StressIndexNode;

impl StressIndexNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for StressIndexNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &[]
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let Some((min_ts, max_ts)) = snapshot_time_span(ctx.observations()) else {
            return Ok(NodeOutput::empty());
        };

        let mut features = Vec::new();
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx.observations(), &window) {
                features.push(feature);
            }
        }

        let signals = high_stress_signals(&features);
        Ok(NodeOutput { features, signals })
    }
}

fn score_window(observations: &[Observation], window: &bio_spec::TimeWindow) -> Option<Feature> {
    let hrv: Vec<&Observation> = observations
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_HRV && in_window(o, window))
        .collect();
    if hrv.is_empty() {
        return None;
    }

    let value = stress_from_hrv(&hrv)?;
    let provenance = hrv.iter().map(|o| o.id).collect();
    let confidence = single_family_confidence(&hrv);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors: Vec::new(),
    })
}

fn stress_from_hrv(hrv: &[&Observation]) -> Option<f64> {
    let mut parts: Vec<f64> = Vec::new();

    // RMSSD only. SDNN never uses these 15/70 ms anchors.
    if let Some(ms) = super::hrv::mean_hrv_ms(hrv) {
        parts.push(hrv_ms_to_stress(ms));
    }
    if let Some(pnn50) = mean_f64_field(hrv, "pnn50") {
        parts.push((100.0 - pnn50.clamp(0.0, 100.0)).clamp(0.0, 100.0));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.iter().sum::<f64>() / parts.len() as f64)
    }
}

/// Low variability (ms) → high stress; linear between catalog anchors.
fn hrv_ms_to_stress(ms: f64) -> f64 {
    if !ms.is_finite() || ms < 0.0 {
        return 0.0;
    }
    if ms <= RMSSD_STRESS_MAX_MS {
        return 100.0;
    }
    if ms >= RMSSD_STRESS_MIN_MS {
        return 0.0;
    }
    let span = RMSSD_STRESS_MIN_MS - RMSSD_STRESS_MAX_MS;
    (100.0 * (RMSSD_STRESS_MIN_MS - ms) / span).clamp(0.0, 100.0)
}

fn mean_f64_field(obs: &[&Observation], key: &str) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(v) = o
            .payload
            .get(key)
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite())
        {
            sum += v;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

/// Maximal contiguous runs of minute-aligned StressIndex > threshold lasting > 5m.
fn high_stress_signals(features: &[Feature]) -> Vec<Signal> {
    let mut samples: Vec<(i64, f64)> = features
        .iter()
        .filter(|f| f.feature_id == FEATURE_ID)
        .filter_map(|f| match f.value {
            FeatureValue::Scalar(v) if v.is_finite() => Some((f.time_window.end.as_secs(), v)),
            _ => None,
        })
        .collect();
    samples.sort_by_key(|(end, _)| *end);
    samples.dedup_by_key(|(end, _)| *end);

    let mut signals = Vec::new();
    let mut run_start: Option<i64> = None;
    let mut run_end: Option<i64> = None;
    let mut prev_end: Option<i64> = None;

    let flush = |signals: &mut Vec<Signal>, start: i64, end: i64| {
        if end - start > HIGH_STRESS_MIN_DURATION_SECS {
            signals.push(Signal {
                id: Uuid::now_v7(),
                signal_type: HIGH_STRESS_SIGNAL_TYPE.to_owned(),
                timestamp_start: UnixTimestamp::from_secs(start),
                timestamp_end: UnixTimestamp::from_secs(end),
                severity: Severity::High,
            });
        }
    };

    for (end, value) in samples {
        let elevated = value > HIGH_STRESS_THRESHOLD;
        let contiguous = prev_end.is_some_and(|p| end - p == STEP_SECS);

        if elevated && (run_start.is_none() || contiguous) {
            if run_start.is_none() {
                run_start = Some(end);
            }
            run_end = Some(end);
        } else {
            if let (Some(s), Some(e)) = (run_start, run_end) {
                flush(&mut signals, s, e);
            }
            if elevated {
                run_start = Some(end);
                run_end = Some(end);
            } else {
                run_start = None;
                run_end = None;
            }
        }
        prev_end = Some(end);
    }
    if let (Some(s), Some(e)) = (run_start, run_end) {
        flush(&mut signals, s, e);
    }

    signals
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, Severity};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::FeatureEngine;

    fn hrv_obs(id: u128, ts: i64, rmssd_ms: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_HRV,
            json!({ "rmssd_ms": rmssd_ms }),
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn low_rmssd_yields_high_stress() {
        // Window end 1800; RMSSD 15 → stress 100.
        let batch = vec![hrv_obs(1, 1500, 15.0), hrv_obs(2, 1800, 15.0)];
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        let last = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("stress");
        let FeatureValue::Scalar(v) = last.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9, "got {v}");
        assert!(last.provenance.contains(&Uuid::from_u128(1)));
        assert!((last.confidence.get() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn low_observation_confidence_lowers_stress_feature_confidence() {
        let batch = vec![
            Observation::try_new(
                Uuid::from_u128(1),
                UnixTimestamp::from_secs(1500),
                "test.provider",
                DATA_TYPE_HRV,
                json!({ "rmssd_ms": 15.0 }),
                0.5,
            )
            .expect("obs"),
            Observation::try_new(
                Uuid::from_u128(2),
                UnixTimestamp::from_secs(1800),
                "test.provider",
                DATA_TYPE_HRV,
                json!({ "rmssd_ms": 15.0 }),
                0.5,
            )
            .expect("obs"),
        ];
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        let last = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("stress");
        assert!((last.confidence.get() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn high_rmssd_yields_low_stress() {
        let batch = vec![hrv_obs(1, 1500, 70.0), hrv_obs(2, 1800, 80.0)];
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        let last = out
            .features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("stress");
        let FeatureValue::Scalar(v) = last.value else {
            panic!("scalar");
        };
        assert!(v < 5.0, "expected low stress, got {v}");
    }

    #[test]
    fn high_stress_signal_after_more_than_five_minutes() {
        // 7 consecutive minute ends with RMSSD 15 → stress 100; span 360 > 300.
        let mut batch = Vec::new();
        for (i, ts) in (900..=1260).step_by(60).enumerate() {
            batch.push(hrv_obs(100 + i as u128, ts, 15.0));
        }
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");

        assert_eq!(out.signals.len(), 1);
        let sig = &out.signals[0];
        assert_eq!(sig.signal_type, HIGH_STRESS_SIGNAL_TYPE);
        assert_eq!(sig.severity, Severity::High);
        assert!(
            sig.timestamp_end.as_secs() - sig.timestamp_start.as_secs()
                > HIGH_STRESS_MIN_DURATION_SECS
        );
    }

    #[test]
    fn no_high_stress_signal_when_elevated_only_five_minutes() {
        // span 300 == 5m, catalog requires strictly longer than 5m.
        let mut batch = Vec::new();
        for (i, ts) in (900..=1200).step_by(60).enumerate() {
            batch.push(hrv_obs(200 + i as u128, ts, 15.0));
        }
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.signals.is_empty(),
            "expected no High_Stress for span==5m, got {:?}",
            out.signals
        );
    }

    #[test]
    fn empty_hrv_emits_nothing() {
        let batch = vec![Observation::try_new(
            Uuid::from_u128(1),
            UnixTimestamp::from_secs(1000),
            "test.provider",
            "context_window",
            json!({ "bundle_id": "a", "app_name": "A" }),
            1.0,
        )
        .expect("obs")];
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(out.features.is_empty());
        assert!(out.signals.is_empty());
    }

    fn sdnn_obs(id: u128, ts: i64, sdnn_ms: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": sdnn_ms }),
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn sdnn_does_not_use_rmssd_anchors() {
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let out = engine.run(&[sdnn_obs(1, 1500, 15.0)]).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "SDNN alone must omit StressIndex, not emit 0"
        );
    }

    #[test]
    fn mixed_methods_follow_rmssd_only() {
        // RMSSD 70 maps to stress 0. SDNN 15 would map to 100 if it were mixed in.
        let batch = vec![hrv_obs(1, 1500, 70.0), sdnn_obs(2, 1600, 15.0)];
        let mut engine = FeatureEngine::new();
        engine.register(StressIndexNode::new()).expect("reg");
        let last = engine
            .run(&batch)
            .expect("run")
            .features
            .into_iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .next_back()
            .expect("StressIndex from RMSSD");
        let FeatureValue::Scalar(v) = last.value else {
            panic!("scalar");
        };
        assert!(v < 1.0, "mixed window must follow RMSSD only, got {v}");
    }
}
