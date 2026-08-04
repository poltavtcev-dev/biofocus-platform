//! Integration: mock collectors → bounded channel → persist worker → SQLite.
//!
//! Covers emit Observation for `context_window` and opt-in `keystrokes`,
//! plus stop/idle: after `stop_stream` probe work must not keep ticking.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bio_spec::Observation;
use ingest::spawn_persist_worker;
use macos_collector::{
    ActiveWindowPlugin, FrontmostApp, FrontmostProbe, InputCountProbe, KeystrokeAggregatePlugin,
    ScriptedInputProbe, CONTEXT_WINDOW_DATA_TYPE, KEYSTROKES_DATA_TYPE,
};
use plugin_sdk::BioFocusPlugin;
use runtime::observation_channel;
use storage::{Database, ObservationRepository};

#[derive(Default)]
struct ScriptedFrontmost {
    apps: Mutex<Vec<Option<FrontmostApp>>>,
    calls: AtomicUsize,
}

impl FrontmostProbe for ScriptedFrontmost {
    fn frontmost(&self) -> macos_collector::CollectorResult<Option<FrontmostApp>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut guard = self.apps.lock().expect("probe lock");
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Wraps [`ScriptedInputProbe`] and counts `take_count` / trust checks for idle asserts.
struct CountingInputProbe {
    inner: ScriptedInputProbe,
    drains: AtomicUsize,
}

impl CountingInputProbe {
    fn new() -> Self {
        Self {
            inner: ScriptedInputProbe::new(),
            drains: AtomicUsize::new(0),
        }
    }
}

impl InputCountProbe for CountingInputProbe {
    fn take_count(&self) -> macos_collector::CollectorResult<u64> {
        self.drains.fetch_add(1, Ordering::SeqCst);
        self.inner.take_count()
    }

    fn accessibility_trusted(&self) -> bool {
        self.inner.accessibility_trusted()
    }
}

async fn wait_for_data_type(
    db_path: &std::path::Path,
    data_type: &str,
    min_count: usize,
    timeout: Duration,
) -> Vec<Observation> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        {
            let db = Database::open(db_path).expect("re-open db");
            let repo = ObservationRepository::new(&db);
            let listed = repo.list_by_data_type(data_type).expect("list");
            if listed.len() >= min_count {
                return listed;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timed out waiting for {min_count} Observation(s) of type {data_type}");
        }
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
}

#[tokio::test]
async fn active_window_emits_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(ScriptedFrontmost {
        apps: Mutex::new(vec![Some(FrontmostApp {
            bundle_id: "com.biofocus.integration".into(),
            app_name: "Integration".into(),
        })]),
        calls: AtomicUsize::new(0),
    });

    let plugin = ActiveWindowPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn FrontmostProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed = wait_for_data_type(&db_path, CONTEXT_WINDOW_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["bundle_id"], "com.biofocus.integration");
    assert_eq!(listed[0].payload["app_name"], "Integration");
    assert!(listed[0].payload.get("window_title").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn keystroke_aggregates_emit_observation_into_channel_and_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("biofocus_main.db");
    let db = Database::open(&db_path).expect("open db");

    let (tx, rx) = observation_channel(8).expect("channel");
    let worker = spawn_persist_worker(rx, db);

    let probe = Arc::new(CountingInputProbe::new());
    probe.inner.push(15);

    let plugin = KeystrokeAggregatePlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn InputCountProbe>,
        Duration::from_millis(40),
    );
    plugin.start_stream(tx).await.expect("start");

    let listed = wait_for_data_type(&db_path, KEYSTROKES_DATA_TYPE, 1, Duration::from_secs(3)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].payload["count"], 15);
    assert!(listed[0].payload.get("text").is_none());
    assert!(listed[0].payload.get("char").is_none());

    plugin.stop_stream().await.expect("stop");
    worker.join().expect("persist worker");
}

#[tokio::test]
async fn active_window_stop_halts_periodic_probe_work() {
    let probe = Arc::new(ScriptedFrontmost::default());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = ActiveWindowPlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn FrontmostProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(probe.calls.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.calls.load(Ordering::SeqCst);

    // Several poll intervals would have fired if the loop kept running.
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.calls.load(Ordering::SeqCst),
        after_stop,
        "frontmost probe must not keep polling after stop_stream (no busy-loop)"
    );
}

#[tokio::test]
async fn keystroke_stop_halts_periodic_probe_work() {
    let probe = Arc::new(CountingInputProbe::new());
    let (tx, _rx) = observation_channel(4).expect("channel");
    let plugin = KeystrokeAggregatePlugin::with_probe(
        Arc::clone(&probe) as Arc<dyn InputCountProbe>,
        Duration::from_millis(50),
    );

    plugin.start_stream(tx).await.expect("start");
    tokio::time::sleep(Duration::from_millis(180)).await;
    assert!(probe.drains.load(Ordering::SeqCst) >= 1);

    plugin.stop_stream().await.expect("stop");
    let after_stop = probe.drains.load(Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(
        probe.drains.load(Ordering::SeqCst),
        after_stop,
        "input probe must not keep draining after stop_stream (no busy-loop)"
    );
}
