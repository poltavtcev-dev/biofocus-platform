//! Frontmost application probe (injectable for tests).

use crate::error::CollectorResult;

/// Snapshot of the frontmost GUI application (metadata only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmostApp {
    /// Bundle identifier (e.g. `com.apple.Terminal`). May be empty for some apps.
    pub bundle_id: String,
    /// Localized application name.
    pub app_name: String,
}

impl FrontmostApp {
    /// Identity key used to detect changes (bundle preferred, else name).
    #[must_use]
    pub fn identity_key(&self) -> String {
        if !self.bundle_id.is_empty() {
            self.bundle_id.clone()
        } else {
            self.app_name.clone()
        }
    }
}

/// OS (or mock) source of the frontmost application.
pub trait FrontmostProbe: Send + Sync {
    /// Returns the current frontmost app, or `None` if unavailable.
    fn frontmost(&self) -> CollectorResult<Option<FrontmostApp>>;
}

/// Production probe: NSWorkspace on macOS, inert stub elsewhere.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemFrontmostProbe;

impl FrontmostProbe for SystemFrontmostProbe {
    fn frontmost(&self) -> CollectorResult<Option<FrontmostApp>> {
        system_frontmost()
    }
}

#[cfg(target_os = "macos")]
fn system_frontmost() -> CollectorResult<Option<FrontmostApp>> {
    macos_nsworkspace::frontmost_via_nsworkspace()
}

#[cfg(not(target_os = "macos"))]
fn system_frontmost() -> CollectorResult<Option<FrontmostApp>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
mod macos_nsworkspace {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;

    use super::FrontmostApp;
    use crate::error::CollectorResult;

    /// Reads frontmost app via `NSWorkspace` (no Accessibility permission).
    pub(super) fn frontmost_via_nsworkspace() -> CollectorResult<Option<FrontmostApp>> {
        // NSWorkspace is generally safe to call off the main thread for this read.
        let workspace = NSWorkspace::sharedWorkspace();
        let Some(app) = workspace.frontmostApplication() else {
            return Ok(None);
        };

        let bundle_id = app
            .bundleIdentifier()
            .map(|s: objc2::rc::Retained<NSString>| s.to_string())
            .unwrap_or_default();
        let app_name = app
            .localizedName()
            .map(|s: objc2::rc::Retained<NSString>| s.to_string())
            .unwrap_or_default();

        if bundle_id.is_empty() && app_name.is_empty() {
            return Ok(None);
        }

        Ok(Some(FrontmostApp {
            bundle_id,
            app_name,
        }))
    }
}
