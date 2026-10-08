//! Shape lock for ADR-030 fixtures. Does not validate new payloads — P1 does that.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wearables")
        .join(name);
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("read {}: {err}", path.display())
    });
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse {name}: {err}"))
}

fn observations_in_samples(doc: &Value) -> Vec<&Value> {
    doc["samples"]
        .as_array()
        .unwrap_or_else(|| panic!("samples array"))
        .iter()
        .map(|sample| &sample["observation"])
        .collect()
}

fn assert_no_personal_name(value: &Value) {
    let Some(obj) = value.as_object() else {
        return;
    };
    for key in ["name", "device_name", "source_name"] {
        assert!(
            !obj.contains_key(key),
            "personal name key `{key}` must not be stored"
        );
    }
    if let Some(src) = obj.get("src") {
        assert_no_personal_name(src);
    }
}

fn assert_observation(obs: &Value) {
    for key in ["id", "timestamp", "provider_id", "data_type", "payload", "confidence"] {
        assert!(obs.get(key).is_some(), "missing {key}");
    }
    assert_eq!(obs["provider_id"], "com.biofocus.applehealth");
    assert_no_personal_name(&obs["payload"]);
}

#[test]
fn watch_fixture_has_sdnn_and_sleep_stages() {
    let doc = fixture("apple_watch.json");
    assert_eq!(doc["schema"], "biofocus.wearables.fixture.v2");
    assert_eq!(doc["synthetic"], true);
    let obs = observations_in_samples(&doc);
    assert!(!obs.is_empty());
    for item in &obs {
        assert_observation(item);
        assert_eq!(item["payload"]["src"]["kind"], "apple_watch");
    }
    assert!(obs.iter().any(|o| o["data_type"] == "hrv" && o["payload"]["method"] == "sdnn"));
    assert!(obs.iter().any(|o| o["payload"]["stage"] == "asleep_deep"));
    assert!(obs.iter().any(|o| o["payload"]["stage"] == "asleep_rem"));
    assert!(obs.iter().any(|o| o["data_type"] == "oxygen_saturation"));
}

#[test]
fn xiaomi_and_zepp_fixtures_omit_hrv_and_stages() {
    for (file, kind) in [
        ("xiaomi_mi_fitness.json", "xiaomi_mi_fitness"),
        ("zepp_life.json", "zepp_life"),
    ] {
        let doc = fixture(file);
        let obs = observations_in_samples(&doc);
        for item in &obs {
            assert_observation(item);
            assert_eq!(item["payload"]["src"]["kind"], kind);
            assert_ne!(item["data_type"], "hrv");
            let stage = item["payload"]["stage"].as_str().unwrap_or("");
            assert!(
                stage != "asleep_deep" && stage != "asleep_rem" && stage != "asleep_core",
                "{file} must not invent sleep stages"
            );
        }
    }
}

#[test]
fn late_write_is_behind_the_date_anchor_and_deletion_points_at_watch() {
    let doc = fixture("late_write_and_deletion.json");
    let late = &doc["late_write"];
    let start = late["sample_start"].as_i64().unwrap();
    let anchor = late["date_anchor_unix"].as_i64().unwrap();
    assert!(start < anchor);
    assert_eq!(late["expect_dated_predicate"], "skipped");
    assert_eq!(late["expect_anchored_query"], "delivered");
    assert_observation(&late["observation"]);

    let deletion = &doc["deletion"]["observation"];
    assert_observation(deletion);
    assert_eq!(deletion["data_type"], "source_deletion");
    let target = deletion["payload"]["target_id"].as_str().unwrap();

    let watch = fixture("apple_watch.json");
    let found = observations_in_samples(&watch)
        .into_iter()
        .any(|obs| obs["id"] == target);
    assert!(found, "deletion target must be a Watch sample id");
}

#[test]
fn legacy_payloads_keep_pre_v2_shapes() {
    let doc = fixture("legacy_payloads.json");
    let obs = doc["observations"].as_array().unwrap();
    assert!(obs.iter().any(|o| {
        o["data_type"] == "hrv" && o["payload"].get("method").is_none() && o["payload"].get("src").is_none()
    }));
    assert!(obs.iter().any(|o| o["data_type"] == "heart_rate" && o["payload"].get("src").is_none()));
    assert!(obs.iter().any(|o| o["payload"]["stage"] == "asleep"));
    for item in obs {
        assert_observation(item);
    }
}
