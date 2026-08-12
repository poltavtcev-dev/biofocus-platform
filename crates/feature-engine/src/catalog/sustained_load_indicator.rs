//! `SustainedLoadIndicator` v1 — calm prolonged-load persistence from
//! Feature-level Stress + Fatigue + MeetingDensity (`docs/06-feature-catalog.md`
//! / ADR-026 / P25-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (catalog DAG; series may coarsen).
//! - **Lookback for persistence math:** 4 hours ending at the Feature window end
//!   (multi-window evidence — longer than CognitiveLoad’s single 15m snapshot;
//!   not multi-day clinical burnout profiling).
//! - **Inputs (Feature-level only):**
//!   - [`StressIndex`](super::StressIndexNode) — mean of samples with ends in lookback
//!   - [`FatigueIndex`](super::FatigueIndexNode) — mean of samples with ends in lookback
//!   - [`MeetingDensity`](super::MeetingDensityNode) — optional schedule reinforcement;
//!     mean density × 100 → 0–100
//! - **Weights:** stress 0.40 / fatigue 0.40 / meeting 0.20 — **renormalized** over
//!   present terms.
//! - **Omit policy (LOCKED):** if **both** StressIndex and FatigueIndex are absent
//!   in the lookback → **omit**. MeetingDensity alone must **not** emit.
//! - **Confidence (ADR-007):** expected slots = 3;
//!   `coverage × mean(upstream Feature.confidence of contributing samples)`.
//! - **Explanation factors:** present components — `stress` / `fatigue` /
//!   `meeting` with calm labels; shares sum to 1.0.
//! - Calm framing: “prolonged load in this window” — **not** burnout /
//!   “you are burned out”. **Distinct from CognitiveLoad** (current demand).
//! - **Do not** use CognitiveLoad / FocusScore / CircadianOffset as inputs.

use bio_spec::{ExplanationFactor, Feature, FeatureValue, TimeWindow};

use crate::catalog::confidence::compute_from_values;
use crate::catalog::fatigue_index;
use crate::catalog::meeting_density;
use crate::catalog::stress_index;
use crate::catalog::window::window_ending_at;
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "SustainedLoadIndicator";

const LOOKBACK_SECS: i64 = 4 * 60 * 60;

const WEIGHT_STRESS: f64 = 0.40;
const WEIGHT_FATIGUE: f64 = 0.40;
const WEIGHT_MEETING: f64 = 0.20;
const EXPECTED_INPUT_SLOTS: usize = 3;

const FACTOR_STRESS: &str = "stress";
const FACTOR_FATIGUE: &str = "fatigue";
const FACTOR_MEETING: &str = "meeting";
const LABEL_STRESS: &str = "Stress load";
const LABEL_FATIGUE: &str = "Fatigue load";
const LABEL_MEETING: &str = "Schedule density";

/// DAG node computing [`FEATURE_ID`]; depends on StressIndex + FatigueIndex +
/// MeetingDensity.
#[derive(Debug, Clone)]
pub struct SustainedLoadIndicatorNode {
    deps: Vec<NodeId>,
}

impl SustainedLoadIndicatorNode {
    /// Constructs the catalog node with Feature-level DAG dependencies.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![
                stress_index::FEATURE_ID.to_owned(),
                fatigue_index::FEATURE_ID.to_owned(),
                meeting_density::FEATURE_ID.to_owned(),
            ],
        }
    }
}

