//! `RecoveryScore` v1 — short-term physiological recovery proxy
//! (`docs/06-feature-catalog.md`).
//!
//! # v1 formula (documented simplifications)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / Stress).
//! - **Inputs:** `hrv` (`rmssd_ms` required to emit); optional `heart_rate`
//!   (`bpm`). Sleep Observations are **not** required for v1.
//! - **HRV component:** linear map — **0** at RMSSD ≤ 15 ms, **100** at
//!   RMSSD ≥ 70 ms (higher variability ⇒ higher short-term recovery proxy).
//!   Inverse anchors of `StressIndex` RMSSD map.
//! - **HR component (optional):** vs early-snapshot baseline BPM —
//!   `100 - clamp((mean_bpm - baseline) / 20 * 100, 0, 100)`. Elevated HR
//!   vs baseline lowers the component; at/below baseline ⇒ 100.
//! - **Weights:** HRV 0.70, HR 0.30 — **renormalized** when HR absent.
//! - **Provenance:** Observation IDs of `hrv` / `heart_rate` in the window.
//! - **Confidence (ADR-007):** expected slots = 2 (HRV / HR);
//!   `coverage × mean(evidence Observation.confidence)`. HRV-only → 0.5
//!   coverage when obs confidence is 1.0. Empty / no usable HRV → omit.
//! - **Explanation factors:** present components — `hrv` / `heart_rate`
//!   with renormalized shares (sum 1.0). Calm input composition only.
//! - Not a clinical recovery diagnosis; no sleep-debt claims.

use bio_spec::{ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends, snapshot_time_span, window_ending_at, WINDOW_SECS,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "RecoveryScore";

const DATA_TYPE_HRV: &str = "hrv";
const DATA_TYPE_HEART_RATE: &str = "heart_rate";

const WEIGHT_HRV: f64 = 0.70;
const WEIGHT_HR: f64 = 0.30;
/// Catalog input families for ADR-007 coverage (HRV / HR).
const EXPECTED_INPUT_SLOTS: usize = 2;

/// RMSSD (ms) at which RecoveryScore HRV component is 0.
const RMSSD_RECOVERY_MIN_MS: f64 = 15.0;
/// RMSSD (ms) at which RecoveryScore HRV component saturates at 100.
const RMSSD_RECOVERY_MAX_MS: f64 = 70.0;
/// BPM rise above baseline that maps HR component to 0.
const HR_ELEVATION_REF_BPM: f64 = 20.0;
/// Fallback resting baseline when no early HR samples exist.
const DEFAULT_BASELINE_BPM: f64 = 60.0;

const FACTOR_HRV: &str = "hrv";
const FACTOR_HR: &str = "heart_rate";
const LABEL_HRV: &str = "Heart-rate variability";
const LABEL_HR: &str = "Heart rate";

/// DAG node computing [`FEATURE_ID`] (independent; no Feature deps).
#[derive(Debug, Default, Clone)]
pub struct RecoveryScoreNode;

impl RecoveryScoreNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for RecoveryScoreNode {
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

        let baseline_bpm = baseline_heart_rate(ctx.observations(), min_ts);

        let mut features = Vec::new();
        for end in sliding_window_ends(min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx.observations(), &window, baseline_bpm) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(
    observations: &[Observation],
    window: &TimeWindow,
    baseline_bpm: f64,
) -> Option<Feature> {
    let in_win: Vec<&Observation> = observations
        .iter()
        .filter(|o| in_window(o, window))
        .collect();

    let hrv: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_HRV)
        .collect();
    let heart_rate: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_HEART_RATE)
        .collect();

    // (factor_id, label, catalog_weight, component_score)
    let mut weighted: Vec<(&str, &str, f64, f64)> = Vec::new();
    let mut present_slots = 0usize;

    let rmssd = mean_f64_field(&hrv, "rmssd_ms")?;
    weighted.push((
        FACTOR_HRV,
        LABEL_HRV,
        WEIGHT_HRV,
        hrv_ms_to_recovery(rmssd),
    ));
    present_slots += 1;

    if let Some(mean_bpm) = mean_bpm(&heart_rate) {
        weighted.push((
            FACTOR_HR,
            LABEL_HR,
            WEIGHT_HR,
            hr_to_recovery(mean_bpm, baseline_bpm),
        ));
        present_slots += 1;
    }

    let w_sum: f64 = weighted.iter().map(|(_, _, w, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted
        .iter()
        .map(|(_, _, w, s)| w * s)
        .sum::<f64>()
        / w_sum;

    let factors: Vec<ExplanationFactor> = weighted
        .iter()
        .map(|(id, label, w, _)| ExplanationFactor {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            share: w / w_sum,
        })
        .collect();

    let mut provenance = Vec::new();
    let mut evidence: Vec<&Observation> = Vec::new();
    for obs in hrv.iter().chain(heart_rate.iter()) {
        provenance.push(obs.id);
        evidence.push(*obs);
    }

    let confidence = compute_feature_confidence(EXPECTED_INPUT_SLOTS, present_slots, &evidence);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors,
    })
}

/// Higher RMSSD (ms) → higher recovery proxy; linear between catalog anchors.
fn hrv_ms_to_recovery(ms: f64) -> f64 {
    if !ms.is_finite() || ms < 0.0 {
        return 0.0;
    }
    if ms <= RMSSD_RECOVERY_MIN_MS {
        return 0.0;
    }
    if ms >= RMSSD_RECOVERY_MAX_MS {
        return 100.0;
    }
    let span = RMSSD_RECOVERY_MAX_MS - RMSSD_RECOVERY_MIN_MS;
    (100.0 * (ms - RMSSD_RECOVERY_MIN_MS) / span).clamp(0.0, 100.0)
}

