//! Bearer token extraction for ingest endpoints.

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

const BEARER_PREFIX: &str = "Bearer ";

/// Returns the Bearer token from `Authorization`, if present and well-formed.
#[must_use]
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix(BEARER_PREFIX)?;
    if token.is_empty() {
        return None;
    }
    Some(token)
}

/// Constant-time-ish equality for short tokens (skeleton; T2 may harden).
#[must_use]
pub fn tokens_equal(expected: &str, provided: &str) -> bool {
    if expected.len() != provided.len() {
        return false;
    }
    expected
        .bytes()
        .zip(provided.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn extracts_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer secret-token"),
        );
        assert_eq!(bearer_token(&headers), Some("secret-token"));
    }

    #[test]
    fn rejects_missing_or_malformed() {
        let mut headers = HeaderMap::new();
        assert!(bearer_token(&headers).is_none());

        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic x"));
        assert!(bearer_token(&headers).is_none());

        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert!(bearer_token(&headers).is_none());
    }

    #[test]
    fn tokens_equal_rejects_mismatch() {
        assert!(tokens_equal("abc", "abc"));
        assert!(!tokens_equal("abc", "abd"));
        assert!(!tokens_equal("abc", "ab"));
    }
}