impl Default for SustainedLoadIndicatorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for SustainedLoadIndicatorNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        // Drive windows from Stress / Fatigue only — MeetingDensity-alone must
        // never create a SustainedLoad step (ADR-026 omit lock).
        let mut ends: Vec<i64> = ctx
            .features()
            .iter()
            .filter(|f| {
                f.feature_id == stress_index::FEATURE_ID
                    || f.feature_id == fatigue_index::FEATURE_ID
            })
            .map(|f| f.time_window.end.as_secs())
            .collect();
        ends.sort_unstable();
        ends.dedup();

        let mut features = Vec::new();
        for end in ends {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let lookback_start = window.end.as_secs().saturating_sub(LOOKBACK_SECS);
    let lookback_end = window.end.as_secs();

    let stress_samples = samples_in_lookback(ctx, stress_index::FEATURE_ID, lookback_start, lookback_end);
    let fatigue_samples =
        samples_in_lookback(ctx, fatigue_index::FEATURE_ID, lookback_start, lookback_end);
    let meeting_samples =
        samples_in_lookback(ctx, meeting_density::FEATURE_ID, lookback_start, lookback_end);

    if stress_samples.is_empty() && fatigue_samples.is_empty() {
        return None;
    }

    // (factor_id, label, catalog_weight, component_score, contributing Features)
    let mut weighted: Vec<(&str, &str, f64, f64, Vec<&Feature>)> = Vec::new();

    if let Some(stress_term) = mean_scalar(&stress_samples) {
        weighted.push((
            FACTOR_STRESS,
            LABEL_STRESS,
            WEIGHT_STRESS,
            stress_term.clamp(0.0, 100.0),
            stress_samples.clone(),
        ));
    }

    if let Some(fatigue_term) = mean_scalar(&fatigue_samples) {
        weighted.push((
            FACTOR_FATIGUE,
            LABEL_FATIGUE,
            WEIGHT_FATIGUE,
            fatigue_term.clamp(0.0, 100.0),
            fatigue_samples.clone(),
        ));
    }

    if let Some(density_mean) = mean_scalar(&meeting_samples) {
        let meeting_term = (density_mean * 100.0).clamp(0.0, 100.0);
        weighted.push((
            FACTOR_MEETING,
            LABEL_MEETING,
            WEIGHT_MEETING,
            meeting_term,
            meeting_samples.clone(),
        ));
    }

    if weighted.is_empty() {
        return None;
    }

    let w_sum: f64 = weighted.iter().map(|(_, _, w, _, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted
        .iter()
        .map(|(_, _, w, s, _)| w * s)
        .sum::<f64>()
        / w_sum;

    let factors: Vec<ExplanationFactor> = weighted
        .iter()
        .map(|(id, label, w, _, _)| ExplanationFactor {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            share: w / w_sum,
        })
        .collect();

    let mut provenance = Vec::new();
    let mut conf_values = Vec::new();
    for (_, _, _, _, feats) in &weighted {
        for feat in feats {
            provenance.extend(feat.provenance.iter().copied());
            conf_values.push(feat.confidence.get());
        }
    }
    provenance.sort_unstable();
    provenance.dedup();

    let present_slots = weighted.len();
    let confidence = compute_from_values(EXPECTED_INPUT_SLOTS, present_slots, &conf_values);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
        confidence,
        factors,
    })
}

fn samples_in_lookback<'a>(
    ctx: &'a ComputeContext<'_>,
    feature_id: &str,
    lookback_start: i64,
    lookback_end: i64,
) -> Vec<&'a Feature> {
    ctx.features()
        .iter()
        .filter(|f| {
            f.feature_id == feature_id
                && f.time_window.end.as_secs() >= lookback_start
                && f.time_window.end.as_secs() <= lookback_end
        })
        .collect()
}

