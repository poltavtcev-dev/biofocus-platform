//! Bind-mode + base URL hints for companion / pairing dogfood (P5-E1-T2).
//!
//! Hints are derived on read (or at status build) — no background spin.
//! Never includes Observation payloads, tokens, or filesystem paths.

use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use serde::{Deserialize, Serialize};

use crate::config::{INGEST_BIND_HOST, INGEST_LAN_BIND_HOST};

/// How the ingest TCP listener is bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindMode {
    /// Listening on loopback only (`127.0.0.1`).
    Loopback,
    /// LAN-reachable bind (opt-in `0.0.0.0` or a non-loopback host).
    Lan,
}

/// Advertise payload shared by `GET /v1/status` and pairing IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvertiseInfo {
    /// Loopback vs LAN opt-in bind.
    pub bind_mode: BindMode,
    /// Usable `http://<host>:<port>` URLs (no trailing slash). First is primary.
    pub base_url_hints: Vec<String>,
}

impl AdvertiseInfo {
    /// Builds hints for `bind_host`/`port` using OS primary-LAN discovery when needed.
    #[must_use]
    pub fn for_bind(bind_host: Ipv4Addr, port: u16) -> Self {
        Self::for_bind_with(bind_host, port, discover_lan_ipv4s)
    }

    /// Testable builder: `discover` supplies candidate LAN IPv4s when bind is unspecified.
    #[must_use]
    pub fn for_bind_with<F>(bind_host: Ipv4Addr, port: u16, discover: F) -> Self
    where
        F: FnOnce() -> Vec<Ipv4Addr>,
    {
        if bind_host.is_loopback() {
            return Self {
                bind_mode: BindMode::Loopback,
                base_url_hints: vec![http_base_url(INGEST_BIND_HOST, port)],
            };
        }

        let mut hints = Vec::new();
        if bind_host != INGEST_LAN_BIND_HOST && !bind_host.is_unspecified() {
            hints.push(http_base_url(bind_host, port));
        } else {
            for ip in discover() {
                if !ip.is_loopback() && !ip.is_unspecified() {
                    let url = http_base_url(ip, port);
                    if !hints.contains(&url) {
                        hints.push(url);
                    }
                }
            }
        }

        Self {
            bind_mode: BindMode::Lan,
            base_url_hints: hints,
        }
    }

    /// Primary companion base URL (first hint), if any.
    #[must_use]
    pub fn primary_base_url(&self) -> Option<&str> {
        self.base_url_hints.first().map(String::as_str)
    }
}

/// Formats `http://<ipv4>:<port>` (no trailing slash).
#[must_use]
pub fn http_base_url(host: Ipv4Addr, port: u16) -> String {
    format!("http://{host}:{port}")
}

/// Best-effort primary LAN IPv4 via UDP connect trick (no packets sent).
///
/// Idle-safe: one socket bind/connect; no retry loop.
fn discover_lan_ipv4s() -> Vec<Ipv4Addr> {
    primary_lan_ipv4().into_iter().collect()
}

fn primary_lan_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    // Destination need not be reachable; connect selects the outbound interface.
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => Some(ip),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_advertise_is_stable() {
        let info = AdvertiseInfo::for_bind_with(INGEST_BIND_HOST, 8787, || {
            panic!("discovery must not run for loopback")
        });
        assert_eq!(info.bind_mode, BindMode::Loopback);
        assert_eq!(
            info.base_url_hints,
            vec!["http://127.0.0.1:8787".to_owned()]
        );
        assert_eq!(info.primary_base_url(), Some("http://127.0.0.1:8787"));
    }

    #[test]
    fn lan_unspecified_uses_discovered_ipv4() {
        let info = AdvertiseInfo::for_bind_with(INGEST_LAN_BIND_HOST, 8787, || {
            vec![Ipv4Addr::new(192, 168, 1, 40)]
        });
        assert_eq!(info.bind_mode, BindMode::Lan);
        assert_eq!(
            info.base_url_hints,
            vec!["http://192.168.1.40:8787".to_owned()]
        );
    }

    #[test]
    fn lan_specific_bind_host_skips_discovery() {
        let host = Ipv4Addr::new(10, 0, 0, 5);
        let info = AdvertiseInfo::for_bind_with(host, 9000, || {
            panic!("discovery must not run for concrete bind host")
        });
        assert_eq!(info.bind_mode, BindMode::Lan);
        assert_eq!(info.base_url_hints, vec!["http://10.0.0.5:9000".to_owned()]);
    }

    #[test]
    fn lan_discovery_empty_still_reports_lan_mode() {
        let info = AdvertiseInfo::for_bind_with(INGEST_LAN_BIND_HOST, 8787, Vec::new);
        assert_eq!(info.bind_mode, BindMode::Lan);
        assert!(info.base_url_hints.is_empty());
        assert_eq!(info.primary_base_url(), None);
    }

    #[test]
    fn serialize_bind_mode_snake_case() {
        let info = AdvertiseInfo {
            bind_mode: BindMode::Lan,
            base_url_hints: vec!["http://192.168.0.2:8787".into()],
        };
        let json = serde_json::to_value(&info).expect("json");
        assert_eq!(json["bind_mode"], "lan");
        assert_eq!(json["base_url_hints"][0], "http://192.168.0.2:8787");
        let round: AdvertiseInfo = serde_json::from_value(json).expect("roundtrip");
        assert_eq!(round, info);
    }

    #[test]
    fn json_has_no_paths_or_secrets() {
        let info = AdvertiseInfo::for_bind_with(INGEST_BIND_HOST, 8787, Vec::new);
        let raw = serde_json::to_string(&info).expect("string");
        assert!(!raw.contains(".biofocus"));
        assert!(!raw.contains("/Users"));
        assert!(!raw.contains("token"));
        assert!(!raw.contains("observation"));
    }
}
