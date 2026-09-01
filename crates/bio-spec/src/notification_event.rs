//! Notification event Observation kinds (ADR-019 / P18-E2).
//!
//! Notification facts are ordinary [`Observation`] values with
//! `data_type == "notification_event"`. Persistence uses the existing Observation
//! repository. Coarse `count` + optional closed-set labels only — **no** body /
//! title / message / screenshots. Opt-in local collector; personal self-tracking only.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for notification event Observations.
pub const DATA_TYPE_NOTIFICATION_EVENT: &str = "notification_event";

/// v1 category: communication.
pub const NOTIFICATION_CATEGORY_COMMUNICATION: &str = "communication";
/// v1 category: calendar.
pub const NOTIFICATION_CATEGORY_CALENDAR: &str = "calendar";
/// v1 category: system.
pub const NOTIFICATION_CATEGORY_SYSTEM: &str = "system";
/// v1 category: media.
pub const NOTIFICATION_CATEGORY_MEDIA: &str = "media";
/// v1 category: social.
pub const NOTIFICATION_CATEGORY_SOCIAL: &str = "social";
/// v1 category: other.
pub const NOTIFICATION_CATEGORY_OTHER: &str = "other";
/// v1 category: unknown.
pub const NOTIFICATION_CATEGORY_UNKNOWN: &str = "unknown";

/// Allowed optional `payload.category` values (ADR-019).
pub const V1_NOTIFICATION_CATEGORIES: &[&str] = &[
    NOTIFICATION_CATEGORY_COMMUNICATION,
    NOTIFICATION_CATEGORY_CALENDAR,
    NOTIFICATION_CATEGORY_SYSTEM,
    NOTIFICATION_CATEGORY_MEDIA,
    NOTIFICATION_CATEGORY_SOCIAL,
    NOTIFICATION_CATEGORY_OTHER,
    NOTIFICATION_CATEGORY_UNKNOWN,
];

/// v1 interruption: passive.
pub const INTERRUPTION_LEVEL_PASSIVE: &str = "passive";
/// v1 interruption: active.
pub const INTERRUPTION_LEVEL_ACTIVE: &str = "active";
/// v1 interruption: time_sensitive.
pub const INTERRUPTION_LEVEL_TIME_SENSITIVE: &str = "time_sensitive";
/// v1 interruption: critical.
pub const INTERRUPTION_LEVEL_CRITICAL: &str = "critical";
/// v1 interruption: unknown.
pub const INTERRUPTION_LEVEL_UNKNOWN: &str = "unknown";

/// Allowed optional `payload.interruption_level` values (ADR-019).
pub const V1_INTERRUPTION_LEVELS: &[&str] = &[
    INTERRUPTION_LEVEL_PASSIVE,
    INTERRUPTION_LEVEL_ACTIVE,
    INTERRUPTION_LEVEL_TIME_SENSITIVE,
    INTERRUPTION_LEVEL_CRITICAL,
    INTERRUPTION_LEVEL_UNKNOWN,
];

/// v1 app_kind: messaging.
pub const NOTIFICATION_APP_KIND_MESSAGING: &str = "messaging";
/// v1 app_kind: mail.
pub const NOTIFICATION_APP_KIND_MAIL: &str = "mail";
/// v1 app_kind: calendar.
pub const NOTIFICATION_APP_KIND_CALENDAR: &str = "calendar";
/// v1 app_kind: social.
pub const NOTIFICATION_APP_KIND_SOCIAL: &str = "social";
/// v1 app_kind: system.
pub const NOTIFICATION_APP_KIND_SYSTEM: &str = "system";
/// v1 app_kind: other.
pub const NOTIFICATION_APP_KIND_OTHER: &str = "other";
/// v1 app_kind: unknown.
pub const NOTIFICATION_APP_KIND_UNKNOWN: &str = "unknown";

/// Allowed optional `payload.app_kind` values (ADR-019).
pub const V1_NOTIFICATION_APP_KINDS: &[&str] = &[
    NOTIFICATION_APP_KIND_MESSAGING,
    NOTIFICATION_APP_KIND_MAIL,
    NOTIFICATION_APP_KIND_CALENDAR,
    NOTIFICATION_APP_KIND_SOCIAL,
    NOTIFICATION_APP_KIND_SYSTEM,
    NOTIFICATION_APP_KIND_OTHER,
    NOTIFICATION_APP_KIND_UNKNOWN,
];

