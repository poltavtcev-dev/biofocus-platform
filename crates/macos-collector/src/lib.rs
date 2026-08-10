//! macOS context collectors — active window + opt-in input aggregates + Calendar
//! + opt-in Browser categories + opt-in Now Playing ambient + opt-in Git activity.
//!
//! Active window: metadata only (`bundle_id`, `app_name`).
//! Input aggregates: key-down **counts/rates** only — never characters / key codes
//! that reconstruct text. Opt-in via `BIOFOCUS_INPUT_AGGREGATES=1`.
//! Calendar: local ICS → `calendar_event` Observations (no titles/bodies). Opt-in via
//! `BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS=/path/to/file.ics`.
//! Browser categories: coarse `category` only (no URLs / titles). Opt-in via
//! `BIOFOCUS_BROWSER_CATEGORIES=1`.
//! Now Playing: coarse `media_kind` + `is_playing` only (no titles / lyrics). Opt-in via
//! `BIOFOCUS_NOW_PLAYING=1`.
//! Git activity: coarse `activity_kind` + optional `event_count` only (no paths /
//! remotes / diffs). Opt-in via `BIOFOCUS_GIT_ACTIVITY=1`.

#![cfg_attr(not(target_os = "macos"), forbid(unsafe_code))]
// objc2 / CoreGraphics bindings require `unsafe` only inside macOS probe modules.

mod browser_plugin;
mod browser_probe;
mod browser_stream;
mod calendar_plugin;
mod calendar_probe;
mod calendar_stream;
mod error;
mod git_activity_plugin;
mod git_activity_probe;
mod git_activity_stream;
mod ics;
mod input_plugin;
mod input_probe;
mod input_stream;
mod now_playing_plugin;
mod now_playing_probe;
mod now_playing_stream;
mod payload;
mod plugin;
mod probe;
mod stream;

pub use browser_plugin::{
    browser_categories_enabled, BrowserCategoryPlugin, BROWSER_ENABLE_ENV,
    DEFAULT_BROWSER_POLL_INTERVAL,
};
pub use browser_probe::{
    BrowserCategoryProbe, BrowserCategorySample, ScriptedBrowserProbe, SystemBrowserProbe,
};
pub use browser_stream::{
    observation_from_browser_category, spawn_browser_category_loop, BrowserCategoryHandle,
};
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
pub use git_activity_plugin::{
    git_activity_enabled, GitActivityPlugin, DEFAULT_GIT_ACTIVITY_POLL_INTERVAL,
    GIT_ACTIVITY_ENABLE_ENV,
};
pub use git_activity_probe::{
    GitActivityProbe, GitActivitySample, ScriptedGitActivityProbe, SystemGitActivityProbe,
};
pub use git_activity_stream::{
    observation_from_git_activity, spawn_git_activity_loop, GitActivityHandle,
};
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
pub use now_playing_plugin::{
    now_playing_enabled, NowPlayingPlugin, DEFAULT_NOW_PLAYING_POLL_INTERVAL, NOW_PLAYING_ENABLE_ENV,
};
pub use now_playing_probe::{
    NowPlayingProbe, NowPlayingSample, ScriptedNowPlayingProbe, SystemNowPlayingProbe,
};
pub use now_playing_stream::{
    observation_from_now_playing, spawn_now_playing_loop, NowPlayingHandle,
};
pub use payload::{
    browser_category_payload, calendar_event_payload, context_window_payload, git_activity_payload,
    keystrokes_payload, now_playing_payload, BROWSER_CATEGORY_DATA_TYPE, CALENDAR_EVENT_DATA_TYPE,
    CONTEXT_WINDOW_DATA_TYPE, GIT_ACTIVITY_DATA_TYPE, KEYSTROKES_DATA_TYPE,
    MACOS_BROWSER_PROVIDER_ID, MACOS_CALENDAR_PROVIDER_ID, MACOS_CONTEXT_PROVIDER_ID,
    MACOS_GIT_PROVIDER_ID, MACOS_INPUT_PROVIDER_ID, MACOS_NOW_PLAYING_PROVIDER_ID,
    NOW_PLAYING_DATA_TYPE,
};
pub use plugin::{ActiveWindowPlugin, DEFAULT_POLL_INTERVAL};
pub use probe::{FrontmostApp, FrontmostProbe, SystemFrontmostProbe};
pub use stream::{observation_from_frontmost, spawn_active_window_loop, ActiveWindowHandle};

/// Crate identity.
pub const CRATE_NAME: &str = "macos-collector";
