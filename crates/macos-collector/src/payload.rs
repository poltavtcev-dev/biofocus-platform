//! Observation payload helpers for macOS context collectors.

use std::time::Duration;

use serde_json::{json, Value as JsonValue};

use crate::calendar_probe::CalendarEvent;
use crate::probe::FrontmostApp;

/// Provider id for the macOS active-window collector.
pub const MACOS_CONTEXT_PROVIDER_ID: &str = "com.biofocus.macos.context";

/// Provider id for the macOS input-aggregate collector.
pub const MACOS_INPUT_PROVIDER_ID: &str = "com.biofocus.macos.input";

/// Provider id for the local Calendar (ICS) collector.
pub const MACOS_CALENDAR_PROVIDER_ID: &str = "com.biofocus.macos.calendar";

/// Provider id for the Browser categories collector (ADR-010).
pub const MACOS_BROWSER_PROVIDER_ID: &str = "com.biofocus.macos.browser";

/// Provider id for the Now Playing ambient collector (ADR-012).
pub const MACOS_NOW_PLAYING_PROVIDER_ID: &str = "com.biofocus.macos.now_playing";

/// Provider id for the Git activity collector (ADR-013).
pub const MACOS_GIT_PROVIDER_ID: &str = "com.biofocus.macos.git";

/// Provider id for the ambient light collector (ADR-015).
pub const MACOS_AMBIENT_LIGHT_PROVIDER_ID: &str = "com.biofocus.macos.ambient_light";

/// Storage / schema `data_type` for active window context.
pub const CONTEXT_WINDOW_DATA_TYPE: &str = "context_window";

/// Storage / schema `data_type` for keystroke aggregates.
pub const KEYSTROKES_DATA_TYPE: &str = "keystrokes";

/// Storage / schema `data_type` for Calendar / meeting events.
pub const CALENDAR_EVENT_DATA_TYPE: &str = "calendar_event";

/// Storage / schema `data_type` for Browser category context.
pub const BROWSER_CATEGORY_DATA_TYPE: &str = "browser_category";

/// Storage / schema `data_type` for Now Playing ambient context.
pub const NOW_PLAYING_DATA_TYPE: &str = "now_playing";

/// Storage / schema `data_type` for Git activity context.
pub const GIT_ACTIVITY_DATA_TYPE: &str = "git_activity";

/// Storage / schema `data_type` for ambient light context.
pub const AMBIENT_LIGHT_DATA_TYPE: &str = "ambient_light";

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

/// Builds privacy-safe calendar payload (schedule metadata only — no title/body).
#[must_use]
pub fn calendar_event_payload(event: &CalendarEvent) -> JsonValue {
    json!({
        "uid": event.uid,
        "start": event.start,
        "end": event.end,
        "all_day": event.all_day,
        "busy": event.busy,
    })
}

/// Builds privacy-safe browser category payload (coarse labels only — no URL/title).
#[must_use]
pub fn browser_category_payload(sample: &crate::browser_probe::BrowserCategorySample) -> JsonValue {
    match &sample.browser_bundle_id {
        Some(bundle) if !bundle.is_empty() => json!({
            "category": sample.category,
            "browser_bundle_id": bundle,
        }),
        _ => json!({
            "category": sample.category,
        }),
    }
}

/// Builds privacy-safe Now Playing payload (coarse kind + playing — no titles).
#[must_use]
pub fn now_playing_payload(sample: &crate::now_playing_probe::NowPlayingSample) -> JsonValue {
    json!({
        "media_kind": sample.media_kind,
        "is_playing": sample.is_playing,
    })
}

/// Builds privacy-safe Git activity payload (coarse kind + optional count — no paths).
#[must_use]
pub fn git_activity_payload(sample: &crate::git_activity_probe::GitActivitySample) -> JsonValue {
    match sample.event_count {
        Some(n) if n >= 1 => json!({
            "activity_kind": sample.activity_kind,
            "event_count": n,
        }),
        _ => json!({
            "activity_kind": sample.activity_kind,
        }),
    }
}

/// Builds privacy-safe ambient light payload (coarse kind + optional level — no frames).
#[must_use]
pub fn ambient_light_payload(sample: &crate::ambient_light_probe::AmbientLightSample) -> JsonValue {
    match sample.level {
        Some(n) if n <= 100 => json!({
            "light_kind": sample.light_kind,
            "level": n,
        }),
        _ => json!({
            "light_kind": sample.light_kind,
        }),
    }
}
