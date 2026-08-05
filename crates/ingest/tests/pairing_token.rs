//! Pairing token persistence: generate → disk → reload (temp dir).

use std::fs;
use std::sync::Mutex;

use ingest::{
    default_pairing_token_path, load_or_create_pairing_token, resolve_ingest_token, IngestConfig,
    BIOFOCUS_HOME_ENV, INGEST_BIND_HOST, INGEST_BIND_HOST_ENV, INGEST_LAN_BIND_HOST, INGEST_LAN_ENV,
    INGEST_TOKEN_ENV, PAIRING_TOKEN_FILE,
};
use uuid::Uuid;

/// Serializes env-mutating tests in this file.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn generate_persist_reload_round_trip() {
    let dir = std::env::temp_dir().join(format!(
        "biofocus-pairing-it-{}-{}",
        std::process::id(),
        Uuid::now_v7()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(PAIRING_TOKEN_FILE);

    let created = load_or_create_pairing_token(&path).expect("create");
    assert_eq!(created.len(), 64);
    assert!(created.chars().all(|c| c.is_ascii_hexdigit()));

    let reloaded = load_or_create_pairing_token(&path).expect("reload");
    assert_eq!(created, reloaded);

    let from_disk = fs::read_to_string(&path).expect("read");
    assert_eq!(from_disk.trim(), created);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn config_load_uses_biofocus_home_file() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let dir = std::env::temp_dir().join(format!(
        "biofocus-pairing-cfg-{}-{}",
        std::process::id(),
        Uuid::now_v7()
    ));
    fs::create_dir_all(&dir).expect("temp dir");

    // SAFETY: serialized by ENV_LOCK; restored before unlock.
    unsafe {
        std::env::remove_var(INGEST_TOKEN_ENV);
        std::env::remove_var(INGEST_LAN_ENV);
        std::env::remove_var(INGEST_BIND_HOST_ENV);
        std::env::set_var(BIOFOCUS_HOME_ENV, &dir);
    }

    let cfg1 = IngestConfig::load().expect("load");
    let cfg2 = IngestConfig::load().expect("reload");
    assert_eq!(cfg1.token, cfg2.token);
    assert_eq!(cfg1.port, ingest::DEFAULT_INGEST_PORT);
    assert_eq!(cfg1.bind_host, INGEST_BIND_HOST);
    assert!(!cfg1.is_lan_bind());
    assert!(dir.join(PAIRING_TOKEN_FILE).exists());

    let path = default_pairing_token_path().expect("path");
    assert_eq!(path, dir.join(PAIRING_TOKEN_FILE));

    unsafe {
        std::env::remove_var(BIOFOCUS_HOME_ENV);
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn env_override_skips_disk_token() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let dir = std::env::temp_dir().join(format!(
        "biofocus-pairing-override-{}-{}",
        std::process::id(),
        Uuid::now_v7()
    ));
    fs::create_dir_all(&dir).expect("temp dir");

    unsafe {
        std::env::set_var(BIOFOCUS_HOME_ENV, &dir);
        std::env::set_var(INGEST_TOKEN_ENV, "ci-override-token");
    }

    let token = resolve_ingest_token().expect("resolve");
    assert_eq!(token, "ci-override-token");
    assert!(!dir.join(PAIRING_TOKEN_FILE).exists());

    unsafe {
        std::env::remove_var(INGEST_TOKEN_ENV);
        std::env::remove_var(BIOFOCUS_HOME_ENV);
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn config_load_lan_opt_in_sets_unspecified_bind() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let dir = std::env::temp_dir().join(format!(
        "biofocus-pairing-lan-{}-{}",
        std::process::id(),
        Uuid::now_v7()
    ));
    fs::create_dir_all(&dir).expect("temp dir");

    unsafe {
        std::env::remove_var(INGEST_TOKEN_ENV);
        std::env::remove_var(INGEST_BIND_HOST_ENV);
        std::env::set_var(BIOFOCUS_HOME_ENV, &dir);
        std::env::set_var(INGEST_LAN_ENV, "1");
    }

    let cfg = IngestConfig::load().expect("load");
    assert_eq!(cfg.bind_host, INGEST_LAN_BIND_HOST);
    assert!(cfg.is_lan_bind());

    unsafe {
        std::env::remove_var(INGEST_LAN_ENV);
        std::env::remove_var(BIOFOCUS_HOME_ENV);
    }
    let _ = fs::remove_dir_all(&dir);
}
