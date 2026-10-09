//! `CognitiveLoad` v1 — calm combined-demand proxy from Feature-level inputs
//! (`docs/06-feature-catalog.md` / ADR-021 / P20-E2).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus / CSR /
//!   NotificationPressure; series may coarsen).
//! - **Inputs (Feature-level):** upstream [`MeetingDensity`](super::MeetingDensityNode),
//!   [`ContextSwitchRate`](super::ContextSwitchRateNode), and
//!   [`NotificationPressure`](super::NotificationPressureNode) for the **same**
//!   window. Not a raw Observation mix.
//! - **Normalize to 0–100:**
//!   - `meeting = MeetingDensity × 100`
//!   - `switches = switch_load(ContextSwitchRate)` — smooth curve, see `switch_curve.rs`
//!     (v1 `× 50` clamp saturated at 2 switches/min)
//!   - `notify = NotificationPressure` (already 0–100)
//! - **Weights:** equal thirds (⅓ each) — **renormalized** over present inputs.
//! - **Omit policy:** if **none** of the three upstream Features are present
//!   for the step → **omit**. One or more present → emit (partial windows OK —
//!   NotificationPressure is often empty when opt-in is off).
//! - **Confidence (ADR-007):** expected slots = 3;
//!   `coverage × mean(upstream Feature.confidence)`.
//! - **Explanation factors:** present components — `meeting` / `switches` /
//!   `notifications` with calm labels; shares sum to 1.0.
//! - Calm framing only: “combined demand in this window” — **not** clinical
//!   cognitive overload / ADHD / burnout / “you are overloaded.”

use bio_spec::{ExplanationFactor, Feature, FeatureValue, TimeWindow};

use crate::catalog::confidence::compute_from_values;
use crate::catalog::context_switch_rate;
use crate::catalog::meeting_density;
use crate::catalog::notification_pressure;
use crate::catalog::window::window_ending_at;
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "CognitiveLoad";

const WEIGHT_MEETING: f64 = 1.0 / 3.0;
const WEIGHT_SWITCHES: f64 = 1.0 / 3.0;
const WEIGHT_NOTIFY: f64 = 1.0 / 3.0;
/// Catalog input families for ADR-007 coverage (meeting / switches / notify).
const EXPECTED_INPUT_SLOTS: usize = 3;
/// Maps CSR (switches/min) onto the switches component scale (ADR-021).

const FACTOR_MEETING: &str = "meeting";
const FACTOR_SWITCHES: &str = "switches";
const FACTOR_NOTIFY: &str = "notifications";
const LABEL_MEETING: &str = "Schedule demand";
const LABEL_SWITCHES: &str = "App switching";
const LABEL_NOTIFY: &str = "Interruption intensity";

/// DAG node computing [`FEATURE_ID`]; depends on MeetingDensity + CSR +
/// NotificationPressure.
#[derive(Debug, Clone)]
pub struct CognitiveLoadNode {
    deps: Vec<NodeId>,
}

impl CognitiveLoadNode {
    /// Constructs the catalog node with Feature-level DAG dependencies.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deps: vec![
                meeting_density::FEATURE_ID.to_owned(),
                context_switch_rate::FEATURE_ID.to_owned(),
                notification_pressure::FEATURE_ID.to_owned(),
            ],
        }
    }
}

impl Default for CognitiveLoadNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureNode for CognitiveLoadNode {
    fn id(&self) -> &str {
        FEATURE_ID
    }

