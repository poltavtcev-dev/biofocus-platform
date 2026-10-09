//! Self-signed ingest certificate for LAN. Loopback stays plain HTTP.
//!
//! The fingerprint is SHA-256 of the certificate DER, lowercase hex. The phone
//! pins that value. A different certificate fails the handshake.

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Write};
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use sha2::{Digest, Sha256};

use crate::error::{IngestError, IngestResult};
use crate::token::biofocus_config_dir;

const CERT_FILE: &str = "ingest_cert.pem";
const KEY_FILE: &str = "ingest_key.pem";

/// How ingest speaks on a bind address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestTransport {
    /// Plain HTTP. Only loopback.
    PlainHttp,
    /// TLS with the local self-signed certificate.
    Tls,
}

/// Plain HTTP is loopback only. Every other bind is TLS.
#[must_use]
pub fn transport_for_bind(host: Ipv4Addr) -> IngestTransport {
    if host.is_loopback() {
        IngestTransport::PlainHttp
    } else {
        IngestTransport::Tls
    }
}

/// PEM pair plus the fingerprint a companion must pin.
#[derive(Debug, Clone)]
pub struct TlsIdentity {
    /// Certificate PEM.
    pub cert_pem: String,
    /// Private key PEM. Never log this.
    pub key_pem: String,
    /// SHA-256 of the certificate DER, 64 lowercase hex chars.
    pub fingerprint_hex: String,
}

/// Loads `ingest_cert.pem` + `ingest_key.pem`, or creates them once.
pub fn load_or_create_tls_identity() -> IngestResult<TlsIdentity> {
    let dir = biofocus_config_dir()?;
    load_or_create_tls_identity_at(&dir)
}

pub(crate) fn load_or_create_tls_identity_at(dir: &Path) -> IngestResult<TlsIdentity> {
    fs::create_dir_all(dir).map_err(|source| IngestError::TokenIo {
        path: dir.display().to_string(),
        source,
    })?;
    let cert_path = dir.join(CERT_FILE);
    let key_path = dir.join(KEY_FILE);
    match (cert_path.exists(), key_path.exists()) {
        (true, true) => read_identity(&cert_path, &key_path),
        (false, false) => {
            let identity = generate_tls_identity()?;
            write_private(&cert_path, identity.cert_pem.as_bytes())?;
            write_private(&key_path, identity.key_pem.as_bytes())?;
            Ok(identity)
        }
        _ => Err(IngestError::Tls(
            "LAN certificate files are incomplete".to_owned(),
        )),
    }
}

/// New self-signed certificate. DNS name is fixed; companions pin the fingerprint
/// and do not check the name, because the LAN address changes.
pub fn generate_tls_identity() -> IngestResult<TlsIdentity> {
    let key = KeyPair::generate().map_err(|err| IngestError::Tls(err.to_string()))?;
    let mut params = CertificateParams::new(vec!["biofocus-ingest".to_owned()])
        .map_err(|err| IngestError::Tls(err.to_string()))?;
    let mut name = DistinguishedName::new();
    name.push(DnType::CommonName, "BioFocus ingest");
    params.distinguished_name = name;
    let cert = params
        .self_signed(&key)
        .map_err(|err| IngestError::Tls(err.to_string()))?;
    let fingerprint_hex = fingerprint_hex(cert.der().as_ref());
    Ok(TlsIdentity {
        cert_pem: cert.pem(),
        key_pem: key.serialize_pem(),
        fingerprint_hex,
    })
}

/// SHA-256 of certificate DER, lowercase hex.
#[must_use]
pub fn fingerprint_hex(cert_der: &[u8]) -> String {
    hex_encode(Sha256::digest(cert_der).as_slice())
}

