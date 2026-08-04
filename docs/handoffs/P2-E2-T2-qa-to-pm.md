# QA → PM: P2-E2-T2

## Meta
- **Task ID:** P2-E2-T2
- **Title:** Keystroke / input aggregates (privacy-safe)
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E2-T2-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p macos-collector` → **8 passed** (3 active_window + 5 keystroke_aggregates)
  - `cargo check -p desktop` → **ok**
  - `cargo test -p desktop` → **4 passed**
  - `cargo test -p ingest` → **28 passed**
- AC results:
  - **AC1 Pass** — `data_type=keystrokes`, provider `com.biofocus.macos.input`; payload contract in `07-contracts.md`.
  - **AC2 Pass** — payload keys only `count`/`window_secs`/`rate_per_min`; tests assert no `char`/`text`/`keys`.
  - **AC3 Pass** — Accessibility documented in `10-security` §3 / `08` / `12`; untrusted probe emits nothing; SystemInputProbe fails soft.
  - **AC4 Pass** — `BIOFOCUS_INPUT_AGGREGATES` default off; host only starts when enabled.
  - **AC5 Pass** — `ingest_host` clones channel → input plugin; shutdown stops input then window; UI no SQLite.
  - **AC6 Pass** — mock emit + untrusted idle + stop → NotRunning.
  - **AC7 Pass** — `P2-E2-T2-dev-to-qa.md` present.
  - **Global DoD Pass** — no unwrap/expect in new prod paths; idle via interval + select!; glossary Observation.
- Extra: channel full drop+warn pattern mirrored from T1.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P2-E2-T2 → Done; Ready **P2-E2-T3** (∥ P2-E0-T1)
- [x] Execution canvas — QUEUE, todos, stats, callout, DAG (C2 done → C3)
- [x] Other docs: `ARCHITECTURE_STATUS.md`, `docs/14-roadmap.md`, `docs/12-development.md` status

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T2 Done; Ready **P2-E2-T3**; brief `docs/handoffs/P2-E2-T3-pm-brief.md`
- [x] ARCHITECTURE_STATUS, 14-roadmap, 12-development
- [x] Execution canvas
- [x] Dev build already committed (`a7fe6f7`); PM docs commit follows
- [ ] Sprint gate / push — **not yet** (E2-T3 still open)

## Suggested next Ready task
- **P2-E2-T3** — Collector integration tests + pause idle  
  (parallel: **P2-E0-T1**)

## Notes for PM
- **Pass with notes:** live Accessibility + GUI smoke not run (same class as T1 mock coverage). Recommend before sprint PR if enabling the flag.
- Dev already committed build (`a7fe6f7`) per new git policy.
- Sprint **not** closed — no push/PR yet (E2-T3 + E3 still open).
