# QA → PM: P5-E1-T1

## Meta
- **Task ID:** P5-E1-T1
- **Title:** Opt-in LAN ingest bind + config
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P5-E1-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p ingest` → **ok** (17 unit + 17 http + 2 persist + 4 pairing)
  - `cargo check -p desktop` → **ok** (earlier in build-qa)
  - `cargo test -p companion --test sample_ingest` → **ok** (4)
- AC results:
  - **AC1 Pass** — default / `IngestConfig::default` / `load` without knobs → `127.0.0.1`; `bind_loopback` unchanged
  - **AC2 Pass** — `BIOFOCUS_INGEST_LAN=1` → `0.0.0.0`; `BIOFOCUS_INGEST_BIND_HOST` override; documented in `10-security` §1.1 + `12-development`
  - **AC3 Pass** — `live_server_on_lan_bind_still_requires_bearer` (401 without token, 202 with Bearer on LAN bind)
  - **AC4 Pass** — without opt-in, resolve/bind stay loopback; existing loopback live test green
  - **AC5 Pass** — unit resolve tests + `bind_lan_opt_in_*` + `config_load_lan_opt_in_*` (ephemeral port)
  - **AC6 Pass** — still `axum::serve` / tokio accept (no spin introduced)
  - **AC7 Pass** — `docs/10-security.md`, `docs/12-development.md`, `docs/09-api.md` bind note
  - **AC8 Pass** — **ADR-005** in `docs/decision-log.md`
  - **AC9 Pass** — no schema / no Companion UI / iOS changes in this task
- Extra checks:
  - Production paths: no new `unwrap`/`expect` outside `#[cfg(test)]`
  - Invalid `BIOFOCUS_INGEST_BIND_HOST` → typed `IngestError::InvalidBindHost`

## Defects (if any)
- None blocking.

## Notes (non-blocking)
1. Pairing IPC `ingestBaseUrl` still hardcodes loopback even when LAN opt-in is on → expected follow-up **P5-E1-T2**.
2. `GET /v1/status` remains unauthenticated (pre-existing); with LAN opt-in it is LAN-reachable without Bearer — still no Observation payload; noted in `09-api.md`.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P5-E1-T1 → Done; Ready **P5-E1-T2**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS.md` (Phase 5 bind note); ensure Phase 5 Kanban open docs from pm-brief are on the branch (`phase/5-wearable-dogfood`)
- [ ] Optional: fold PM Phase-5-open stash (`phase5-pm-open-wip`) if not yet on this branch

## Suggested next Ready task
- **P5-E1-T2** — Advertise bind mode + base URL hints

## Notes for PM
- Branch: `phase/5-wearable-dogfood` (from `origin/main`). Code + AC docs landed; Kanban/canvas not touched by Dev/QA.
- Cluster PR later with Phase 5 E1 (or when user asks) — not required to mark Done.
