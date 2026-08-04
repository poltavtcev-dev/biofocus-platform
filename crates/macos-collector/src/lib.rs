//! macOS context collectors — active window → `context_window` Observations.
//!
//! Emits metadata only (`bundle_id`, `app_name`). No keystrokes, clipboard,
//! screenshots, or window titles (window title would need Accessibility → T2+).

#![cfg_attr(not(target_os = "macos"), forbid(unsafe_code))]
// objc2 AppKit bindings require `unsafe` only inside the macOS probe module.

mod error;
mod payload;
mod plugin;
mod probe;
mod stream;

pub use error::{CollectorError, CollectorResult};
pub use payload::{
    context_window_payload, CONTEXT_WINDOW_DATA_TYPE, MACOS_CONTEXT_PROVIDER_ID,
};
pub use plugin::{ActiveWindowPlugin, DEFAULT_POLL_INTERVAL};
pub use probe::{FrontmostApp, FrontmostProbe, SystemFrontmostProbe};
pub use stream::{observation_from_frontmost, spawn_active_window_loop, ActiveWindowHandle};

/// Crate identity.
pub const CRATE_NAME: &str = "macos-collector";
