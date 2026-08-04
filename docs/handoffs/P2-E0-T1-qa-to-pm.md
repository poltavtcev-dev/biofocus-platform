# QA → PM: P2-E0-T1

## Meta
- **Task ID:** P2-E0-T1
- **Title:** Sanitize IPC `dbError` paths
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E0-T1-dev-to-qa.md`
- **Commit reviewed:** `d877604` (`P2-E0-T1: Sanitize IPC/HTTP dbError paths via public_message.`)
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p storage --lib error::tests` → 2 passed
  - `cargo test -p ingest --lib status::tests` → 1 passed
  - `cargo test -p desktop --lib storage_public_message` → 1 passed
  - `cargo test -p storage` → 19 passed (lib + migrate + observation_repo + open)
  - `cargo test -p ingest` → 29 passed (lib + ingest_http + ingest_persist + pairing_token)
  - `cargo check -p desktop` → ok
- AC results:
  - **AC1 Pass** — `probe_database_at` / `probe_default_database` map via `public_message`; desktop unit asserts JSON has no `/Users` / `.biofocus`.
  - **AC2 Pass** — short fixed strings (`Could not create/locate/open…`); `Display` still has paths for logs only.
  - **AC3 Pass** — storage mapping tests + desktop IPC serialize assert + ingest status JSON assert.
  - **Optional Pass** — HTTP `probe_db_at` uses same mapping; contract note in `docs/09-api.md`.
  - **DoD Pass** — no `unwrap`/`expect` in prod probe paths; UI↛DB unchanged; no Observation leakage.
- Extra checks:
  - Code review: single mapping site (`StorageError::public_message`); IPC + HTTP both call it.
  - Custom inject strings in ingest tests (`Err("…".into())`) unchanged by design — not a status-probe regression.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P2-E0-T1 → Done; Epic P2-E0 Done; clear Ready / set sprint gate next
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/ARCHITECTURE_STATUS.md`, `docs/12-development.md`, `docs/14-roadmap.md` — clear open follow-up / Ready for hygiene task
- [x] Note: Phase 2 hygiene closed → **sprint gate** (push + PR → `main`) is next unless PM defers

## Suggested next Ready task
- **Sprint gate** — `docs/handoffs/SPRINT-GATE.md` + push `phase/2-ingest-http` + `gh pr create --base main`  
  (manual smokes noted in roadmap: Companion Copy/QR + window stream + companion CLI)

## Notes for PM
- Live GUI smoke (force CreateDir failure on real host) not run — unit mapping + serialize coverage accepted for this hygiene task.
- Logs/`tracing` may still contain paths via `Display` — intentional; only status surfaces sanitized.
- After Done: Phase 2 epics E0–E3 complete on branch; ready for Sprint 3–4 PR gate.
