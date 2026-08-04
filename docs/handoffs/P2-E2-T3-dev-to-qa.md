# Dev → QA: P2-E2-T3

## Meta
- **Task ID:** P2-E2-T3
- **Title:** Collector integration tests + pause idle
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E2-T3; brief `docs/handoffs/P2-E2-T3-pm-brief.md`

## What changed
- Added `crates/macos-collector/tests/collector_integration.rs`:
  - `context_window` and `keystrokes` emit → bounded channel → `spawn_persist_worker` → SQLite (temp DB).
  - After `stop_stream`, frontmost / input probe call counts freeze (no busy-loop).
- Dev-deps on `ingest`, `storage`, `tempfile` for the persist path (same worker as desktop host).
- Documented suite in `docs/12-development.md` (§ Collector test suite) and layer note in `docs/11-testing.md`.
- No T1/T2 payload contract changes; pause = `stop_stream` (no separate pause API on `BioFocusPlugin`).

## How to verify (commands)
```bash
cargo test -p macos-collector
cargo test -p macos-collector --test collector_integration
cargo check -p macos-collector
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Integration tests emit Observation for active window + opt-in keystrokes into channel/storage
- [ ] AC2: Pause/stop — after `stop_stream` no periodic probe work (call counts freeze; join clean)
- [ ] AC3: `docs/12-development.md` (and `11-testing` if present) documents how to run the suite
- [ ] AC4: This handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; idle footprint; glossary terms

## Risks / not covered
- Live NSWorkspace / Accessibility GUI smoke still deferred to sprint PR (mocks only).
- Collectors have no distinct `pause` vs `stop` — both map to `stop_stream`.
- Persist path reuses `ingest::spawn_persist_worker` (not a separate desktop host hook).

## Notes for QA
- All four integration tests should pass on Linux CI (mock probes; no macOS APIs).
- Existing unit suites `active_window` + `keystroke_aggregates` remain green.

## Git
Commit after this handoff per agent git policy (no push until sprint gate).
