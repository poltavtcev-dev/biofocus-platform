# Dev → QA: P2-E0-T1

## Meta
- **Task ID:** P2-E0-T1
- **Title:** Sanitize IPC `dbError` paths
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P2-E0 / P2-E0-T1; PM brief `docs/handoffs/P2-E0-T1-pm-brief.md`

## What changed
- Added `StorageError::public_message()` — short, path-free strings for UI/HTTP surfaces; full `Display` unchanged for logs.
- IPC `get_status` probe (`probe_database_at` / `probe_default_database`) maps errors via `public_message`.
- Optional (done): HTTP `GET /v1/status` probe (`ingest::probe_db_at`) uses the same mapping.
- Contract note in `docs/09-api.md` (`dbError` / `db_error`: no absolute filesystem paths).
- Unit tests: storage mapping, desktop IPC payload, ingest status JSON.

## Crates / apps / files touched
- `crates/storage/src/error.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `crates/ingest/src/status.rs`
- `docs/09-api.md`
- `docs/handoffs/P2-E0-T1-dev-to-qa.md`

## How to verify (commands)
```bash
cargo test -p storage --lib error::tests
cargo test -p ingest --lib status::tests
cargo test -p desktop --lib storage_public_message
cargo test -p storage
cargo test -p ingest
cargo check -p desktop
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: UI / `get_status` does not show absolute filesystem paths in `dbError`
- [ ] AC2: Message is short and safe (`StorageError::public_message`)
- [ ] AC3: Test(s) cover the mapping (storage + desktop; ingest status optional)
- [ ] Optional: HTTP `/v1/status` `db_error` sanitized the same way
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live GUI smoke (force CreateDir failure on real host) not run — covered by unit mapping + serialize asserts.
- SQLite `Display` may still contain paths in **logs** (`tracing`); only status surfaces are sanitized.
- Custom probe closures in ingest tests that inject raw strings (`Err("…".into())`) are unchanged by design (caller-supplied message).

## Notes for QA
- Example safe messages: `"Could not create local data directory."`, `"Could not locate local data directory."`, `"Could not open local database."`.
- Assert JSON from `build_status(Err(err.public_message()))` / `StatusResponse::from_probe` has no `/Users`, `.biofocus`, or absolute path segments.
