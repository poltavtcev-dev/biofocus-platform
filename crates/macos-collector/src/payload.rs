//! `context_window` Observation payload helpers.

use serde_json::{json, Value as JsonValue};

use crate::probe::FrontmostApp;

/// Provider id for the macOS context collector.
pub const MACOS_CONTEXT_PROVIDER_ID: &str = "com.biofocus.macos.context";

/// Storage / schema `data_type` for active window context.
pub const CONTEXT_WINDOW_DATA_TYPE: &str = "context_window";

/// Builds privacy-safe payload: app identity only (no window title / content).
#[must_use]
pub fn context_window_payload(app: &FrontmostApp) -> JsonValue {
    json!({
        "bundle_id": app.bundle_id,
        "app_name": app.app_name,
    })
}