fn hr_to_recovery(mean_bpm: f64, baseline_bpm: f64) -> f64 {
    let rise = (mean_bpm - baseline_bpm).max(0.0);
    (100.0 - (rise / HR_ELEVATION_REF_BPM * 100.0)).clamp(0.0, 100.0)
}

fn baseline_heart_rate(observations: &[Observation], snapshot_min: i64) -> f64 {
    let start = snapshot_min;
    let end = snapshot_min.saturating_add(WINDOW_SECS);
    let early_hr: Vec<&Observation> = observations
        .iter()
        .filter(|o| {
            o.data_type == DATA_TYPE_HEART_RATE
                && o.timestamp.as_secs() >= start
                && o.timestamp.as_secs() <= end
        })
        .collect();
    mean_bpm(&early_hr).unwrap_or(DEFAULT_BASELINE_BPM)
}

fn mean_bpm(obs: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(v) = o
            .payload
            .get("bpm")
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite() && *r > 0.0)
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

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::register_catalog_v1;
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

    fn hrv_obs_conf(id: u128, ts: i64, rmssd_ms: f64, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_HRV,
            json!({ "rmssd_ms": rmssd_ms }),
            confidence,
        )
        .expect("obs")
    }

    fn hr_obs(id: u128, ts: i64, bpm: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": bpm }),
            1.0,
        )
        .expect("obs")
    }

    fn last_recovery(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        engine.register(RecoveryScoreNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("RecoveryScore")
    }

    #[test]
    fn empty_snapshot_idle_safe() {
        let mut engine = FeatureEngine::new();
        engine.register(RecoveryScoreNode::new()).expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
        assert!(out.signals.is_empty());
    }

    #[test]
    fn no_hrv_emits_nothing_even_with_hr() {
        let batch = vec![hr_obs(1, 1500, 60.0)];
        let mut engine = FeatureEngine::new();
        engine.register(RecoveryScoreNode::new()).expect("reg");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().all(|f| f.feature_id != FEATURE_ID),
            "HR-only must omit RecoveryScore"
        );
    }

    #[test]
    fn high_rmssd_yields_high_recovery() {
        let feat = last_recovery(&[hrv_obs(1, 1500, 70.0), hrv_obs(2, 1800, 80.0)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v >= 99.0, "expected high recovery, got {v}");
        assert!((feat.confidence.get() - 0.5).abs() < 1e-12, "HRV-only coverage");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_HRV);
        assert!((feat.factors[0].share - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors[0].label, LABEL_HRV);
    }

    #[test]
    fn low_rmssd_yields_low_recovery() {
        let feat = last_recovery(&[hrv_obs(1, 1500, 15.0), hrv_obs(2, 1800, 15.0)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!(v < 1.0, "expected low recovery, got {v}");
    }

    #[test]
    fn rich_hrv_and_calm_hr_full_confidence_and_factors() {
        // Early baseline ~60; window HR stays 60 → HR component 100; RMSSD 70 → 100.
        let batch = vec![
            hr_obs(1, 900, 60.0),
            hrv_obs(2, 1500, 70.0),
            hr_obs(3, 1600, 60.0),
            hrv_obs(4, 1800, 70.0),
            hr_obs(5, 1800, 60.0),
        ];
        let feat = last_recovery(&batch);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9, "got {v}");
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        let hrv_f = feat.factors.iter().find(|f| f.id == FACTOR_HRV).expect("hrv");
        let hr_f = feat
            .factors
            .iter()
            .find(|f| f.id == FACTOR_HR)
            .expect("hr");
        assert!((hrv_f.share - WEIGHT_HRV).abs() < 1e-12);
        assert!((hr_f.share - WEIGHT_HR).abs() < 1e-12);
        assert!(feat.provenance.contains(&Uuid::from_u128(2)));
        assert!(feat.provenance.contains(&Uuid::from_u128(5)));
    }

    #[test]
    fn elevated_hr_lowers_recovery_vs_hrv_only() {
        let hrv_only = last_recovery(&[hrv_obs(1, 1500, 45.0), hrv_obs(2, 1800, 45.0)]);
        // Early HR (ts 100) sets baseline 60 outside the last 15m window; later HR 80
        // elevates vs baseline so the HR factor pulls the blend down.
        let with_elevated_hr = last_recovery(&[
            hr_obs(10, 100, 60.0),
            hrv_obs(11, 1500, 45.0),
            hr_obs(12, 1600, 80.0),
            hrv_obs(13, 1800, 45.0),
            hr_obs(14, 1800, 80.0),
        ]);
        let FeatureValue::Scalar(v_only) = hrv_only.value else {
            panic!("scalar");
        };
        let FeatureValue::Scalar(v_hr) = with_elevated_hr.value else {
            panic!("scalar");
        };
        assert!(
            v_hr < v_only,
            "elevated HR should lower blended recovery: {v_hr} vs {v_only}"
        );
    }

    #[test]
    fn thin_hrv_only_confidence_is_half() {
        let feat = last_recovery(&[hrv_obs(1, 1800, 50.0)]);
        assert!((feat.confidence.get() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let feat = last_recovery(&[
            hrv_obs_conf(1, 1500, 70.0, 0.4),
            hrv_obs_conf(2, 1800, 70.0, 0.4),
        ]);
        // coverage 0.5 × mean obs 0.4 = 0.2
        assert!((feat.confidence.get() - 0.2).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_recovery_score() {
        let batch = vec![hrv_obs(1, 1500, 70.0), hrv_obs(2, 1800, 70.0)];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "RecoveryScore must be registered via register_catalog_v1"
        );
        // StressIndex also present from same HRV; RecoveryScore is independent.
        assert!(out
            .features
            .iter()
            .any(|f| f.feature_id == "StressIndex"));
    }
}
