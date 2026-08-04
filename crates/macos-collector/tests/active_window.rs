//! Unit tests for active window collector (mock probe; no NSWorkspace).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bio_spec::Observation;
use plugin_sdk::BioFocusPlugin;
use runtime::observation_channel;

use macos_collector::{
    observation_from_frontmost, ActiveWindowPlugin, FrontmostApp, FrontmostProbe,
    CONTEXT_WINDOW_DATA_TYPE, MACOS_CONTEXT_PROVIDER_ID,
};

#[derive(Default)]
struct ScriptedProbe {
    apps: Mutex<Vec<Option<FrontmostApp>>>,
    calls: AtomicUsize,
}

impl FrontmostProbe for ScriptedProbe {
    fn frontmost(&self) -> macos_collector::CollectorResult<Option<FrontmostApp>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut guard = self.apps.lock().expect("probe lock");
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

#[test]
fn observation_payload_is_metadata_only() {
    let app = FrontmostApp {
        bundle_id: "com.apple.Terminal".into(),
        app_name: "Terminal".into(),
    };
    let obs = observation_from_frontmost(&app).expect("obs");
    assert_eq!(obs.provider_id, MACOS_CONTEXT_PROVIDER_ID);
    assert_eq!(obs.data_type, CONTEXT_WINDOW_DATA_TYPE);
    assert_eq!(obs.payload["bundle_id"], "com.apple.Terminal");
    assert_eq!(obs.payload["app_name"], "Terminal");
    assert!(obs.payload.get("window_title").is_none());
    assert!(obs.payload.get("keystrokes").is_none());
}

#[tokio::test]
async fn emits_only_on_frontmost_change_then_stops() {
    let probe = Arc::new(ScriptedProbe {
        apps: Mutex::new(vec![
            Some(FrontmostApp {
                bundle_id: "a.one".into(),
                app_name: "One".into(),
            }),
            Some(FrontmostApp {
                bundle_id: "a.one".into(),
                app_name: "One".into(),
            }),
            Some(FrontmostApp {
                bundle_id: "a.two".into(),
                app_name: "Two".into(),
            }),
        ]),
        calls: AtomicUsize::new(0),
    });

    let (tx, mut rx) = observation_channel(8).expect("channel");
    let plugin = ActiveWindowPlugin::with_probe(Arc::clone(&probe) as Arc<dyn FrontmostProbe>, Duration::from_millis(50));

    plugin.start_stream(tx).await.expect("start");

    let first = recv_obs(&mut rx, Duration::from_secs(2)).await;
    assert_eq!(first.payload["bundle_id"], "a.one");

    let second = recv_obs(&mut rx, Duration::from_secs(2)).await;
    assert_eq!(second.payload["bundle_id"], "a.two");

    // No third while script exhausted (None) — brief wait then stop.
    let timed_out = tokio::time::timeout(Duration::from_millis(120), rx.recv()).await;
    assert!(timed_out.is_err() || timed_out.ok().flatten().is_none());

    plugin.stop_stream().await.expect("stop");
    // After stop, further recv should eventually end when we drop — channel still open until tx dropped by task.
    assert!(probe.calls.load(Ordering::SeqCst) >= 2);
}

#[tokio::test]
async fn stop_without_busy_spin_when_idle() {
    let probe = Arc::new(ScriptedProbe::default());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin =
        ActiveWindowPlugin::with_probe(probe as Arc<dyn FrontmostProbe>, Duration::from_millis(80));

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(100)).await;
    plugin.stop_stream().await.expect("stop");
    // Second stop → NotRunning
    let err = plugin.stop_stream().await.expect_err("already stopped");
    assert!(matches!(err, plugin_sdk::PluginError::NotRunning));
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
