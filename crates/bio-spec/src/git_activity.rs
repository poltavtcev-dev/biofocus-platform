//! Git activity Observation kinds (P13-E2-T1 / ADR-013).
//!
//! Git activity facts are ordinary [`Observation`] values with
//! `data_type == "git_activity"`. Persistence uses the existing Observation
//! repository. Coarse `activity_kind` + optional `event_count` only — no repo
//! paths, remotes, branch names, SHAs, commit messages, diffs, or authors.
//! Opt-in local collector; personal self-tracking only.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for Git activity Observations.
pub const DATA_TYPE_GIT_ACTIVITY: &str = "git_activity";

/// v1 activity kind: commit.
pub const ACTIVITY_KIND_COMMIT: &str = "commit";
/// v1 activity kind: checkout.
pub const ACTIVITY_KIND_CHECKOUT: &str = "checkout";
/// v1 activity kind: sync (fetch/pull/push events as kinds only).
pub const ACTIVITY_KIND_SYNC: &str = "sync";
/// v1 activity kind: other coarse VCS event.
pub const ACTIVITY_KIND_OTHER: &str = "other";
/// v1 activity kind: idle (opt-in probe saw no recent activity).
pub const ACTIVITY_KIND_IDLE: &str = "idle";
/// v1 activity kind: unknown (soft-fail mapping).
pub const ACTIVITY_KIND_UNKNOWN: &str = "unknown";

/// Allowed `payload.activity_kind` values for v1 (`docs/07-contracts.md` / ADR-013).
pub const V1_ACTIVITY_KINDS: &[&str] = &[
    ACTIVITY_KIND_COMMIT,
    ACTIVITY_KIND_CHECKOUT,
    ACTIVITY_KIND_SYNC,
    ACTIVITY_KIND_OTHER,
    ACTIVITY_KIND_IDLE,
    ACTIVITY_KIND_UNKNOWN,
];

/// Returns `true` when `activity_kind` is a documented v1 Git activity kind.
#[must_use]
pub fn is_v1_activity_kind(activity_kind: &str) -> bool {
    V1_ACTIVITY_KINDS.contains(&activity_kind)
}

/// Validates a Git activity `payload` object.
///
/// Required: `activity_kind` ∈ v1 closed set.
/// Optional: `event_count` positive integer (≥ 1).
/// Unknown keys are allowed (forward-compatible) but collectors must not emit
/// paths, remotes, branch names, SHAs, messages, diffs, or authors — pipeline
/// normalize strips those extras.
pub fn validate_git_activity_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidGitActivityPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("activity_kind") {
        None => {
            return Err(SpecError::InvalidGitActivityPayload {
                reason: "missing required field activity_kind".to_owned(),
            });
        }
        Some(JsonValue::String(s)) if is_v1_activity_kind(s) => {}
        Some(JsonValue::String(s)) => {
            return Err(SpecError::InvalidGitActivityPayload {
                reason: format!("activity_kind `{s}` is not a v1 Git activity kind"),
            });
        }
        Some(_) => {
            return Err(SpecError::InvalidGitActivityPayload {
                reason: "activity_kind must be a string".to_owned(),
            });
        }
    }

    if let Some(count_val) = obj.get("event_count") {
        let count = match count_val {
            JsonValue::Number(n) => n
                .as_u64()
                .or_else(|| {
                    n.as_i64()
                        .and_then(|i| u64::try_from(i).ok())
                        .or_else(|| {
                            n.as_f64()
                                .filter(|f| f.is_finite() && *f >= 0.0 && f.fract() == 0.0)
                                .map(|f| f as u64)
                        })
                })
                .ok_or_else(|| SpecError::InvalidGitActivityPayload {
                    reason: "event_count must be a positive integer".to_owned(),
                })?,
            _ => {
                return Err(SpecError::InvalidGitActivityPayload {
                    reason: "event_count must be a positive integer".to_owned(),
                });
            }
        };
        if count < 1 {
            return Err(SpecError::InvalidGitActivityPayload {
                reason: "event_count must be ≥ 1".to_owned(),
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
    fn accepts_commit_with_count() {
        validate_git_activity_payload(&json!({
            "activity_kind": "commit",
            "event_count": 2
        }))
        .expect("valid");
    }

    #[test]
    fn accepts_idle_without_count() {
        validate_git_activity_payload(&json!({
            "activity_kind": "idle"
        }))
        .expect("valid");
    }

    #[test]
    fn rejects_unknown_kind() {
        let err = validate_git_activity_payload(&json!({
            "activity_kind": "rebase"
        }))
        .expect_err("not in v1 set");
        assert!(matches!(err, SpecError::InvalidGitActivityPayload { .. }));
    }

    #[test]
    fn rejects_zero_event_count() {
        let err = validate_git_activity_payload(&json!({
            "activity_kind": "sync",
            "event_count": 0
        }))
        .expect_err("count ≥ 1");
        assert!(matches!(err, SpecError::InvalidGitActivityPayload { .. }));
    }

    #[test]
    fn rejects_missing_kind() {
        assert!(matches!(
            validate_git_activity_payload(&json!({ "event_count": 1 })),
            Err(SpecError::InvalidGitActivityPayload { .. })
        ));
    }
}