    fn depends_on(&self) -> &[NodeId] {
        &self.deps
    }

    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
        // Feature-level composite: iterate unique windows already emitted by
        // MeetingDensity / CSR / NotificationPressure (not Observation span —
        // calendar meetings may span beyond the Observation.timestamp).
        let mut ends: Vec<i64> = ctx
            .features()
            .iter()
            .filter(|f| {
                f.feature_id == meeting_density::FEATURE_ID
                    || f.feature_id == context_switch_rate::FEATURE_ID
                    || f.feature_id == notification_pressure::FEATURE_ID
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
    // (factor_id, label, catalog_weight, component_score, upstream Feature)
    let mut weighted: Vec<(&str, &str, f64, f64, &Feature)> = Vec::new();

    if let Some(md) = upstream_feature(ctx, meeting_density::FEATURE_ID, window) {
        let density = scalar_value(md)?;
        let meeting = (density * 100.0).clamp(0.0, 100.0);
        weighted.push((FACTOR_MEETING, LABEL_MEETING, WEIGHT_MEETING, meeting, md));
    }

    if let Some(csr) = upstream_feature(ctx, context_switch_rate::FEATURE_ID, window) {
        let rate = scalar_value(csr)?;
        let switches = crate::catalog::switch_curve::switch_load(rate);
        weighted.push((
            FACTOR_SWITCHES,
            LABEL_SWITCHES,
            WEIGHT_SWITCHES,
            switches,
            csr,
        ));
    }

    if let Some(np) = upstream_feature(ctx, notification_pressure::FEATURE_ID, window) {
        let notify = scalar_value(np)?.clamp(0.0, 100.0);
        weighted.push((FACTOR_NOTIFY, LABEL_NOTIFY, WEIGHT_NOTIFY, notify, np));
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
    for (_, _, _, _, feat) in &weighted {
        provenance.extend(feat.provenance.iter().copied());
        conf_values.push(feat.confidence.get());
    }

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

fn upstream_feature<'a>(
    ctx: &'a ComputeContext<'_>,
    feature_id: &str,
    window: &TimeWindow,
) -> Option<&'a Feature> {
    ctx.features()
        .iter()
        .rev()
        .find(|f| f.feature_id == feature_id && f.time_window == *window)
}

fn scalar_value(feat: &Feature) -> Option<f64> {
    match feat.value {
        FeatureValue::Scalar(v) => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{
        FeatureValue, Observation, UnixTimestamp, DATA_TYPE_CALENDAR_EVENT,
        DATA_TYPE_NOTIFICATION_EVENT,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::catalog::{
        register_calendar_v1, register_catalog_v1, register_focus_v1, register_notification_v1,
    };
    use crate::FeatureEngine;

    fn cal_obs(id: u128, start: i64, end: i64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(start),
            "com.biofocus.macos.calendar",
            DATA_TYPE_CALENDAR_EVENT,
            json!({ "uid": format!("evt-{id}"), "start": start, "end": end, "busy": true }),
            1.0,
        )
        .expect("obs")
    }

    fn ctx_obs(id: u128, ts: i64, bundle: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.context",
            "context_window",
            json!({ "bundle_id": bundle }),
            1.0,
        )
        .expect("obs")
    }

    fn notif_obs(id: u128, ts: i64, count: u64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.notifications",
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": count }),
            1.0,
        )
        .expect("obs")
    }

    fn notif_obs_conf(id: u128, ts: i64, count: u64, confidence: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.notifications",
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": count }),
            confidence,
        )
        .expect("obs")
    }

    fn register_cognitive_deps(engine: &mut FeatureEngine) {
        register_focus_v1(engine).expect("focus");
        register_calendar_v1(engine).expect("calendar");
        register_notification_v1(engine).expect("notification");
        engine
            .register(CognitiveLoadNode::new())
            .expect("cognitive");
    }

    fn last_cognitive(batch: &[Observation]) -> Feature {
        let mut engine = FeatureEngine::new();
        register_cognitive_deps(&mut engine);
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("CognitiveLoad")
    }

    #[test]
    fn empty_snapshot_omits_feature() {
        let mut engine = FeatureEngine::new();
        register_cognitive_deps(&mut engine);
        let out = engine.run(&[]).expect("run");
        assert!(!out.features.iter().any(|f| f.feature_id == FEATURE_ID));
    }

