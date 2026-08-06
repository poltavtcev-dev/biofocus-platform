//! macOS context collectors — active window + opt-in input aggregates + Calendar.
//!
//! Active window: metadata only (`bundle_id`, `app_name`).
//! Input aggregates: key-down **counts/rates** only — never characters / key codes
//! that reconstruct text. Opt-in via `BIOFOCUS_INPUT_AGGREGATES=1`.
//! Calendar: local ICS → `calendar_event` Observations (no titles/bodies). Opt-in via
//! `BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS=/path/to/file.ics`.

#![cfg_attr(not(target_os = "macos"), forbid(unsafe_code))]
// objc2 / CoreGraphics bindings require `unsafe` only inside macOS probe modules.

mod calendar_plugin;
mod calendar_probe;
mod calendar_stream;
mod error;
mod ics;
mod input_plugin;
mod input_probe;
mod input_stream;
mod payload;
mod plugin;
mod probe;
mod stream;

pub use calendar_plugin::{
    calendar_enabled, calendar_ics_path_from_env, CalendarPlugin, CALENDAR_ENABLE_ENV,
    CALENDAR_ICS_ENV, DEFAULT_CALENDAR_POLL_INTERVAL,
};
pub use calendar_probe::{CalendarEvent, CalendarProbe, IcsFileCalendarProbe, ScriptedCalendarProbe};
pub use calendar_stream::{
    observation_from_calendar_event, spawn_calendar_loop, CalendarHandle, DEFAULT_HORIZON_FUTURE,
    DEFAULT_HORIZON_PAST,
};
pub use error::{CollectorError, CollectorResult};
pub use ics::{parse_ics_events, parse_ics_file};
pub use input_plugin::{
    input_aggregates_enabled, KeystrokeAggregatePlugin, DEFAULT_AGGREGATE_WINDOW, ENABLE_ENV,
};
pub use input_probe::{
    InputCountProbe, ScriptedInputProbe, SharedKeyCounter, SystemInputProbe,
};
pub use input_stream::{
    observation_from_aggregate, spawn_keystroke_aggregate_loop, KeystrokeAggregateHandle,
};
pub use payload::{
    calendar_event_payload, context_window_payload, keystrokes_payload, CALENDAR_EVENT_DATA_TYPE,
    CONTEXT_WINDOW_DATA_TYPE, KEYSTROKES_DATA_TYPE, MACOS_CALENDAR_PROVIDER_ID,
    MACOS_CONTEXT_PROVIDER_ID, MACOS_INPUT_PROVIDER_ID,
};
pub use plugin::{ActiveWindowPlugin, DEFAULT_POLL_INTERVAL};
pub use probe::{FrontmostApp, FrontmostProbe, SystemFrontmostProbe};
pub use stream::{observation_from_frontmost, spawn_active_window_loop, ActiveWindowHandle};

/// Crate identity.
pub const CRATE_NAME: &str = "macos-collector";
