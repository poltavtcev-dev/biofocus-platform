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

/// Best-effort LAN IPv4 discovery.
///
/// 1. UDP connect trick toward 8.8.8.8 (no packets sent) — picks the outbound NIC.
/// 2. If that fails (offline Wi-Fi, no default route, captive portal), list
///    interface addresses via `ifconfig` and keep usable private/LAN IPv4s.
///
/// Runs on each call (no caching) so a UI "Reload" re-detects the address.
fn discover_lan_ipv4s() -> Vec<Ipv4Addr> {
    let mut out: Vec<Ipv4Addr> = primary_lan_ipv4().into_iter().collect();
    for ip in interface_ipv4s() {
        if !out.contains(&ip) {
            out.push(ip);
        }
    }
    out
}

/// Interface IPv4s from `ifconfig` (macOS / BSD; also present on many Linux boxes).
/// Missing tool or failure → empty (soft-fail).
fn interface_ipv4s() -> Vec<Ipv4Addr> {
    let output = ["/sbin/ifconfig", "/usr/sbin/ifconfig", "ifconfig"]
        .iter()
        .find_map(|bin| std::process::Command::new(bin).output().ok())
        .filter(|o| o.status.success());
    match output {
        Some(o) => parse_ifconfig_ipv4s(&String::from_utf8_lossy(&o.stdout)),
        None => Vec::new(),
    }
}

/// Parses `inet a.b.c.d` lines; keeps addresses a phone on the same network could
/// reach. Private ranges first, then other non-link-local addresses.
pub(crate) fn parse_ifconfig_ipv4s(text: &str) -> Vec<Ipv4Addr> {
    let mut private = Vec::new();
    let mut other = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if parts.next() != Some("inet") {
            continue;
        }
        let Some(raw) = parts.next() else { continue };
        // Linux `ifconfig` may print `addr:1.2.3.4`; `ip`-style may print `1.2.3.4/24`.
        let raw = raw.trim_start_matches("addr:");
        let raw = raw.split('/').next().unwrap_or(raw);
        let Ok(ip) = raw.parse::<Ipv4Addr>() else { continue };
        if !is_usable_lan_ipv4(ip) {
            continue;
        }
        let bucket = if ip.is_private() { &mut private } else { &mut other };
        if !bucket.contains(&ip) {
            bucket.push(ip);
        }
    }
    private.extend(other);
    private
}

fn is_usable_lan_ipv4(ip: Ipv4Addr) -> bool {
    !ip.is_loopback() && !ip.is_unspecified() && !ip.is_link_local() && !ip.is_broadcast()
        && !ip.is_multicast()
}

fn primary_lan_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    // Destination need not be reachable; connect selects the outbound interface.
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if is_usable_lan_ipv4(ip) => Some(ip),
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

    #[test]
    fn parse_ifconfig_macos_output_skips_loopback_and_link_local() {
        let text = "lo0: flags=8049<UP,LOOPBACK>\n\tinet 127.0.0.1 netmask 0xff000000\n\
en0: flags=8863<UP>\n\tinet 169.254.10.2 netmask 0xffff0000\n\
\tinet 192.168.0.37 netmask 0xffffff00 broadcast 192.168.0.255\n\
utun4: flags=8051<UP>\n\tinet 100.64.1.5 --> 100.64.1.5 netmask 0xffffffff\n";
        let ips = super::parse_ifconfig_ipv4s(text);
        assert_eq!(
            ips,
            vec![Ipv4Addr::new(192, 168, 0, 37), Ipv4Addr::new(100, 64, 1, 5)]
        );
    }

    #[test]
    fn parse_ifconfig_linux_styles() {
        let text = "inet addr:10.0.0.4  Bcast:10.0.0.255\n    inet 172.16.3.9/24 brd x\n";
        let ips = super::parse_ifconfig_ipv4s(text);
        assert_eq!(
            ips,
            vec![Ipv4Addr::new(10, 0, 0, 4), Ipv4Addr::new(172, 16, 3, 9)]
        );
    }

    #[test]
    fn parse_ifconfig_empty_or_garbage_is_empty() {
        assert!(super::parse_ifconfig_ipv4s("").is_empty());
        assert!(super::parse_ifconfig_ipv4s("inet not-an-ip\ninet6 ::1").is_empty());
    }
}
