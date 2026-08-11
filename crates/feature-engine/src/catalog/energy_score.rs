//! `EnergyScore` v1 — calm subjective energy proxy from activity + optional
//! HR / rest context (`docs/06-feature-catalog.md` / ADR-018).
//!
//! # v1 formula
//!
//! - **Window / step:** 15 minutes / context step.
//! - **Inputs (at least one required):**
//!   - `active_energy` (`kcal`) — maps 0→0, ≥80 kcal in-window → 100.
//!   - `heart_rate` (`bpm`) optional — vs early-snapshot baseline:
//!     `100 - clamp((mean_bpm - baseline) / 25 * 100, 0, 100)`.
//!   - `sleep_interval` rest overlap in 24h lookback (same rest rules as
//!     `SleepDebt`) → sufficiency `100 × clamp(rest / 8h, 0, 1)`.
//! - **Weights:** energy 0.45, sleep 0.35, HR 0.20 — **renormalized** over
//!   present families. SpO2 unused (sparse / non-clinical).
//! - **Omit:** none of the three families present for the window/lookback.
//! - **Confidence (ADR-007):** expected slots = 3; coverage × mean evidence.
//! - **Explanation factors:** present families with renormalized shares.
//! - Not a clinical energy / fatigue diagnosis.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow, DATA_TYPE_ACTIVE_ENERGY,
    DATA_TYPE_SLEEP_INTERVAL, SLEEP_STAGE_ASLEEP, SLEEP_STAGE_AWAKE, SLEEP_STAGE_IN_BED,
    SLEEP_STAGE_UNKNOWN,
};

use crate::catalog::confidence::compute_feature_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "EnergyScore";

const DATA_TYPE_HEART_RATE: &str = "heart_rate";
const EXPECTED_INPUT_SLOTS: usize = 3;
const WEIGHT_ENERGY: f64 = 0.45;
const WEIGHT_SLEEP: f64 = 0.35;
const WEIGHT_HR: f64 = 0.20;
const KCAL_SATURATION: f64 = 80.0;
const HR_ELEVATION_REF_BPM: f64 = 25.0;
const DEFAULT_BASELINE_BPM: f64 = 60.0;
const LOOKBACK_SECS: i64 = 24 * 60 * 60;
const TARGET_REST_SECS: f64 = 8.0 * 60.0 * 60.0;

const FACTOR_ENERGY: &str = "active_energy";
const FACTOR_SLEEP: &str = "sleep_interval";
const FACTOR_HR: &str = "heart_rate";
const LABEL_ENERGY: &str = "Active energy";
const LABEL_SLEEP: &str = "Recent rest";
const LABEL_HR: &str = "Heart rate";

/// DAG node computing [`FEATURE_ID`] (independent; no Feature deps).
#[derive(Debug, Default, Clone)]
pub struct EnergyScoreNode;

impl EnergyScoreNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for EnergyScoreNode {
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
        for end in sliding_window_ends_for(ctx, min_ts, max_ts) {
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

    let energy: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_ACTIVE_ENERGY)
        .collect();
    let heart_rate: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_HEART_RATE)
        .collect();

    let mut weighted: Vec<(&str, &str, f64, f64)> = Vec::new();
    let mut evidence: Vec<&Observation> = Vec::new();
    let mut present_slots = 0usize;

    if let Some(kcal) = sum_kcal(&energy) {
        weighted.push((
            FACTOR_ENERGY,
            LABEL_ENERGY,
            WEIGHT_ENERGY,
            (100.0 * kcal / KCAL_SATURATION).clamp(0.0, 100.0),
        ));
        present_slots += 1;
        evidence.extend(energy.iter().copied());
    }

    if let Some((rest_secs, sleep_ev)) = rest_in_lookback(observations, window.end.as_secs()) {
        let sufficiency = (100.0 * rest_secs / TARGET_REST_SECS).clamp(0.0, 100.0);
        weighted.push((FACTOR_SLEEP, LABEL_SLEEP, WEIGHT_SLEEP, sufficiency));
        present_slots += 1;
        evidence.extend(sleep_ev);
    }

    if let Some(mean_bpm) = mean_bpm(&heart_rate) {
        weighted.push((
            FACTOR_HR,
            LABEL_HR,
            WEIGHT_HR,
            hr_to_energy(mean_bpm, baseline_bpm),
        ));
        present_slots += 1;
        evidence.extend(heart_rate.iter().copied());
    }

    if weighted.is_empty() {
        return None;
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

    let provenance: Vec<_> = evidence.iter().map(|o| o.id).collect();
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

fn sum_kcal(obs: &[&Observation]) -> Option<f64> {
    if obs.is_empty() {
        return None;
    }
    let mut total = 0.0;
    let mut any = false;
    for o in obs {
        if let Some(k) = o.payload.get("kcal").and_then(|v| v.as_f64()) {
            if k.is_finite() && k >= 0.0 {
                total += k;
                any = true;
            }
        }
    }
    any.then_some(total)
}

fn mean_bpm(obs: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(bpm) = o.payload.get("bpm").and_then(|v| v.as_f64()) {
            if bpm.is_finite() && bpm > 0.0 {
                sum += bpm;
                n += 1;
            }
        }
    }
    (n > 0).then_some(sum / n as f64)
}

