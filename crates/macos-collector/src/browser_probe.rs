//! Browser category probe: frontmost browser → coarse category (no URLs).

use std::sync::Mutex;

use crate::error::CollectorResult;
use crate::probe::{FrontmostProbe, SystemFrontmostProbe};

/// Privacy-safe browser category sample (no URL / title / content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCategorySample {
    /// Coarse v1 category (`work` / `communication` / … / `unknown`).
    pub category: String,
    /// Optional frontmost browser bundle id.
    pub browser_bundle_id: Option<String>,
}

impl BrowserCategorySample {
    /// Identity key used to detect changes (category + optional bundle).
    #[must_use]
    pub fn identity_key(&self) -> String {
        match &self.browser_bundle_id {
            Some(b) if !b.is_empty() => format!("{}|{b}", self.category),
            _ => self.category.clone(),
        }
    }
}

/// Injectable browser-category source (tests / OS stub).
pub trait BrowserCategoryProbe: Send + Sync {
    /// Returns the current coarse category, or `None` if unavailable / not a browser.
    fn current(&self) -> CollectorResult<Option<BrowserCategorySample>>;
}

/// Scripted in-memory probe for fixture tests.
#[derive(Debug, Default)]
pub struct ScriptedBrowserProbe {
    samples: Mutex<Vec<Option<BrowserCategorySample>>>,
}

impl ScriptedBrowserProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the fixture queue (consumed FIFO by [`BrowserCategoryProbe::current`]).
    pub fn set_samples(&self, samples: Vec<Option<BrowserCategorySample>>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        *guard = samples;
    }

    /// Append one fixture sample.
    pub fn push(&self, sample: Option<BrowserCategorySample>) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(sample);
    }
}

impl BrowserCategoryProbe for ScriptedBrowserProbe {
    fn current(&self) -> CollectorResult<Option<BrowserCategorySample>> {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return Ok(None);
        }
        Ok(guard.remove(0))
    }
}

/// Production probe: known browsers via frontmost app; category stays `unknown`
/// without URL mapping (privacy-first soft-fail). Non-browser → `None`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemBrowserProbe;

impl BrowserCategoryProbe for SystemBrowserProbe {
    fn current(&self) -> CollectorResult<Option<BrowserCategorySample>> {
        system_browser_category()
    }
}

fn system_browser_category() -> CollectorResult<Option<BrowserCategorySample>> {
    let Some(app) = SystemFrontmostProbe.frontmost()? else {
        return Ok(None);
    };
    if !is_known_browser_bundle(&app.bundle_id) {
        return Ok(None);
    }
    Ok(Some(BrowserCategorySample {
        // No URL / title capture in v1 OS probe — coarse unknown only.
        category: bio_spec::BROWSER_CATEGORY_UNKNOWN.to_string(),
        browser_bundle_id: if app.bundle_id.is_empty() {
            None
        } else {
            Some(app.bundle_id)
        },
    }))
}

/// Known browser bundle ids (macOS). Used only to detect browser vs other apps.
fn is_known_browser_bundle(bundle_id: &str) -> bool {
    matches!(
        bundle_id,
        "com.apple.Safari"
            | "com.google.Chrome"
            | "com.google.Chrome.canary"
            | "org.mozilla.firefox"
            | "org.mozilla.firefoxdeveloperedition"
            | "company.thebrowser.Browser" // Arc
            | "com.brave.Browser"
            | "com.microsoft.edgemac"
            | "com.operasoftware.Opera"
            | "com.vivaldi.Vivaldi"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_safari_is_browser() {
        assert!(is_known_browser_bundle("com.apple.Safari"));
        assert!(!is_known_browser_bundle("com.apple.Terminal"));
    }

    #[test]
    fn scripted_probe_fifo() {
        let probe = ScriptedBrowserProbe::new();
        probe.push(Some(BrowserCategorySample {
            category: "work".into(),
            browser_bundle_id: Some("com.apple.Safari".into()),
        }));
        probe.push(None);
        let first = probe.current().expect("ok").expect("sample");
        assert_eq!(first.category, "work");
        assert!(probe.current().expect("ok").is_none());
        assert!(probe.current().expect("ok").is_none());
    }
}
