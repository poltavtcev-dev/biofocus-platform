//! Feature Worker idle / stop tests (P3-E1-T4).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bio_spec::{Observation, UnixTimestamp};
use pipeline::NormalizedBatch;
use runtime::{
    FeatureHook, FeatureWorkerConfig, ObservationSource, RuntimeError, spawn_feature_worker,
};
use serde_json::json;
use uuid::Uuid;

fn sample(n: u64, id: &str) -> Observation {
    Observation::try_new(
        Uuid::parse_str(id).expect("uuid"),
        UnixTimestamp::from_secs(n as i64),
        "com.biofocus.test",
        "heart_rate",
        json!({ "bpm": 70.0 + n as f64 }),
        1.0,
    )
    .expect("valid observation")
}

struct CountingSource {
    polls: Arc<AtomicUsize>,
    queue: Mutex<Vec<Observation>>,
}

impl ObservationSource for CountingSource {
    fn poll_new(&mut self) -> Result<Vec<Observation>, RuntimeError> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        let mut q = self
            .queue
            .lock()
            .map_err(|_| RuntimeError::FeatureWorkerSource("queue lock poisoned".into()))?;
        if q.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(std::mem::take(&mut *q))
        }
    }
}

struct CountingHook {
    calls: Arc<AtomicUsize>,
    kept: Arc<AtomicUsize>,
}

impl FeatureHook for CountingHook {
    fn on_normalized(&mut self, batch: &NormalizedBatch) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.kept.fetch_add(batch.len(), Ordering::SeqCst);
    }
}

#[test]
fn feature_worker_stop_freezes_poll_count() {
    let polls = Arc::new(AtomicUsize::new(0));
    let source = CountingSource {
        polls: Arc::clone(&polls),
        queue: Mutex::new(Vec::new()),
    };
    let hook = CountingHook {
        calls: Arc::new(AtomicUsize::new(0)),
        kept: Arc::new(AtomicUsize::new(0)),
    };

    let handle = spawn_feature_worker(
        source,
        hook,
        FeatureWorkerConfig {
            poll_interval: Duration::from_millis(40),
        },
    );

    std::thread::sleep(Duration::from_millis(150));
    assert!(
        polls.load(Ordering::SeqCst) >= 1,
        "worker should poll at least once while running"
    );

    handle.stop();
    let after_stop = polls.load(Ordering::SeqCst);

    // Several poll intervals would have fired if the loop kept running.
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        polls.load(Ordering::SeqCst),
        after_stop,
        "source must not keep polling after stop (idle freeze)"
    );
}

#[test]
fn feature_worker_processes_queued_observations_then_idles() {
    let polls = Arc::new(AtomicUsize::new(0));
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let kept = Arc::new(AtomicUsize::new(0));

    let source = CountingSource {
        polls: Arc::clone(&polls),
        queue: Mutex::new(vec![
            sample(1, "0190ecb5-7c2a-7123-8901-23456789abc1"),
            sample(2, "0190ecb5-7c2a-7123-8901-23456789abc2"),
        ]),
    };
    let hook = CountingHook {
        calls: Arc::clone(&hook_calls),
        kept: Arc::clone(&kept),
    };

    let handle = spawn_feature_worker(
        source,
        hook,
        FeatureWorkerConfig {
            poll_interval: Duration::from_millis(30),
        },
    );

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while hook_calls.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        hook_calls.load(Ordering::SeqCst) >= 1,
        "hook should run after queued Observations"
    );
    assert_eq!(kept.load(Ordering::SeqCst), 2);

    handle.stop();
    let polls_after = polls.load(Ordering::SeqCst);
    let hooks_after = hook_calls.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(polls.load(Ordering::SeqCst), polls_after);
    assert_eq!(hook_calls.load(Ordering::SeqCst), hooks_after);
}
