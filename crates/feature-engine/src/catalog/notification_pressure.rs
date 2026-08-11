//! `NotificationPressure` v1 — calm interruption intensity from notification
//! events (`docs/06-feature-catalog.md` / ADR-019 / P18-E3).
//!
//! # v1 formula (documented)
//!
//! - **Window / step:** 15 minutes / 1 minute (aligned with Focus catalog;
//!   series may coarsen).
//! - **Inputs (required):** `notification_event` Observations with usable
//!   `count` ≥ 1 in the window.
//! - **Value (0–100):** `sum(count)` over usable events → linear intensity map
//!   `clamp(100 * sum / 20, 0, 100)` (saturation = **20** deliveries in the
//!   15m window → 100).
//! - **Omit policy:** empty window / no usable `notification_event` → **omit**.
//! - **Confidence (ADR-007):** single family (`notification_event`); when
//!   emitted `confidence = mean(evidence Observation.confidence)`.
//! - **Explanation factors:** when optional labels present — prefer `category`
//!   count-shares; else `interruption_level`; else `app_kind`. Shares sum to
//!   1.0 when any factors emit. No body/title content ever.
//! - Calm framing only: “interruption intensity in this window” — **not**
//!   “you are overloaded” / clinical ADHD / workplace productivity scoring.

use bio_spec::{
    ExplanationFactor, Feature, FeatureValue, Observation, TimeWindow,
    DATA_TYPE_NOTIFICATION_EVENT, INTERRUPTION_LEVEL_ACTIVE, INTERRUPTION_LEVEL_CRITICAL,
    INTERRUPTION_LEVEL_PASSIVE, INTERRUPTION_LEVEL_TIME_SENSITIVE, INTERRUPTION_LEVEL_UNKNOWN,
    NOTIFICATION_APP_KIND_CALENDAR, NOTIFICATION_APP_KIND_MAIL, NOTIFICATION_APP_KIND_MESSAGING,
    NOTIFICATION_APP_KIND_OTHER, NOTIFICATION_APP_KIND_SOCIAL, NOTIFICATION_APP_KIND_SYSTEM,
    NOTIFICATION_APP_KIND_UNKNOWN, NOTIFICATION_CATEGORY_CALENDAR,
    NOTIFICATION_CATEGORY_COMMUNICATION, NOTIFICATION_CATEGORY_MEDIA, NOTIFICATION_CATEGORY_OTHER,
    NOTIFICATION_CATEGORY_SOCIAL, NOTIFICATION_CATEGORY_SYSTEM, NOTIFICATION_CATEGORY_UNKNOWN,
};

use crate::catalog::confidence::single_family_confidence;
use crate::catalog::window::{
    in_window, sliding_window_ends_for, snapshot_time_span, window_ending_at,
};
use crate::{ComputeContext, FeatureEngineResult, FeatureNode, NodeId, NodeOutput};

/// Stable Feature / node id (`docs/06-feature-catalog.md`).
pub const FEATURE_ID: &str = "NotificationPressure";

/// Sum of `count` in a 15m window that maps to intensity 100.
pub const SATURATION_COUNT: f64 = 20.0;

/// DAG node computing [`FEATURE_ID`] (independent; no upstream Feature deps).
#[derive(Debug, Clone, Default)]
pub struct NotificationPressureNode;

impl NotificationPressureNode {
    /// Constructs the catalog node.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl FeatureNode for NotificationPressureNode {
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
            if let Some(feature) = score_window(ctx, &window) {
                features.push(feature);
            }
        }

        Ok(NodeOutput::features(features))
    }
}

fn score_window(ctx: &ComputeContext<'_>, window: &TimeWindow) -> Option<Feature> {
    let mut samples: Vec<&Observation> = ctx
        .observations()
        .iter()
        .filter(|o| o.data_type == DATA_TYPE_NOTIFICATION_EVENT && in_window(o, window))
        .collect();
    samples.sort_by_key(|o| o.timestamp.as_secs());

    let usable: Vec<&Observation> = samples
        .iter()
        .copied()
        .filter(|o| event_count(o) >= 1)
        .collect();

    if usable.is_empty() {
        return None;
    }

    let sum_count = usable.iter().map(|o| event_count(o) as f64).sum::<f64>();
    let value = intensity_from_sum(sum_count);

    let factors = label_count_factors(&usable);
    let provenance: Vec<_> = usable.iter().map(|o| o.id).collect();
    let confidence = single_family_confidence(&usable);

    Some(Feature {
        feature_id: FEATURE_ID.to_owned(),
        time_window: *window,
        value: FeatureValue::Scalar(value),
        provenance,
        confidence,
        factors,
    })
}

