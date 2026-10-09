//! Browser category Observation kinds (P10-E2-T1 / ADR-010).
//!
//! Browser categories are ordinary [`Observation`] values with
//! `data_type == "browser_category"`. Persistence uses the existing Observation
//! repository. Coarse labels only — no full URLs / page titles. Opt-in local
//! collector; personal self-tracking only.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for Browser category Observations.
pub const DATA_TYPE_BROWSER_CATEGORY: &str = "browser_category";

/// v1 coarse category: work.
pub const BROWSER_CATEGORY_WORK: &str = "work";
/// v1 coarse category: communication.
pub const BROWSER_CATEGORY_COMMUNICATION: &str = "communication";
/// v1 coarse category: entertainment.
pub const BROWSER_CATEGORY_ENTERTAINMENT: &str = "entertainment";
/// v1 coarse category: reference.
pub const BROWSER_CATEGORY_REFERENCE: &str = "reference";
/// v1 coarse category: shopping.
pub const BROWSER_CATEGORY_SHOPPING: &str = "shopping";
/// v1 coarse category: unknown (no URL mapping / soft-fail).
pub const BROWSER_CATEGORY_UNKNOWN: &str = "unknown";

/// Allowed `payload.category` values for v1 (`docs/07-contracts.md` / ADR-010).
pub const V1_BROWSER_CATEGORIES: &[&str] = &[
    BROWSER_CATEGORY_WORK,
    BROWSER_CATEGORY_COMMUNICATION,
    BROWSER_CATEGORY_ENTERTAINMENT,
    BROWSER_CATEGORY_REFERENCE,
    BROWSER_CATEGORY_SHOPPING,
    BROWSER_CATEGORY_UNKNOWN,
];

/// Returns `true` when `category` is a documented v1 browser category.
#[must_use]
pub fn is_v1_browser_category(category: &str) -> bool {
    V1_BROWSER_CATEGORIES.contains(&category)
}

/// Validates a Browser category `payload` object.
///
/// Required: `category` ∈ v1 closed set.
/// Optional: `browser_bundle_id` (string).
/// Unknown keys are allowed (forward-compatible) but collectors must not emit
/// URLs, titles, or content fields.
pub fn validate_browser_category_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidBrowserCategoryPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("category") {
        None => {
            return Err(SpecError::InvalidBrowserCategoryPayload {
                reason: "missing required field category".to_owned(),
            });
        }
        Some(JsonValue::String(s)) if is_v1_browser_category(s) => {}
        Some(JsonValue::String(s)) => {
            return Err(SpecError::InvalidBrowserCategoryPayload {
                reason: format!("category `{s}` is not a v1 browser category"),
            });
        }
        Some(_) => {
            return Err(SpecError::InvalidBrowserCategoryPayload {
                reason: "category must be a string".to_owned(),
            });
        }
    }

    if let Some(bundle) = obj.get("browser_bundle_id") {
        if !bundle.is_string() {
            return Err(SpecError::InvalidBrowserCategoryPayload {
                reason: "browser_bundle_id must be a string when present".to_owned(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_work_with_bundle() {
        validate_browser_category_payload(&json!({
            "category": "work",
            "browser_bundle_id": "com.apple.Safari"
        }))
        .expect("valid");
    }

    #[test]
    fn accepts_unknown_only() {
        validate_browser_category_payload(&json!({ "category": "unknown" })).expect("valid");
    }

    #[test]
    fn rejects_unknown_label() {
        let err = validate_browser_category_payload(&json!({
            "category": "social"
        }))
        .expect_err("not in v1 set");
        assert!(matches!(
            err,
            SpecError::InvalidBrowserCategoryPayload { .. }
        ));
    }

    #[test]
    fn rejects_non_string_bundle() {
        let err = validate_browser_category_payload(&json!({
            "category": "work",
            "browser_bundle_id": 1
        }))
        .expect_err("bundle type");
        assert!(matches!(
            err,
            SpecError::InvalidBrowserCategoryPayload { .. }
        ));
    }
}
