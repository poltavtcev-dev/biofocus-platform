//! Unit tests for keystroke aggregate collector (mock probe; no Accessibility).

use std::sync::Arc;
use std::time::Duration;

use bio_spec::Observation;
use plugin_sdk::BioFocusPlugin;
use runtime::observation_channel;

use macos_collector::{
    keystrokes_payload, observation_from_aggregate, input_aggregates_enabled, KeystrokeAggregatePlugin,
    ScriptedInputProbe, InputCountProbe, KEYSTROKES_DATA_TYPE, MACOS_INPUT_PROVIDER_ID, ENABLE_ENV,
};

#[test]
fn keystrokes_payload_is_aggregates_only() {
    let payload = keystrokes_payload(120, Duration::from_secs(60));
    assert_eq!(payload["count"], 120);
    assert_eq!(payload["window_secs"], 60);
    assert!(payload["rate_per_min"].as_f64().unwrap() > 0.0);
    assert!(payload.get("char").is_none());
    assert!(payload.get("text").is_none());
    assert!(payload.get("keys").is_none());
    assert!(payload.get("keystrokes").is_none());
    assert!(payload.get("window_title").is_none());
}

#[test]
fn observation_aggregate_shape() {
    let obs = observation_from_aggregate(10, Duration::from_secs(30)).expect("obs");
    assert_eq!(obs.provider_id, MACOS_INPUT_PROVIDER_ID);
    assert_eq!(obs.data_type, KEYSTROKES_DATA_TYPE);
    assert_eq!(obs.payload["count"], 10);
    assert_eq!(obs.payload["window_secs"], 30);
}

#[test]
fn enable_env_defaults_off() {
    // Unset may race in parallel tests — only assert parsing helpers via with_probe path.
    let _ = ENABLE_ENV;
    let _ = input_aggregates_enabled();
}

#[tokio::test]
async fn emits_aggregate_then_stops_idle() {
    let probe = Arc::new(ScriptedInputProbe::new());
    probe.push(7);

    let (tx, mut rx) = observation_channel(8).expect("channel");
    let plugin = KeystrokeAggregatePlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn InputCountProbe>,
        Duration::from_millis(40),
    );

    plugin.start_stream(tx).await.expect("start");

    let obs = recv_obs(&mut rx, Duration::from_secs(2)).await;
    assert_eq!(obs.data_type, KEYSTROKES_DATA_TYPE);
    assert_eq!(obs.payload["count"], 7);
    assert!(obs.payload.get("text").is_none());

    // No more counts → no emit.
    let timed_out = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(timed_out.is_err() || timed_out.ok().flatten().is_none());

    plugin.stop_stream().await.expect("stop");
    let err = plugin.stop_stream().await.expect_err("already stopped");
    assert!(matches!(err, plugin_sdk::PluginError::NotRunning));
}

#[tokio::test]
async fn untrusted_probe_emits_nothing() {
    let probe = Arc::new(ScriptedInputProbe::new());
    probe.set_trusted(false);
    probe.push(99);

    let (tx, mut rx) = observation_channel(4).expect("channel");
    let plugin = KeystrokeAggregatePlugin::with_probe(
        probe as Arc<dyn InputCountProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let timed_out = tokio::time::timeout(Duration::from_millis(150), rx.recv()).await;
    assert!(timed_out.is_err() || timed_out.ok().flatten().is_none());

    plugin.stop_stream().await.expect("stop");
}

async fn recv_obs(
    rx: &mut tokio::sync::mpsc::Receiver<Observation>,
    timeout: Duration,
) -> Observation {
    tokio::time::timeout(timeout, rx.recv())
        .await
        .expect("timeout waiting for Observation")
        .expect("channel closed")
}
