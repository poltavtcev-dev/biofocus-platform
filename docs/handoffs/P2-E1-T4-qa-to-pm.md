# QA → PM: P2-E1-T4

## Meta
- **Task ID:** P2-E1-T4
- **Title:** Host wire + `GET /v1/status` + idle DoD
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E1-T4-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p ingest` → **28 passed** (unit + ingest_http + ingest_persist + pairing_token)
  - `cargo test -p desktop` → **4 passed**
  - `cargo check -p ingest` / `cargo check -p desktop` → **ok**
- AC results (pass/fail per item):
  - **AC1 Pass** — `ingest_host::start_ingest_host`: `IngestConfig::load` → default DB path → `observation_channel` → `Database::open` + `spawn_persist_worker` (or discard drain) → `serve_with_shutdown`; bind via `bind_loopback` / `INGEST_BIND_HOST` = `127.0.0.1`, default port **8787**. Wired from Tauri `setup`.
  - **AC2 Pass** — `RunEvent::ExitRequested` → `stop_ingest_host` → oneshot shutdown + wait server_done (3s) + `JoinHandle` for worker. No orphan busy-loop in design.
  - **AC3 Pass** — `GET /v1/status` → `StatusResponse` `{version, db_status, db_error?}`; tests assert no `observations` / `payload` / `hrv`. Soft probe → `db_status: "error"`.
  - **AC4 Pass** — Axum/tokio accept idle; persist/discard worker uses `blocking_recv`.
  - **AC5 Pass** — UI `apps/desktop/src/coreStatus.ts` still `invoke("get_status")` only; docs state HTTP status is companion/debug.
  - **AC6 Pass** — `get_status_ok_shape_has_no_observation_fields`, `get_status_maps_db_probe_error_soft`, `get_status_probes_real_temp_db_path`, `status_response_unit_mapping`; manual smoke documented in Dev handoff + `docs/12-development.md`.
  - **AC7 Pass** — `docs/handoffs/P2-E1-T4-dev-to-qa.md` present.
  - **Global DoD Pass** — no `unwrap`/`expect` on prod paths in ingest host / status / routes / server; UI↛DB; glossary (`Observation`) used correctly.
- Extra checks (edge / security):
  - Loopback-only bind unchanged; `/v1/status` auth: none (documented, loopback-only).
  - Soft-fail: config/DB path/channel failure → log + no panic; DB open fail → discard drain + HTTP still up with soft status error.
  - HTTP JSON snake_case vs IPC camelCase — intentional, documented.
  - Docs touch: `09-api.md`, `12-development.md`, `10-security.md` align with ship.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P2-E1-T4** to Done; mark Epic **P2-E1** Done (T1–T4); set Ready to **P2-E2-T1** (optionally note ∥ **P2-E0-T1**)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG (E1 complete → E2)
- [x] Other docs: `ARCHITECTURE_STATUS.md`, `docs/14-roadmap.md`; brief `docs/handoffs/P2-E2-T1-pm-brief.md`; E1 sprint PR recommended

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T4 Done; Epic E1 ✅; Ready **P2-E2-T1**
- [x] ARCHITECTURE_STATUS, 14-roadmap
- [x] Brief `docs/handoffs/P2-E2-T1-pm-brief.md`
- [x] Execution canvas
- [x] Commit policy: **per task** on branch; **PR per sprint** (`docs/12-development.md`)
- [ ] Optional: one manual `tauri dev` + curl `/v1/status` smoke before sprint PR

## Suggested next Ready task
- **P2-E2-T1** — Active window Observation stream  
  (parallel optional: **P2-E0-T1** sanitize IPC `dbError`)

## Notes for PM
- **Pass with notes** only because full Tauri GUI smoke (`pnpm tauri dev` + curl + quit) was **not** executed in this QA pass (same as Dev: manual-only, not in CI). Host lifecycle verified by code review + crate tests. Recommend one manual smoke before/at sprint PR.
- No automated unit test specifically for `IngestHost::shutdown` join path — acceptable for T4 scope; optional follow-up if flake appears.
- `db_error` on HTTP status still uses `err.to_string()` (may include paths) — same family as IPC; sanitize remains **P2-E0-T1** (IPC-focused); consider extending E0 to HTTP if desired.
- Branch `phase/2-ingest-http`: T4 changes currently **uncommitted** in working tree at QA time — PM/Dev should commit before PR batch.
- After PM close: Epic E1 complete → collectors (E2) or hygiene (E0).