fn mean_scalar(samples: &[&Feature]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for f in samples {
        if let FeatureValue::Scalar(v) = f.value {
            if v.is_finite() {
                sum += v;
                n += 1;
            }
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
    use bio_spec::{
        FeatureValue, Observation, UnixTimestamp, DATA_TYPE_CALENDAR_EVENT,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::{
        register_calendar_v1, register_catalog_v1, register_focus_v1, register_stress_v1,
        COGNITIVE_LOAD_ID,
    };
    use crate::FeatureEngine;

    fn obs(
        id: u128,
        ts: i64,
        data_type: &str,
        payload: serde_json::Value,
    ) -> Observation {
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

    fn cal_obs(id: u128, start: i64, end: i64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(start),
            "com.biofocus.macos.calendar",
            DATA_TYPE_CALENDAR_EVENT,
            json!({ "uid": format!("evt-{id}"), "start": start, "end": end, "busy": true }),
            1.0,
        )
        .expect("cal")
    }

    fn register_sustained_deps(engine: &mut FeatureEngine) {
        register_focus_v1(engine).expect("focus");
        register_stress_v1(engine).expect("stress");
        register_calendar_v1(engine).expect("calendar");
        engine
            .register(SustainedLoadIndicatorNode::new())
            .expect("sustained");
    }

    fn last_sustained(batch: &[Observation]) -> Option<Feature> {
        let mut engine = FeatureEngine::new();
        register_sustained_deps(&mut engine);
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
    }

    /// Stress (low HRV) + Fatigue path (typing/HR/context) + full meeting window.
    fn rich_batch() -> Vec<Observation> {
        vec![
            obs(
                1,
                900,
                "context_window",
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(2, 1000, "heart_rate", json!({ "bpm": 60.0 })),
            obs(
                3,
                1200,
                "keystrokes",
                json!({ "count": 10, "window_secs": 60, "rate_per_min": 10.0 }),
            ),
            obs(4, 1400, "hrv", json!({ "rmssd_ms": 20.0 })),
            obs(5, 1600, "heart_rate", json!({ "bpm": 85.0 })),
            obs(
                6,
                1800,
                "context_window",
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            cal_obs(7, 900, 1800),
        ]
    }

    #[test]
    fn stress_fatigue_meeting_emits_with_factors() {
        let feat = last_sustained(&rich_batch()).expect("SustainedLoadIndicator");
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((0.0..=100.0).contains(&v));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_STRESS));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_FATIGUE));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_MEETING));
        assert_eq!(feat.factors.len(), 3);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-9);
        assert!(!feat.provenance.is_empty());
        // coverage = 1.0; mean upstream conf may be < 1 when Fatigue is partial
        assert!(feat.confidence.get() > 0.5 && feat.confidence.get() <= 1.0);
        for f in &feat.factors {
            let lower = f.label.to_lowercase();
            assert!(!lower.contains("burnout"));
            assert!(!lower.contains("burned"));
            assert!(!lower.contains("clinical"));
        }
    }

    #[test]
    fn stress_fatigue_without_meeting_renormalizes() {
        let batch = vec![
            obs(
                1,
                900,
                "context_window",
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(2, 1000, "heart_rate", json!({ "bpm": 60.0 })),
            obs(
                3,
                1200,
                "keystrokes",
                json!({ "count": 10, "window_secs": 60, "rate_per_min": 10.0 }),
            ),
            obs(4, 1400, "hrv", json!({ "rmssd_ms": 25.0 })),
            obs(5, 1600, "heart_rate", json!({ "bpm": 80.0 })),
            obs(
                6,
                1800,
                "context_window",
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
        ];
        let feat = last_sustained(&batch).expect("emit without meeting");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_STRESS));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_FATIGUE));
        assert!(!feat.factors.iter().any(|f| f.id == FACTOR_MEETING));
        assert_eq!(feat.factors.len(), 2);
        // coverage = 2/3; mean upstream conf ≤ 1 → confidence ≤ 2/3
        assert!(feat.confidence.get() <= 2.0 / 3.0 + 1e-9);
        assert!(feat.confidence.get() > 0.2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn meetings_alone_omits() {
        let batch = [cal_obs(1, 900, 1800)];
        assert!(
            last_sustained(&batch).is_none(),
            "MeetingDensity alone must omit"
        );
    }

    #[test]
    fn omits_when_both_stress_and_fatigue_absent() {
        let mut engine = FeatureEngine::new();
        register_sustained_deps(&mut engine);
        let out = engine.run(&[]).expect("run");
        assert!(!out.features.iter().any(|f| f.feature_id == FEATURE_ID));
    }

    #[test]
    fn stress_from_hrv_emits_without_meeting() {
        // HRV alone drives StressIndex; Focus/Fatigue may also appear via HRV→Focus
        // path — MeetingDensity must stay absent; SustainedLoad must emit.
        let batch = [obs(1, 1500, "hrv", json!({ "rmssd_ms": 20.0 }))];
        let feat = last_sustained(&batch).expect("stress path");
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_STRESS));
        assert!(!feat.factors.iter().any(|f| f.id == FACTOR_MEETING));
        assert!(feat.confidence.get() > 0.0 && feat.confidence.get() <= 1.0);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((0.0..=100.0).contains(&v));
    }

    #[test]
    fn no_cognitive_load_as_input_path() {
        let src = include_str!("sustained_load_indicator.rs");
        let prod: String = src
            .lines()
            .take_while(|l| !l.contains("mod tests"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!prod.contains("CognitiveLoadNode"));
        assert!(!prod.contains("\"CognitiveLoad\""));
        assert!(!prod.contains("cognitive_load::"));
        assert!(!prod.contains("FocusScoreNode"));
        assert!(!prod.contains("\"FocusScore\""));
        assert!(!prod.contains("CircadianOffsetNode"));
        assert!(!prod.contains("\"CircadianOffset\""));
        assert!(!prod.contains("SleepDebtNode"));
        assert!(!prod.contains("\"SleepDebt\""));
    }

    #[test]
    fn register_catalog_v1_includes_sustained_load() {
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&rich_batch()).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "SustainedLoadIndicator must be registered via register_catalog_v1"
        );
        // Sibling CognitiveLoad may also emit from overlapping inputs — that's OK;
        // SustainedLoad must not depend on it.
        let _ = COGNITIVE_LOAD_ID;
    }

    #[test]
    fn lookback_constant_is_four_hours() {
        assert_eq!(LOOKBACK_SECS, 4 * 60 * 60);
    }
}
