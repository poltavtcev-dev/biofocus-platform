//! Integration tests for the Core runtime host.

use bio_spec::{Observation, UnixTimestamp};
use runtime::{
    CoreRuntime, DEFAULT_OBSERVATION_BUFFER, RuntimeConfig, RuntimeError, init_tracing,
    observation_channel,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn core_runtime_block_on_async_work() {
    let rt = CoreRuntime::try_new(&RuntimeConfig {
        worker_threads: Some(2),
        ..RuntimeConfig::default()
    })
    .expect("runtime builds");

    let value = rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        42_u32
    });
    assert_eq!(value, 42);
}

#[test]
fn observation_channel_is_bounded() {
    let (tx, mut rx) = observation_channel(2).expect("capacity 2 is valid");

    let mk = |n: u64| {
        Observation::try_new(
            Uuid::nil(),
            UnixTimestamp::from_secs(n as i64),
            "com.biofocus.test",
            "heart_rate",
            json!({ "n": n }),
            1.0,
        )
        .expect("valid observation")
    };

    tx.try_send(mk(1)).expect("slot 1");
    tx.try_send(mk(2)).expect("slot 2");
    let err = tx.try_send(mk(3)).expect_err("channel must be full");
    assert!(matches!(
        err,
        tokio::sync::mpsc::error::TrySendError::Full(_)
    ));

    let first = rx.try_recv().expect("queued observation");
    assert_eq!(first.timestamp.as_secs(), 1);
}

#[test]
fn observation_channel_rejects_zero_capacity() {
    let err = observation_channel(0).expect_err("zero capacity");
    assert!(matches!(err, RuntimeError::InvalidChannelCapacity));
}

#[test]
fn default_buffer_constant_is_positive() {
    assert!(DEFAULT_OBSERVATION_BUFFER > 0);
}

#[test]
fn init_tracing_is_idempotent() {
    init_tracing(Some("warn")).expect("first init");
    init_tracing(Some("info")).expect("second init must not fail");
}
