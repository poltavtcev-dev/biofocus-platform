//! `FocusScore` v1 — depth-of-focus metric (`docs/06-feature-catalog.md`).
//!
//! # v1 formula (documented simplifications)
//!
//! - **Window / step:** 15 minutes / 1 minute.
//! - **Inputs (catalog → v1 mapping):**
//!   - `keystrokes.rate_per_min` → typing score (`mean_rate / 200 * 100`, clamped 0–100).
//!   - App category → **not** a taxonomy yet; use upstream [`super::ContextSwitchRateNode`]
//!     stability: `100 - rate * 50` (clamped). Lower switch rate ⇒ higher stability.
//!   - `hrv.rmssd_ms` → comfort score (peak 100 at 45 ms, falloff to 0 at 0 / 120 ms).
//! - **Weights:** typing 0.40, stability 0.35, HRV 0.25 — **renormalized** over
//!   components that have data in the window.
//! - **Provenance:** Observation IDs of `keystrokes`, `hrv`, and `context_window`
//!   inside the window (union).
//! - Emits a Feature only when at least one component has data.

use bio_spec::{Feature, FeatureValue, Observation, TimeWindow};

use crate::catalog::context_switch_rate;
use crate::catalog::window::{
    in_window, sliding_window_ends, snapshot_time_span, window_ending_at,
};
use crate::{
    ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput,
};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "FocusScore";

const DATA_TYPE_KEYSTROKES: &str = "keystrokes";
const DATA_TYPE_HRV: &str = "hrv";
const DATA_TYPE_CONTEXT_WINDOW: &str = "context_window";

const WEIGHT_TYPING: f64 = 0.40;
const WEIGHT_STABILITY: f64 = 0.35;
const WEIGHT_HRV: f64 = 0.25;
/// Reference typing rate (keys/min) that maps to typing score 100.
const TYPING_RATE_REF: f64 = 200.0;

/// DAG node computing [`FEATURE_ID`]; depends on [`ContextSwitchRateNode`].
#[derive(Debug, Clone)]
pub struct FocusScoreNode {
    deps: Vec<NodeId>,
}

impl FocusScoreNode {
    /// Constructs the catalog node with a dependency on `ContextSwitchRate`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![context_switch_rate::FEATURE_ID.to_owned()],
        }
    }
}

impl Default for FocusScoreNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for FocusScoreNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        let Some((min_ts, max_ts)) = snapshot_time_span(ctx.observations()) else {
            return Ok(NodeOutput::empty());
        };

        let mut features = Vec::new();
        for end in sliding_window_ends(min_ts, max_ts) {
            let window = window_ending_at(end);
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let in_win: Vec<&Observation> = ctx
        .observations()
        .iter()
        .filter(|o| in_window(o, window))
        .collect();

    let keystrokes: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_KEYSTROKES)
        .collect();
    let hrv: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_HRV)
        .collect();
    let context: Vec<&Observation> = in_win
        .iter()
        .copied()
        .filter(|o| o.data_type == DATA_TYPE_CONTEXT_WINDOW)
        .collect();

    let mut weighted: Vec<(f64, f64)> = Vec::new();

    if let Some(mean_rate) = mean_keystroke_rate(&keystrokes) {
        let typing = (mean_rate / TYPING_RATE_REF * 100.0).clamp(0.0, 100.0);
        weighted.push((WEIGHT_TYPING, typing));
    }

    if let Some(csr) = upstream_csr(ctx, window) {
        let stability = (100.0 - csr * 50.0).clamp(0.0, 100.0);
        weighted.push((WEIGHT_STABILITY, stability));
    } else if !context.is_empty() {
        // CSR node skipped empty steps; treat present context with 0 switches as full stability.
        weighted.push((WEIGHT_STABILITY, 100.0));
    }

    if let Some(rmssd) = mean_rmssd_ms(&hrv) {
        weighted.push((WEIGHT_HRV, hrv_comfort_score(rmssd)));
    }

    if weighted.is_empty() {
        return None;
    }

    let w_sum: f64 = weighted.iter().map(|(w, _)| *w).sum();
    if w_sum <= 0.0 {
        return None;
    }
    let value = weighted.iter().map(|(w, s)| w * s).sum::<f64>() / w_sum;

    let mut provenance = Vec::new();
    for obs in keystrokes
        .iter()
        .chain(hrv.iter())
        .chain(context.iter())
    {
        provenance.push(obs.id);
    }

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value.clamp(0.0, 100.0)),
        provenance,
    })
}

