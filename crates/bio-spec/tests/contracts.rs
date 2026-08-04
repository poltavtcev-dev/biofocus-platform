//! Unit tests for `bio-spec` contracts.

use serde_json::json;
use uuid::Uuid;

use bio_spec::{
    Confidence, EvidenceRef, Feature, FeatureValue, Insight, Observation, Severity, Signal,
    SpecError, TimeWindow, UnixTimestamp,
};

/// Sample Observation JSON from `docs/07-contracts.md`.
const CONTRACT_OBSERVATION_JSON: &str = r#"{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.applehealth",
  "data_type": "heart_rate",
  "payload": {
    "bpm": 74.0,
    "source": "Apple Watch Series 9"
  },
  "confidence": 0.98
}"#;

#[test]
fn observation_contract_json_round_trip() {
    let parsed: Observation = serde_json::from_str(CONTRACT_OBSERVATION_JSON)
        .expect("contract sample must deserialize");

    assert_eq!(
        parsed.id,
        Uuid::parse_str("0190ecb5-7c2a-7123-8901-23456789abcd").expect("valid uuid")
    );
    assert_eq!(parsed.timestamp.as_secs(), 1_721_990_400);
    assert_eq!(parsed.provider_id, "com.biofocus.applehealth");
    assert_eq!(parsed.data_type, "heart_rate");
    assert_eq!(parsed.payload["bpm"], json!(74.0));
    assert!((parsed.confidence.get() - 0.98).abs() < f64::EPSILON);

    let encoded = serde_json::to_value(&parsed).expect("serialize");
    let again: Observation = serde_json::from_value(encoded).expect("re-deserialize");
    assert_eq!(again, parsed);
}

#[test]
fn observation_try_new_rejects_invalid_confidence() {
    let err = Observation::try_new(
        Uuid::nil(),
        UnixTimestamp::from_secs(0),
        "com.biofocus.test",
        "heart_rate",
        json!({}),
        1.5,
    )
    .expect_err("confidence > 1.0 must fail");

    assert_eq!(err, SpecError::InvalidConfidence(1.5));
}

#[test]
fn confidence_bounds() {
    assert!(Confidence::try_new(0.0).is_ok());
    assert!(Confidence::try_new(1.0).is_ok());
    assert!(Confidence::try_new(-0.01).is_err());
    assert!(Confidence::try_new(f64::NAN).is_err());
}

#[test]
fn time_window_rejects_inverted_range() {
    let err = TimeWindow::try_new(UnixTimestamp::from_secs(10), UnixTimestamp::from_secs(5))
        .expect_err("end < start");
    assert!(matches!(
        err,
        SpecError::InvalidTimeWindow { start: 10, end: 5 }
    ));
}

#[test]
fn signal_feature_insight_serde_smoke() {
    let signal = Signal {
        id: Uuid::nil(),
        signal_type: "HR_Spike".into(),
        timestamp_start: UnixTimestamp::from_secs(100),
        timestamp_end: UnixTimestamp::from_secs(120),
        severity: Severity::High,
    };
    let signal_json = serde_json::to_value(&signal).expect("signal serialize");
    assert_eq!(signal_json["type"], "HR_Spike");
    let signal_back: Signal = serde_json::from_value(signal_json).expect("signal deserialize");
    assert_eq!(signal_back, signal);

    let window = TimeWindow::try_new(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(900))
        .expect("valid window");
    let feature = Feature {
        feature_id: "FocusScore".into(),
        time_window: window,
        value: FeatureValue::Scalar(72.5),
        provenance: vec![Uuid::nil()],
    };
    let feature_back: Feature =
        serde_json::from_str(&serde_json::to_string(&feature).expect("feature serialize"))
            .expect("feature deserialize");
    assert_eq!(feature_back, feature);

    let insight = Insight {
        id: Uuid::nil(),
        title: "Morning focus dip".into(),
        description: "FocusScore fell while context switches rose.".into(),
        category: "focus".into(),
        evidence_list: vec![
            EvidenceRef::Feature("FocusScore".into()),
            EvidenceRef::Signal(Uuid::nil()),
        ],
        action_recommendation: Some("Take a short break.".into()),
    };
    let insight_back: Insight =
        serde_json::from_str(&serde_json::to_string(&insight).expect("insight serialize"))
            .expect("insight deserialize");
    assert_eq!(insight_back, insight);
}