fn intensity_from_sum(sum_count: f64) -> f64 {
    if !sum_count.is_finite() || sum_count <= 0.0 {
        return 0.0;
    }
    (100.0 * sum_count / SATURATION_COUNT).clamp(0.0, 100.0)
}

fn event_count(obs: &Observation) -> u64 {
    match obs.payload.get("count") {
        Some(v) => v
            .as_u64()
            .or_else(|| {
                v.as_i64()
                    .and_then(|i| u64::try_from(i).ok())
                    .or_else(|| {
                        v.as_f64()
                            .filter(|f| f.is_finite() && *f >= 1.0 && f.fract() == 0.0)
                            .map(|f| f as u64)
                    })
            })
            .filter(|n| *n >= 1)
            .unwrap_or(0),
        None => 0,
    }
}

fn payload_str<'a>(obs: &'a Observation, key: &str) -> Option<&'a str> {
    obs.payload.get(key).and_then(|v| v.as_str())
}

fn label_count_factors(samples: &[&Observation]) -> Vec<ExplanationFactor> {
    if let Some(factors) = shares_for_key(samples, "category", category_label) {
        return factors;
    }
    if let Some(factors) = shares_for_key(samples, "interruption_level", interruption_label) {
        return factors;
    }
    if let Some(factors) = shares_for_key(samples, "app_kind", app_kind_label) {
        return factors;
    }
    Vec::new()
}

fn shares_for_key(
    samples: &[&Observation],
    key: &str,
    label_for: fn(&str) -> Option<&'static str>,
) -> Option<Vec<ExplanationFactor>> {
    let mut buckets: Vec<(&str, &str, u64)> = Vec::new();
    for o in samples {
        let Some(raw) = payload_str(o, key) else {
            continue;
        };
        let Some(label) = label_for(raw) else {
            continue;
        };
        let n = event_count(o);
        if n == 0 {
            continue;
        }
        if let Some(entry) = buckets.iter_mut().find(|(id, _, _)| *id == raw) {
            entry.2 += n;
        } else {
            buckets.push((raw, label, n));
        }
    }
    if buckets.is_empty() {
        return None;
    }
    let total: u64 = buckets.iter().map(|(_, _, n)| *n).sum();
    if total == 0 {
        return None;
    }
    Some(
        buckets
            .into_iter()
            .map(|(id, label, n)| ExplanationFactor {
                id: id.to_owned(),
                label: label.to_owned(),
                share: n as f64 / total as f64,
            })
            .collect(),
    )
}

fn category_label(raw: &str) -> Option<&'static str> {
    match raw {
        NOTIFICATION_CATEGORY_COMMUNICATION => Some("Communication"),
        NOTIFICATION_CATEGORY_CALENDAR => Some("Calendar"),
        NOTIFICATION_CATEGORY_SYSTEM => Some("System"),
        NOTIFICATION_CATEGORY_MEDIA => Some("Media"),
        NOTIFICATION_CATEGORY_SOCIAL => Some("Social"),
        NOTIFICATION_CATEGORY_OTHER => Some("Other"),
        NOTIFICATION_CATEGORY_UNKNOWN => Some("Unknown category"),
        _ => None,
    }
}

fn interruption_label(raw: &str) -> Option<&'static str> {
    match raw {
        INTERRUPTION_LEVEL_PASSIVE => Some("Passive"),
        INTERRUPTION_LEVEL_ACTIVE => Some("Active"),
        INTERRUPTION_LEVEL_TIME_SENSITIVE => Some("Time-sensitive"),
        INTERRUPTION_LEVEL_CRITICAL => Some("Critical"),
        INTERRUPTION_LEVEL_UNKNOWN => Some("Unknown interruption"),
        _ => None,
    }
}