fn hr_to_energy(mean_bpm: f64, baseline_bpm: f64) -> f64 {
    let rise = (mean_bpm - baseline_bpm).max(0.0);
    (100.0 - (rise / HR_ELEVATION_REF_BPM * 100.0)).clamp(0.0, 100.0)
}

fn baseline_heart_rate(observations: &[Observation], snapshot_min: i64) -> f64 {
    let end = snapshot_min.saturating_add(5 * 60);
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in observations
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_HEART_RATE)
    {
        let t = o.timestamp.as_secs();
        if t < snapshot_min || t > end {
            continue;
        }
        if let Some(bpm) = o.payload.get("bpm").and_then(|v| v.as_f64()) {
            if bpm.is_finite() && bpm > 0.0 {
                sum += bpm;
                n += 1;
            }
        }
    }
    if n == 0 {
        DEFAULT_BASELINE_BPM
    } else {
        sum / n as f64
    }
}

fn rest_in_lookback(
    observations: &[Observation],
    lookback_end: i64,
) -> Option<(f64, Vec<&Observation>)> {
    let lookback_start = lookback_end.saturating_sub(LOOKBACK_SECS);
    let mut evidence = Vec::new();
    let mut rest_secs = 0.0;
    for obs in observations
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_SLEEP_INTERVAL)
    {
        if !qualifies_as_rest(obs) {
            continue;
        }
        let Some((start, end)) = interval_bounds(obs) else {
            continue;
        };
        let overlap = overlap_secs(start, end, lookback_start, lookback_end);
        if overlap <= 0.0 {
            continue;
        }
        rest_secs += overlap;
        evidence.push(obs);
    }
    if evidence.is_empty() {
        None
    } else {
        Some((rest_secs, evidence))
    }
}

fn qualifies_as_rest(obs: &Observation) -> bool {
    match obs.payload.get("stage").and_then(|v| v.as_str()) {
        None => true,
        Some(SLEEP_STAGE_ASLEEP | SLEEP_STAGE_IN_BED) => true,
        Some(SLEEP_STAGE_AWAKE | SLEEP_STAGE_UNKNOWN) => false,
        Some(_) => false,
    }
}

fn interval_bounds(obs: &Observation) -> Option<(i64, i64)> {
    let start = obs.payload.get("start").and_then(json_i64)?;
    let end = obs.payload.get("end").and_then(json_i64)?;
    if end < start {
        return None;
    }
    Some((start, end))
}

fn overlap_secs(a0: i64, a1: i64, b0: i64, b1: i64) -> f64 {
    let start = a0.max(b0);
    let end = a1.min(b1);
    (end - start).max(0) as f64
}

fn json_i64(v: &serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64().or_else(|| {
            n.as_u64()
                .and_then(|u| i64::try_from(u).ok())
                .or_else(|| {
                    n.as_f64()
                        .filter(|f| f.is_finite() && f.fract() == 0.0)
                        .map(|f| f as i64)
                })
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::FeatureEngine;

    fn energy_obs(id: u128, ts: i64, kcal: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "test.provider",
            DATA_TYPE_ACTIVE_ENERGY,
            json!({ "kcal": kcal }),
            1.0,
        )
        .expect("obs")
    }

    fn last_energy(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        engine.register(EnergyScoreNode::new()).expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .next_back()
            .expect("EnergyScore")
    }

    #[test]
    fn omits_without_inputs() {
        let mut engine = FeatureEngine::new();
        engine.register(EnergyScoreNode::new()).expect("reg");
        assert!(engine.run(&[]).expect("run").features.is_empty());
    }

    #[test]
    fn emits_from_active_energy_alone() {
        let feat = last_energy(&[energy_obs(1, 1500, 40.0)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 50.0).abs() < 0.01, "got {v}");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_ENERGY);
        // 1 of 3 slots
        assert!((feat.confidence.get() - (1.0 / 3.0)).abs() < 0.01);
    }
}