/// Constant-time compare of a certificate against an expected fingerprint.
#[must_use]
pub fn pin_matches(cert_der: &[u8], expected_hex: &str) -> bool {
    let Some(expected) = decode_sha256_hex(expected_hex) else {
        return false;
    };
    let actual = Sha256::digest(cert_der);
    let mut diff = 0u8;
    for (left, right) in actual.iter().zip(expected.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

/// rustls client that accepts only the pinned certificate.
pub fn pinned_client_config(fingerprint_hex: &str) -> IngestResult<rustls::ClientConfig> {
    install_ring();
    let expected = decode_sha256_hex(fingerprint_hex).ok_or_else(|| {
        IngestError::Tls("certificate fingerprint must be 64 hex characters".to_owned())
    })?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = Arc::new(FingerprintVerifier { expected });
    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|err| IngestError::Tls(err.to_string()))?;
    Ok(builder
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth())
}

pub(crate) fn install_ring() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[derive(Debug)]
struct FingerprintVerifier {
    expected: [u8; 32],
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if pin_matches(end_entity.as_ref(), &hex_encode(&self.expected)) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General("certificate pin mismatch".into()))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn read_identity(cert_path: &Path, key_path: &Path) -> IngestResult<TlsIdentity> {
    let cert_pem = fs::read_to_string(cert_path).map_err(|source| IngestError::TokenIo {
        path: cert_path.display().to_string(),
        source,
    })?;
    let key_pem = fs::read_to_string(key_path).map_err(|source| IngestError::TokenIo {
        path: key_path.display().to_string(),
        source,
    })?;
    let der = first_cert_der(&cert_pem)?;
    Ok(TlsIdentity {
        cert_pem,
        key_pem,
        fingerprint_hex: fingerprint_hex(&der),
    })
}

fn first_cert_der(pem: &str) -> IngestResult<Vec<u8>> {
    let mut reader = BufReader::new(pem.as_bytes());
    let mut certs = rustls_pemfile::certs(&mut reader);
    match certs.next() {
        Some(Ok(der)) => Ok(der.as_ref().to_vec()),
        Some(Err(err)) => Err(IngestError::Tls(err.to_string())),
        None => Err(IngestError::Tls(
            "LAN certificate file has no certificate".to_owned(),
        )),
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> IngestResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| IngestError::TokenIo {
                path: path.display().to_string(),
                source,
            })?;
        }
    }
    let tmp = path.with_extension("tmp");
    {
        let mut file = open_exclusive_private(&tmp).map_err(|source| IngestError::TokenIo {
            path: tmp.display().to_string(),
            source,
        })?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|source| IngestError::TokenIo {
                path: tmp.display().to_string(),
                source,
            })?;
    }
    fs::rename(&tmp, path).map_err(|source| IngestError::TokenIo {
        path: path.display().to_string(),
        source,
    })?;
    Ok(())
}

fn open_exclusive_private(path: &Path) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

fn decode_sha256_hex(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    if text.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = text.as_bytes();
    for (index, slot) in out.iter_mut().enumerate() {
        let hi = hex_val(bytes[index * 2])?;
        let lo = hex_val(bytes[index * 2 + 1])?;
        *slot = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_is_plain_http_and_lan_is_tls() {
        assert_eq!(
            transport_for_bind(Ipv4Addr::LOCALHOST),
            IngestTransport::PlainHttp
        );
        assert_eq!(
            transport_for_bind(Ipv4Addr::UNSPECIFIED),
            IngestTransport::Tls
        );
        assert_eq!(
            transport_for_bind(Ipv4Addr::new(192, 168, 1, 10)),
            IngestTransport::Tls
        );
    }

    #[test]
    fn pin_rejects_a_different_certificate() {
        let first = generate_tls_identity().expect("cert");
        let second = generate_tls_identity().expect("cert");
        let der = first_cert_der(&first.cert_pem).expect("der");
        assert!(pin_matches(&der, &first.fingerprint_hex));
        assert!(!pin_matches(&der, &second.fingerprint_hex));
        assert!(!pin_matches(&der, "abcd"));
    }

    #[test]
    fn identity_file_reloads_the_same_fingerprint() {
        let dir = std::env::temp_dir().join(format!(
            "biofocus-tls-{}-{}",
            std::process::id(),
            uuid_stub()
        ));
        let _ = fs::remove_dir_all(&dir);
        let first = load_or_create_tls_identity_at(&dir).expect("create");
        let second = load_or_create_tls_identity_at(&dir).expect("reload");
        assert_eq!(first.fingerprint_hex, second.fingerprint_hex);
        assert_eq!(first.fingerprint_hex.len(), 64);
        let _ = fs::remove_dir_all(&dir);
    }

    fn uuid_stub() -> String {
        format!("{:?}", std::time::SystemTime::now())
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }
}
