//! Storage open / pragma tests.

use storage::Database;

#[test]
fn open_temp_file_enables_wal_and_normal_sync() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("biofocus_main.db");

    let db = Database::open(&path).expect("open db");
    assert_eq!(db.path(), path.as_path());
    assert_eq!(db.journal_mode().expect("journal_mode"), "wal");
    assert_eq!(db.synchronous().expect("synchronous"), 1);
}

#[test]
fn open_creates_missing_parent_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("nested").join("data").join("biofocus_main.db");

    let db = Database::open(&path).expect("open nested path");
    assert!(path.exists());
    assert_eq!(db.journal_mode().expect("journal_mode"), "wal");
}

#[test]
fn in_memory_applies_synchronous_normal() {
    let db = Database::open_in_memory().expect("in-memory");
    assert_eq!(db.synchronous().expect("synchronous"), 1);
}