fn app_kind_label(raw: &str) -> Option<&'static str> {
    match raw {
        NOTIFICATION_APP_KIND_MESSAGING => Some("Messaging"),
        NOTIFICATION_APP_KIND_MAIL => Some("Mail"),
        NOTIFICATION_APP_KIND_CALENDAR => Some("Calendar apps"),
        NOTIFICATION_APP_KIND_SOCIAL => Some("Social apps"),
        NOTIFICATION_APP_KIND_SYSTEM => Some("System apps"),
        NOTIFICATION_APP_KIND_OTHER => Some("Other apps"),
        NOTIFICATION_APP_KIND_UNKNOWN => Some("Unknown apps"),
        _ => None,
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

    fn notif_obs_cat(id: u128, ts: i64, count: u64, category: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.notifications",
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": count, "category": category }),
            1.0,
        )
        .expect("obs")
    }

    fn notif_obs_level(id: u128, ts: i64, count: u64, level: &str) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.macos.notifications",
            DATA_TYPE_NOTIFICATION_EVENT,
            json!({ "count": count, "interruption_level": level }),
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

    fn last_pressure(batch: &[Observation]) -> bio_spec::Feature {
        let mut engine = FeatureEngine::new();
        engine
            .register(NotificationPressureNode::new())
            .expect("reg");
        let out = engine.run(batch).expect("run");
        out.features
            .into_iter()
            .rev()
            .find(|f| f.feature_id == FEATURE_ID)
            .expect("NotificationPressure")
    }

    #[test]
    fn empty_snapshot_omits_feature() {
        let mut engine = FeatureEngine::new();
        engine
            .register(NotificationPressureNode::new())
            .expect("reg");
        let out = engine.run(&[]).expect("run");
        assert!(out.features.is_empty());
    }

    #[test]
    fn rich_events_map_sum_to_intensity() {
        // sum count = 4 → 100 * 4/20 = 20
        let feat = last_pressure(&[
            notif_obs_cat(1, 1500, 1, "communication"),
            notif_obs_cat(2, 1800, 3, "system"),
        ]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 20.0).abs() < 1e-9, "expected 20 intensity, got {v}");
        assert!((feat.confidence.get() - 1.0).abs() < 1e-12);
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        assert!(feat.factors.iter().all(|f| {
            f.id != "title" && f.id != "body" && !f.label.to_lowercase().contains("message body")
        }));
    }

    #[test]
    fn saturation_clamps_at_100() {
        let feat = last_pressure(&[notif_obs(1, 1500, 25)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 100.0).abs() < 1e-9);
    }

    #[test]
    fn interruption_level_factors_when_no_category() {
        let feat = last_pressure(&[
            notif_obs_level(1, 1500, 1, "active"),
            notif_obs_level(2, 1800, 1, "passive"),
        ]);
        assert_eq!(feat.factors.len(), 2);
        let share_sum: f64 = feat.factors.iter().map(|f| f.share).sum();
        assert!((share_sum - 1.0).abs() < 1e-12);
        assert!(feat.factors.iter().any(|f| f.id == "active"));
        assert!(feat.factors.iter().any(|f| f.id == "passive"));
    }

    #[test]
    fn count_only_emits_without_factors() {
        let feat = last_pressure(&[notif_obs(1, 1500, 2)]);
        let FeatureValue::Scalar(v) = feat.value else {
            panic!("scalar");
        };
        assert!((v - 10.0).abs() < 1e-9);
        assert!(feat.factors.is_empty());
    }

    #[test]
    fn low_observation_confidence_lowers_feature() {
        let feat = last_pressure(&[
            notif_obs_conf(1, 1500, 1, 0.4),
            notif_obs_conf(2, 1800, 1, 0.4),
        ]);
        assert!((feat.confidence.get() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn register_catalog_v1_includes_notification_pressure() {
        let batch = vec![notif_obs(1, 1500, 1), notif_obs(2, 1800, 1)];
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("catalog");
        let out = engine.run(&batch).expect("run");
        assert!(
            out.features.iter().any(|f| f.feature_id == FEATURE_ID),
            "NotificationPressure must be registered via register_catalog_v1"
        );
    }
}