/// Returns `true` when `category` is a documented v1 notification category.
#[must_use]
pub fn is_v1_notification_category(category: &str) -> bool {
    V1_NOTIFICATION_CATEGORIES.contains(&category)
}

/// Returns `true` when `level` is a documented v1 interruption level.
#[must_use]
pub fn is_v1_interruption_level(level: &str) -> bool {
    V1_INTERRUPTION_LEVELS.contains(&level)
}

/// Returns `true` when `app_kind` is a documented v1 notification app kind.
#[must_use]
pub fn is_v1_notification_app_kind(app_kind: &str) -> bool {
    V1_NOTIFICATION_APP_KINDS.contains(&app_kind)
}

/// Validates a notification event `payload` object.
///
/// Required: `count` integer ≥ 1.
/// Optional: closed-set `category` / `interruption_level` / `app_kind`.
/// Unknown keys are allowed (forward-compatible) but collectors must not emit
/// body/title/message — pipeline normalize strips those extras.
pub fn validate_notification_event_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidNotificationEventPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("count") {
        None => {
            return Err(SpecError::InvalidNotificationEventPayload {
                reason: "missing required field count".to_owned(),
            });
        }
        Some(v) => {
            let count = nonneg_u64(v).ok_or_else(|| SpecError::InvalidNotificationEventPayload {
                reason: "count must be an integer ≥ 1".to_owned(),
            })?;
            if count < 1 {
                return Err(SpecError::InvalidNotificationEventPayload {
                    reason: "count must be an integer ≥ 1".to_owned(),
                });
            }
        }
    }

    validate_optional_closed_set(obj, "category", is_v1_notification_category)?;
    validate_optional_closed_set(obj, "interruption_level", is_v1_interruption_level)?;
    validate_optional_closed_set(obj, "app_kind", is_v1_notification_app_kind)?;

    Ok(())
}

fn validate_optional_closed_set(
    obj: &serde_json::Map<String, JsonValue>,
    key: &str,
    is_allowed: fn(&str) -> bool,
) -> SpecResult<()> {
    match obj.get(key) {
        None => Ok(()),
        Some(JsonValue::String(s)) if is_allowed(s) => Ok(()),
        Some(JsonValue::String(s)) => Err(SpecError::InvalidNotificationEventPayload {
            reason: format!("{key} `{s}` is not a v1 closed-set value"),
        }),
        Some(_) => Err(SpecError::InvalidNotificationEventPayload {
            reason: format!("{key} must be a string"),
        }),
    }
}

fn nonneg_u64(v: &JsonValue) -> Option<u64> {
    match v {
        JsonValue::Number(n) => n.as_u64().or_else(|| {
            n.as_i64()
                .and_then(|i| u64::try_from(i).ok())
                .or_else(|| {
                    n.as_f64()
                        .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                        .map(|f| f as u64)
                })
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn accepts_count_only() {
        validate_notification_event_payload(&json!({ "count": 1 })).expect("ok");
    }

    #[test]
    fn accepts_full_optional_set() {
        validate_notification_event_payload(&json!({
            "count": 3,
            "category": "communication",
            "interruption_level": "active",
            "app_kind": "messaging"
        }))
        .expect("ok");
    }

    #[test]
    fn rejects_missing_count() {
        assert!(matches!(
            validate_notification_event_payload(&json!({})),
            Err(SpecError::InvalidNotificationEventPayload { .. })
        ));
    }

    #[test]
    fn rejects_zero_count() {
        assert!(matches!(
            validate_notification_event_payload(&json!({ "count": 0 })),
            Err(SpecError::InvalidNotificationEventPayload { .. })
        ));
    }

    #[test]
    fn rejects_bad_category() {
        assert!(matches!(
            validate_notification_event_payload(&json!({
                "count": 1,
                "category": "urgent_work"
            })),
            Err(SpecError::InvalidNotificationEventPayload { .. })
        ));
    }

    #[test]
    fn rejects_bad_interruption_level() {
        assert!(matches!(
            validate_notification_event_payload(&json!({
                "count": 1,
                "interruption_level": "loud"
            })),
            Err(SpecError::InvalidNotificationEventPayload { .. })
        ));
    }

    #[test]
    fn rejects_bad_app_kind() {
        assert!(matches!(
            validate_notification_event_payload(&json!({
                "count": 1,
                "app_kind": "Slack"
            })),
            Err(SpecError::InvalidNotificationEventPayload { .. })
        ));
    }
}
