//! Now Playing Observation kinds (P12-E2-T1 / ADR-012).
//!
//! Now Playing facts are ordinary [`Observation`] values with
//! `data_type == "now_playing"`. Persistence uses the existing Observation
//! repository. Coarse `media_kind` + `is_playing` only — no titles / artists /
//! lyrics / playlists. Opt-in local collector; personal self-tracking only.

use serde_json::Value as JsonValue;

use crate::{SpecError, SpecResult};

/// Canonical `data_type` for Now Playing Observations.
pub const DATA_TYPE_NOW_PLAYING: &str = "now_playing";

/// v1 media kind: music.
pub const MEDIA_KIND_MUSIC: &str = "music";
/// v1 media kind: podcast.
pub const MEDIA_KIND_PODCAST: &str = "podcast";
/// v1 media kind: other (active media not music/podcast).
pub const MEDIA_KIND_OTHER: &str = "other";
/// v1 media kind: none (idle / nothing playing).
pub const MEDIA_KIND_NONE: &str = "none";
/// v1 media kind: unknown (soft-fail mapping).
pub const MEDIA_KIND_UNKNOWN: &str = "unknown";

/// Allowed `payload.media_kind` values for v1 (`docs/07-contracts.md` / ADR-012).
pub const V1_MEDIA_KINDS: &[&str] = &[
    MEDIA_KIND_MUSIC,
    MEDIA_KIND_PODCAST,
    MEDIA_KIND_OTHER,
    MEDIA_KIND_NONE,
    MEDIA_KIND_UNKNOWN,
];

/// Returns `true` when `media_kind` is a documented v1 Now Playing kind.
#[must_use]
pub fn is_v1_media_kind(media_kind: &str) -> bool {
    V1_MEDIA_KINDS.contains(&media_kind)
}

/// Validates a Now Playing `payload` object.
///
/// Required: `media_kind` ∈ v1 closed set; `is_playing` boolean.
/// Unknown keys are allowed (forward-compatible) but collectors must not emit
/// titles, artists, albums, lyrics, playlist ids, or mic/waveform fields.
pub fn validate_now_playing_payload(payload: &JsonValue) -> SpecResult<()> {
    let obj = payload
        .as_object()
        .ok_or_else(|| SpecError::InvalidNowPlayingPayload {
            reason: "payload must be a JSON object".to_owned(),
        })?;

    match obj.get("media_kind") {
        None => {
            return Err(SpecError::InvalidNowPlayingPayload {
                reason: "missing required field media_kind".to_owned(),
            });
        }
        Some(JsonValue::String(s)) if is_v1_media_kind(s) => {}
        Some(JsonValue::String(s)) => {
            return Err(SpecError::InvalidNowPlayingPayload {
                reason: format!("media_kind `{s}` is not a v1 Now Playing kind"),
            });
        }
        Some(_) => {
            return Err(SpecError::InvalidNowPlayingPayload {
                reason: "media_kind must be a string".to_owned(),
            });
        }
    }

    match obj.get("is_playing") {
        None => {
            return Err(SpecError::InvalidNowPlayingPayload {
                reason: "missing required field is_playing".to_owned(),
            });
        }
        Some(JsonValue::Bool(_)) => {}
        Some(_) => {
            return Err(SpecError::InvalidNowPlayingPayload {
                reason: "is_playing must be a boolean".to_owned(),
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
    fn accepts_music_playing() {
        validate_now_playing_payload(&json!({
            "media_kind": "music",
            "is_playing": true
        }))
        .expect("valid");
    }

    #[test]
    fn accepts_none_idle() {
        validate_now_playing_payload(&json!({
            "media_kind": "none",
            "is_playing": false
        }))
        .expect("valid");
    }

    #[test]
    fn rejects_unknown_kind() {
        let err = validate_now_playing_payload(&json!({
            "media_kind": "audiobook",
            "is_playing": true
        }))
        .expect_err("not in v1 set");
        assert!(matches!(err, SpecError::InvalidNowPlayingPayload { .. }));
    }

    #[test]
    fn rejects_non_bool_playing() {
        let err = validate_now_playing_payload(&json!({
            "media_kind": "music",
            "is_playing": "yes"
        }))
        .expect_err("playing type");
        assert!(matches!(err, SpecError::InvalidNowPlayingPayload { .. }));
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(matches!(
            validate_now_playing_payload(&json!({ "media_kind": "music" })),
            Err(SpecError::InvalidNowPlayingPayload { .. })
        ));
        assert!(matches!(
            validate_now_playing_payload(&json!({ "is_playing": true })),
            Err(SpecError::InvalidNowPlayingPayload { .. })
        ));
    }
}
