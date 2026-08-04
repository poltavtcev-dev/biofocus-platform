//! Pairing token generation and local persistence under `~/.biofocus/`.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{IngestError, IngestResult};

/// Directory name under `$HOME` for BioFocus local state (`docs/04-storage.md`).
pub const BIOFOCUS_DIR: &str = ".biofocus";

/// File name for the ingest pairing Bearer token (under [`BIOFOCUS_DIR`]).
pub const PAIRING_TOKEN_FILE: &str = "pairing_token";

/// Optional override for the BioFocus home directory (tests / custom layouts).
///
/// When set, the token path is `$BIOFOCUS_HOME/pairing_token` instead of
/// `$HOME/.biofocus/pairing_token`.
pub const BIOFOCUS_HOME_ENV: &str = "BIOFOCUS_HOME";

/// Environment variable that overrides the on-disk pairing token.
pub const INGEST_TOKEN_ENV: &str = "BIOFOCUS_INGEST_TOKEN";

/// Number of random bytes used for a newly generated pairing token.
const TOKEN_BYTES: usize = 32;

/// Returns the default on-disk path for the pairing token.
///
/// Resolution:
/// 1. `$BIOFOCUS_HOME/pairing_token` if `BIOFOCUS_HOME` is set and non-empty
/// 2. otherwise `$HOME/.biofocus/pairing_token`
pub fn default_pairing_token_path() -> IngestResult<PathBuf> {
    if let Ok(home) = env::var(BIOFOCUS_HOME_ENV) {
        let home = home.trim();
        if !home.is_empty() {
            return Ok(PathBuf::from(home).join(PAIRING_TOKEN_FILE));
        }
    }

    let home = env::var_os("HOME").ok_or(IngestError::HomeDirUnavailable)?;
    Ok(PathBuf::from(home)
        .join(BIOFOCUS_DIR)
        .join(PAIRING_TOKEN_FILE))
}

/// Generates a cryptographically strong pairing token (64 hex chars).
pub fn generate_pairing_token() -> IngestResult<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|err| IngestError::TokenEntropy(err.to_string()))?;
    Ok(hex_encode(&bytes))
}

/// Loads an existing token from `path`, or generates and persists a new one.
pub fn load_or_create_pairing_token(path: &Path) -> IngestResult<String> {
    if path.exists() {
        return read_token_file(path);
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| IngestError::TokenIo {
                path: path.display().to_string(),
                source,
            })?;
        }
    }

    let token = generate_pairing_token()?;
    write_token_file(path, &token)?;
    Ok(token)
}

/// Resolves the ingest Bearer token.
///
/// Priority:
/// 1. Non-empty `BIOFOCUS_INGEST_TOKEN` env (dev / CI override)
/// 2. Load or create the file at [`default_pairing_token_path`]
pub fn resolve_ingest_token() -> IngestResult<String> {
    if let Ok(token) = env::var(INGEST_TOKEN_ENV) {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_owned());
        }
    }

    let path = default_pairing_token_path()?;
    load_or_create_pairing_token(&path)
}

fn read_token_file(path: &Path) -> IngestResult<String> {
    let raw = fs::read_to_string(path).map_err(|source| IngestError::TokenIo {
        path: path.display().to_string(),
        source,
    })?;
    let token = raw.trim();
    if token.is_empty() {
        return Err(IngestError::EmptyTokenFile {
            path: path.display().to_string(),
        });
    }
    Ok(token.to_owned())
}

fn write_token_file(path: &Path, token: &str) -> IngestResult<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut file = open_exclusive_private(&tmp).map_err(|source| IngestError::TokenIo {
            path: tmp.display().to_string(),
            source,
        })?;
        file.write_all(token.as_bytes())
            .and_then(|()| file.write_all(b"\n"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes env-mutating tests in this module.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn generate_token_is_64_hex_chars() {
        let token = generate_pairing_token().expect("entropy");
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn load_or_create_persists_and_reloads() {
        let dir = std::env::temp_dir().join(format!(
            "biofocus-pairing-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(PAIRING_TOKEN_FILE);

        let first = load_or_create_pairing_token(&path).expect("create");
        let second = load_or_create_pairing_token(&path).expect("reload");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);

        let on_disk = fs::read_to_string(&path).expect("read");
        assert_eq!(on_disk.trim(), first);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_prefers_env_override() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let dir = std::env::temp_dir().join(format!(
            "biofocus-pairing-env-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&dir).expect("temp dir");

        // SAFETY: serialized by ENV_LOCK; restored before unlock.
        unsafe {
            env::set_var(BIOFOCUS_HOME_ENV, &dir);
            env::set_var(INGEST_TOKEN_ENV, "env-override-token");
        }

        let token = resolve_ingest_token().expect("resolve");
        assert_eq!(token, "env-override-token");
        assert!(!dir.join(PAIRING_TOKEN_FILE).exists());

        unsafe {
            env::remove_var(INGEST_TOKEN_ENV);
            env::remove_var(BIOFOCUS_HOME_ENV);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_loads_from_biofocus_home() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let dir = std::env::temp_dir().join(format!(
            "biofocus-pairing-home-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&dir).expect("temp dir");

        unsafe {
            env::remove_var(INGEST_TOKEN_ENV);
            env::set_var(BIOFOCUS_HOME_ENV, &dir);
        }

        let first = resolve_ingest_token().expect("create");
        let second = resolve_ingest_token().expect("reload");
        assert_eq!(first, second);
        assert!(dir.join(PAIRING_TOKEN_FILE).exists());

        unsafe {
            env::remove_var(BIOFOCUS_HOME_ENV);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_token_file_errors() {
        let dir = std::env::temp_dir().join(format!(
            "biofocus-pairing-empty-{}-{}",
            std::process::id(),
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(PAIRING_TOKEN_FILE);
        fs::write(&path, "   \n").expect("write");

        let err = load_or_create_pairing_token(&path).expect_err("empty");
        assert!(matches!(err, IngestError::EmptyTokenFile { .. }));

        let _ = fs::remove_dir_all(&dir);
    }
}
