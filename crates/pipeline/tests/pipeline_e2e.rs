//! Pipeline E2E (P3-E2-T4): fixture / mock Observation stream → quality pipeline
//! → Feature Engine catalog v1 → asserted Features / Signals.
//!
//! See `docs/11-testing.md` §5 and `docs/12-development.md` § Pipeline E2E.

use std::path::PathBuf;

use bio_spec::{FeatureValue, Observation, Severity};
use feature_engine::{
    CONTEXT_SWITCH_RATE_ID, FATIGUE_INDEX_ID, FOCUS_SCORE_ID, FeatureEngine,
    HIGH_STRESS_MIN_DURATION_SECS, HIGH_STRESS_SIGNAL_TYPE, HIGH_STRESS_THRESHOLD, STRESS_INDEX_ID,
    register_catalog_v1,
};
use pipeline::{DedupeState, PipelineStage, run_quality_pipeline};
use serde_json::json;
use uuid::Uuid;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> Vec<Observation> {
    let path = fixture_path(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()))
}

fn run_e2e(raw: Vec<Observation>) -> (Vec<Observation>, feature_engine::EngineOutput) {
    let mut dedupe = DedupeState::new();
    let normalized = run_quality_pipeline(raw, &mut dedupe).expect("quality pipeline");
    assert_eq!(normalized.stage(), PipelineStage::Normalized);

    let observations = normalized.into_observations();
    let mut engine = FeatureEngine::new();
    register_catalog_v1(&mut engine).expect("register_catalog_v1");
    let output = engine.run(&observations).expect("feature engine");
    (observations, output)
}

fn scalar_features<'a>(
    output: &'a feature_engine::EngineOutput,
    feature_id: &str,
) -> Vec<&'a bio_spec::Feature> {
    output
        .features
        .iter()
        .filter(|f| f.feature_id == feature_id)
        .collect()
}

fn last_scalar(output: &feature_engine::EngineOutput, feature_id: &str) -> f64 {
    let features = scalar_features(output, feature_id);
    let last = features
        .last()
        .unwrap_or_else(|| panic!("expected Feature {feature_id}"));
    match last.value {
        FeatureValue::Scalar(v) => v,
        _ => panic!("expected scalar for {feature_id}"),
    }
}

/// Happy path: fixture JSON (alias payloads + duplicate + unknown type) →
/// normalized Observations → catalog Features + `High_Stress` Signal.
#[test]
fn e2e_fixture_yields_features_and_high_stress() {
    let raw = load_fixture("e2e_happy_path.json");
    // 15 rows in file; one duplicate id dropped by dedupe → 14 kept.
    assert_eq!(raw.len(), 15);

    let (observations, output) = run_e2e(raw);

    assert_eq!(
        observations.len(),
        14,
        "dedupe should drop the duplicate HRV id"
    );

    // Normalize: HRV alias `rmssd` → `rmssd_ms`; context aliases; HR `hr` → `bpm`.
    let hrv = observations
        .iter()
        .find(|o| o.data_type == "hrv")
        .expect("hrv");
    assert!(hrv.payload.get("rmssd_ms").is_some());
    assert!(hrv.payload.get("rmssd").is_none());

    let ctx = observations
        .iter()
        .find(|o| o.data_type == "context_window")
        .expect("context");
    assert_eq!(ctx.payload["bundle_id"], json!("com.dev.ide"));
    assert_eq!(ctx.payload["app_name"], json!("IDE"));

    let hr = observations
        .iter()
        .find(|o| o.data_type == "heart_rate")
        .expect("hr");
    assert_eq!(hr.payload["bpm"], json!(62.0));

    let unknown = observations
        .iter()
        .find(|o| o.data_type == "custom.debug")
        .expect("unknown type must pass through");
    assert_eq!(unknown.payload["note"], json!("unknown-type-pass-through"));

    // Catalog Features present with provenance.
    assert!(!scalar_features(&output, CONTEXT_SWITCH_RATE_ID).is_empty());
    assert!(!scalar_features(&output, FOCUS_SCORE_ID).is_empty());
    assert!(!scalar_features(&output, STRESS_INDEX_ID).is_empty());
    assert!(!scalar_features(&output, FATIGUE_INDEX_ID).is_empty());

    let stress = last_scalar(&output, STRESS_INDEX_ID);
    assert!(
        stress > HIGH_STRESS_THRESHOLD,
        "expected elevated StressIndex, got {stress}"
    );
    assert!(
        (stress - 100.0).abs() < 1e-6,
        "RMSSD 15 → stress 100, got {stress}"
    );

    let stress_feats = scalar_features(&output, STRESS_INDEX_ID);
    let stress_feat = stress_feats.last().expect("stress feat");
    assert!(
        !stress_feat.provenance.is_empty(),
        "StressIndex must carry Observation provenance"
    );

    // High_Stress: contiguous StressIndex > 75 for longer than 5 minutes.
    assert_eq!(output.signals.len(), 1, "expected one High_Stress signal");
    let sig = &output.signals[0];
    assert_eq!(sig.signal_type, HIGH_STRESS_SIGNAL_TYPE);
    assert_eq!(sig.severity, Severity::High);
    assert!(
        sig.timestamp_end.as_secs() - sig.timestamp_start.as_secs() > HIGH_STRESS_MIN_DURATION_SECS
    );
}

/// Edge: empty Observation batch → idle-friendly empty Features / Signals.
#[test]
fn e2e_empty_batch_is_idle_ok() {
    let (observations, output) = run_e2e(Vec::new());
    assert!(observations.is_empty());
    assert!(output.is_empty());
}

/// Edge: high RMSSD (low stress) → StressIndex Features may exist, but no High_Stress.
#[test]
fn e2e_no_high_stress_below_threshold() {
    let mut raw = Vec::new();
    for (i, ts) in (900..=1260).step_by(60).enumerate() {
        raw.push(
            Observation::try_new(
                Uuid::from_u128(500 + i as u128),
                bio_spec::UnixTimestamp::from_secs(ts),
                "com.biofocus.e2e",
                "hrv",
                // Alias path: `hrv_ms` → normalize → `rmssd_ms` 70 → stress ~0.
                json!({ "hrv_ms": 70.0 }),
                1.0,
            )
            .expect("obs"),
        );
    }

    let (_observations, output) = run_e2e(raw);
    assert!(!scalar_features(&output, STRESS_INDEX_ID).is_empty());
    let stress = last_scalar(&output, STRESS_INDEX_ID);
    assert!(
        stress <= HIGH_STRESS_THRESHOLD,
        "expected StressIndex at/below threshold, got {stress}"
    );
    assert!(
        output
            .signals
            .iter()
            .all(|s| s.signal_type != HIGH_STRESS_SIGNAL_TYPE),
        "High_Stress must not emit below threshold, got {:?}",
        output.signals
    );
}