fn upstream_csr(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<f64> {
    ctx.features()
        .iter()
        .rev()
        .find(|f| {
            f.feature_id == context_switch_rate::FEATURE_ID && f.time_window == *window
        })
        .and_then(|f| match f.value {
            FeatureValue::Scalar(v) => Some(v),
            _ => None,
        })
}

fn mean_keystroke_rate(obs: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(rate) = o
            .payload
            .get("rate_per_min")
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite())
        {
            sum += rate;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

fn mean_rmssd_ms(obs: &[&Observation]) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(v) = o
            .payload
            .get("rmssd_ms")
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite() && *r >= 0.0)
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

/// Peak comfort at 45 ms RMSSD; linear falloff to 0 at 0 ms and 120 ms.
fn hrv_comfort_score(rmssd_ms: f64) -> f64 {
    if !rmssd_ms.is_finite() || rmssd_ms <= 0.0 {
        return 0.0;
    }
    if rmssd_ms <= 45.0 {
        (rmssd_ms / 45.0 * 100.0).clamp(0.0, 100.0)
    } else {
        (100.0 * (1.0 - (rmssd_ms - 45.0) / 75.0)).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{FeatureValue, Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::register_focus_v1;
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

    #[test]
    fn high_typing_low_switches_good_hrv_scores_high() {
        // One aligned end at 1800; window [900, 1800].
        let batch = vec![
            obs(
                1,
                1000,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
            obs(
                2,
                1400,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
            obs(
                3,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 180, "window_secs": 60, "rate_per_min": 180.0 }),
            ),
            obs(
                4,
                1500,
                DATA_TYPE_HRV,
                json!({ "rmssd_ms": 45.0 }),
            ),
            obs(
                5,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "com.dev.ide", "app_name": "IDE" }),
            ),
        ];

        let mut engine = FeatureEngine::new();
        register_focus_v1(&mut engine).expect("register");
        let out = engine.run(&batch).expect("run");

        let focus: Vec<_> = out
            .features
            .iter()
            .filter(|f| f.feature_id == FEATURE_ID)
            .collect();
        assert!(!focus.is_empty());
        let last = focus.last().expect("focus");
        assert_eq!(last.time_window.end.as_secs(), 1800);
        let FeatureValue::Scalar(score) = last.value else {
            panic!("scalar");
        };
        // typing ~90, stability 100 (0 switches), hrv 100 → weighted ≈ 96
        assert!(score > 90.0, "expected high focus, got {score}");
        assert!(!last.provenance.is_empty());
        assert!(last.provenance.contains(&Uuid::from_u128(3)));
        assert!(last.provenance.contains(&Uuid::from_u128(4)));
    }

    #[test]
    fn frequent_switches_lower_focus_than_stable() {
        let stable = vec![
            obs(
                1,
                900,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(
                2,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(
                3,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 100, "window_secs": 60, "rate_per_min": 100.0 }),
            ),
        ];
        let chaotic = vec![
            obs(
                11,
                900,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "a", "app_name": "A" }),
            ),
            obs(
                12,
                1100,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "b", "app_name": "B" }),
            ),
            obs(
                13,
                1300,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "c", "app_name": "C" }),
            ),
            obs(
                14,
                1500,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "d", "app_name": "D" }),
            ),
            obs(
                15,
                1800,
                DATA_TYPE_CONTEXT_WINDOW,
                json!({ "bundle_id": "e", "app_name": "E" }),
            ),
            obs(
                16,
                1200,
                DATA_TYPE_KEYSTROKES,
                json!({ "count": 100, "window_secs": 60, "rate_per_min": 100.0 }),
            ),
        ];

        let mut eng_s = FeatureEngine::new();
        register_focus_v1(&mut eng_s).expect("reg");
        let mut eng_c = FeatureEngine::new();
        register_focus_v1(&mut eng_c).expect("reg");

        let score_stable = last_focus(&eng_s.run(&stable).expect("run"));
        let score_chaotic = last_focus(&eng_c.run(&chaotic).expect("run"));
        assert!(
            score_stable > score_chaotic,
            "stable {score_stable} should beat chaotic {score_chaotic}"
        );
    }

    fn last_focus(out: &crate::EngineOutput) -> f64 {
        out.features
            .iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .and_then(|f| match f.value {
                FeatureValue::Scalar(v) => Some(v),
                _ => None,
            })
            .expect("FocusScore")
    }
}
