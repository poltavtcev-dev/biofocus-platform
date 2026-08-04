//! Observation payload helpers for macOS context collectors.

use std::time::Duration;

use serde_json::{json, Value as JsonValue};

use crate::probe::FrontmostApp;

/// Provider id for the macOS active-window collector.
pub const MACOS_CONTEXT_PROVIDER_ID: &str = "com.biofocus.macos.context";

/// Provider id for the macOS input-aggregate collector.
pub const MACOS_INPUT_PROVIDER_ID: &str = "com.biofocus.macos.input";

/// Storage / schema `data_type` for active window context.
pub const CONTEXT_WINDOW_DATA_TYPE: &str = "context_window";

/// Storage / schema `data_type` for keystroke aggregates.
pub const KEYSTROKES_DATA_TYPE: &str = "keystrokes";

/// Builds privacy-safe payload: app identity only (no window title / content).
#[must_use]
pub fn context_window_payload(app: &FrontmostApp) -> JsonValue {
    json!({
        "bundle_id": app.bundle_id,
        "app_name": app.app_name,
    })
}

/// Builds privacy-safe keystroke aggregate payload (counts / rates only).
#[must_use]
pub fn keystrokes_payload(count: u64, window: Duration) -> JsonValue {
    let window_secs = window.as_secs().max(1);
    let rate_per_min = (count as f64) * 60.0 / (window_secs as f64);
    json!({
        "count": count,
        "window_secs": window_secs,
        "rate_per_min": rate_per_min,
    })
}
