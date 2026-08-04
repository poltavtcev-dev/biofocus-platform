# QA → PM: P2-E2-T1

## Meta
- **Task ID:** P2-E2-T1
- **Title:** Active window Observation stream
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E2-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p macos-collector` → **3 passed** (`observation_payload_is_metadata_only`, `emits_only_on_frontmost_change_then_stops`, `stop_without_busy_spin_when_idle`)
  - `cargo test -p plugin-sdk` → **0 tests** (traits only; OK)
  - `cargo check -p desktop` → **ok**
  - `cargo test -p desktop` → **4 passed**
  - `cargo test -p ingest` → **28 passed** (8 unit + 15 http + 2 persist + 3 pairing)
- AC results (pass/fail per item):
  - **AC1 Pass** — `data_type = context_window`, provider `com.biofocus.macos.context`; matches `docs/04-storage.md` / `docs/07-contracts.md`. `ActiveWindowPlugin` + `NSWorkspace` frontmost on macOS; mock probe in tests.
  - **AC2 Pass** — payload keys only `bundle_id` + `app_name`; test asserts no `window_title` / `keystrokes`. Docs: `07-contracts`, `10-security` §3 — no Accessibility for T1.
  - **AC3 Pass** — `DEFAULT_POLL_INTERVAL = 1s`; `tokio::interval` + `select!` (biased stop); emit only on `identity_key` change. No busy-loop.
  - **AC4 Pass** — `stop_stream` → oneshot + join; host `IngestHost::shutdown` stops collector before HTTP/worker. Test `stop_without_busy_spin_when_idle` + second stop → `NotRunning`.
  - **AC5 Pass** — `ingest_host`: `tx.clone()` → collector; original `tx` → `IngestState` / HTTP; `rx` → persist worker. UI: no SQLite in `apps/desktop/src`.
  - **AC6 Pass** — mock `ScriptedProbe`: emit on change + stop idle covered.
  - **AC7 Pass** — `docs/handoffs/P2-E2-T1-dev-to-qa.md` present.
  - **Global DoD Pass** — no `unwrap`/`expect` in `macos-collector` / `plugin-sdk` prod paths (only `unwrap_or` / `unwrap_or_default` for time / Optional NSString). Glossary `Observation` used correctly. Idle: interval wait + `select!`.
- Extra checks (edge / security):
  - Channel full → drop + warn (no panic); channel closed → loop exits.
  - Non-macOS probe → `None` (no emissions) — expected.
  - Privacy: NSWorkspace only; no window title / Accessibility — documented.
  - Docs touch (`07`, `08`, `10`, `12`) align with ship.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P2-E2-T1** to Done; set Ready to **P2-E2-T2**; keep ∥ **P2-E0-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG (E2 T1 done → T2)
- [x] Other docs: `ARCHITECTURE_STATUS.md`, `docs/14-roadmap.md`, `docs/12-development.md` status line

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T1 Done; Ready **P2-E2-T2**; brief `docs/handoffs/P2-E2-T2-pm-brief.md`
- [x] ARCHITECTURE_STATUS, 14-roadmap, 12-development
- [x] Execution canvas
- [ ] Dev: **commit P2-E2-T1** on `phase/2-ingest-http` (uncommitted at QA time)
- [ ] Before sprint PR: one manual GUI→SQLite smoke for `context_window`

## Suggested next Ready task
- **P2-E2-T2** — Keystroke / input aggregates (privacy-safe)  
  (parallel optional: **P2-E0-T1**)

## Notes for PM
- **Pass with notes** only because full GUI smoke (`pnpm tauri dev` → switch apps → SQLite `context_window`) was **not** executed in this QA pass (same as Dev: mock covers AC logic). Recommend one manual smoke before/at sprint PR.
- First poll tick may emit immediately after start (baseline) — by design; documented in Dev handoff.
- Working tree still **uncommitted** for this task at QA time — commit after PM Done (per-task policy).
- `plugin-sdk` has no unit tests yet (trait surface only) — acceptable for T1; deeper coverage lives in `macos-collector`.
