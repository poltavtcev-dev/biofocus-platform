//! Pairing QR text. Four lines, no JSON, so a phone can split on newlines.
//!
//! ```text
//! biofocus:1
//! https://192.168.1.10:8787
//! <bearer token>
//! <sha256 cert fingerprint, or "-" on loopback>
//! ```

/// Parsed contents of a pairing QR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPairingQr {
    /// Ingest base URL without a trailing slash.
    pub url: String,
    /// Bearer token.
    pub token: String,
    /// SHA-256 fingerprint of the server certificate, when the URL is TLS.
    pub fingerprint: Option<String>,
}

/// Builds the QR payload. Empty fingerprint becomes `-` (loopback HTTP).
#[must_use]
pub fn pairing_qr_payload(url: &str, token: &str, fingerprint: Option<&str>) -> String {
    let pin = fingerprint
        .map(str::trim)
        .filter(|pin| !pin.is_empty())
        .unwrap_or("-");
    format!("biofocus:1\n{url}\n{token}\n{pin}\n")
}

/// Parses a QR payload. Returns `None` when the version or shape is wrong.
#[must_use]
pub fn parse_pairing_qr(text: &str) -> Option<ParsedPairingQr> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.len() != 4 || lines[0] != "biofocus:1" {
        return None;
    }
    let url = lines[1].trim_end_matches('/').to_owned();
    if !(url.starts_with("https://") || url.starts_with("http://")) || lines[2].is_empty() {
        return None;
    }
    let fingerprint = if lines[3] == "-" {
        None
    } else {
        Some(lines[3].to_owned())
    };
    Some(ParsedPairingQr {
        url,
        token: lines[2].to_owned(),
        fingerprint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_pin_and_loopback_dash() {
        let pin = "ab".repeat(32);
        let text = pairing_qr_payload("https://10.0.0.8:8787", "tok", Some(&pin));
        let parsed = parse_pairing_qr(&text).expect("parse");
        assert_eq!(parsed.url, "https://10.0.0.8:8787");
        assert_eq!(parsed.token, "tok");
        assert_eq!(parsed.fingerprint.as_deref(), Some(pin.as_str()));

        let local = pairing_qr_payload("http://127.0.0.1:8787", "tok", None);
        let parsed = parse_pairing_qr(&local).expect("parse");
        assert_eq!(parsed.fingerprint, None);
    }

    #[test]
    fn rejects_unknown_version() {
        assert!(parse_pairing_qr("biofocus:2\nhttp://127.0.0.1:8787\nt\n-\n").is_none());
        assert!(parse_pairing_qr("just-a-token").is_none());
    }
}