    #[test]
    fn rich_all_three_emits_with_factors() {
        // Full 15m meeting → MeetingDensity 1.0 → meeting component 100
        // Two bundle switches in window → CSR = 2/15 → switches = clamp(2/15*50) ≈ 6.666…
        // Notification sum count = 4 → pressure 20
        // Equal thirds: (100 + 6.666… + 20) / 3 ≈ 42.222…
        let batch = vec![
            cal_obs(1, 900, 1800),
            ctx_obs(2, 1000, "a.b"),
            ctx_obs(3, 1200, "c.d"),
            ctx_obs(4, 1400, "a.b"),
            notif_obs(5, 1500, 1),
            notif_obs(6, 1800, 3),
        ];
        let feat = last_cognitive(&batch);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        let switches = crate::catalog::switch_curve::switch_load(2.0 / 15.0);
        let expected = (100.0 + switches + 20.0) / 3.0;
        assert!(
            (v - expected).abs() < 1e-9,
            "expected {expected}, got {v}"
        );
        assert_eq!(feat.factors.len(), 3);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_MEETING));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_SWITCHES));
        assert!(feat.factors.iter().any(|f| f.id == FACTOR_NOTIFY));
        assert!(feat.factors.iter().all(|f| {
            let lower = f.label.to_lowercase();
            !lower.contains("overload")
                && !lower.contains("burnout")
                && !lower.contains("adhd")
                && !lower.contains("you are")
        }));
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert!(!feat.provenance.is_empty());
    }

    #[test]
    fn partial_notifications_only_renormalizes() {
        // Only NotificationPressure present → value = notify; coverage 1/3
        let feat = last_cognitive(&[notif_obs(1, 1500, 4)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 20.0).abs() < 1e-9, "expected 20, got {v}");
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_NOTIFY);
        assert!((feat.factors[0].share - 1.0).abs() < 1e-12);
        assert!((feat.confidence.get() - (1.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn partial_meeting_only_renormalizes() {
        let feat = last_cognitive(&[cal_obs(1, 900, 1800)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9);
        assert_eq!(feat.factors.len(), 1);
        assert_eq!(feat.factors[0].id, FACTOR_MEETING);
        assert!((feat.confidence.get() - (1.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn low_upstream_confidence_lowers_feature() {
        let feat = last_cognitive(&[notif_obs_conf(1, 1500, 4, 0.6)]);
        // coverage 1/3 × mean upstream conf 0.6 = 0.2
        assert!((feat.confidence.get() - 0.2).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_cognitive_load() {
        let batch = vec![notif_obs(1, 1500, 1)];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "CognitiveLoad must be registered via register_catalog_v1"
        );
    }

    #[test]
    fn saturation_switching_only_is_below_100() {
        // 30 switches in 15 min (2/min) with no calendar / notifications.
        // Old behaviour: 2 × 50 = 100 → CognitiveLoad pinned at 100.
        let batch: Vec<Observation> = (0..=30i64)
            .map(|i| ctx_obs(500 + i as u128, 900 + i * 30, if i % 2 == 0 { "a" } else { "b" }))
            .collect();
        let feat = last_cognitive(&batch);
        let v = match feat.value {
            FeatureValue::Scalar(v) => v,
            _ => panic!("scalar"),
        };
        assert!(v > 60.0 && v < 90.0, "got {v}");
        // Only one of three inputs present → confidence must be clearly partial.
        assert_eq!(feat.factors.len(), 1);
        assert!(feat.confidence.get() < 0.5, "confidence {:?}", feat.confidence);
    }

    #[test]
    fn saturation_extreme_switching_stays_bounded() {
        let batch: Vec<Observation> = (0..=300i64)
            .map(|i| ctx_obs(900 + i as u128, 900 + i * 3, if i % 2 == 0 { "a" } else { "b" }))
            .collect();
        let feat = last_cognitive(&batch);
        match feat.value {
            FeatureValue::Scalar(v) => assert!((0.0..=100.0).contains(&v), "got {v}"),
            _ => panic!("scalar"),
        }
    }
}
