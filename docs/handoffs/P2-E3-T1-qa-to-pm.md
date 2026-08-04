# QA → PM: P2-E3-T1

## Meta
- **Task ID:** P2-E3-T1
- **Title:** Companion contract + minimal HealthKit sample path
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E3-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p companion` → **4 passed**
  - `cargo check --workspace --exclude desktop` → **ok**
- AC results:
  - **AC1 Pass** — sample `heart_rate` posts to loopback ingest (integration + CLI docs); reachability documented.
  - **AC2 Pass** — `post_observations` sends JSON array; contract aligned with `07` / ingest tests.
  - **AC3 Pass** — `401` → `CompanionError::Unauthorized` (test + CLI exit 3); connection refused → `Network` (test + CLI exit 2).
  - **AC4 Pass** — no Feature/dashboard/cloud code in companion.
  - **AC5 Pass** — `P2-E3-T1-dev-to-qa.md` present.
  - **Global DoD Pass** — no `unwrap`/`expect` in `apps/companion/src`; local-only.
- Extra: CI workflow includes `-p companion`.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P2-E3-T1 → Done; Ready **P2-E3-T2**
- [x] Execution canvas — QUEUE, todos, stats, callout, DAG (M1 done → M2)
- [x] Other docs: `ARCHITECTURE_STATUS.md` / `14-roadmap.md` / `12-development.md` + brief `P2-E3-T2-pm-brief.md`
- [ ] Sprint gate / push — deferred

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T1 Done; Ready **P2-E3-T2**
- [x] ARCHITECTURE_STATUS, 14-roadmap, 12-development
- [x] Execution canvas
- [x] Dev build committed (`4c4eae7`); PM docs commit follows
- [ ] Sprint gate / push — deferred

## Suggested next Ready task
- **P2-E3-T2** — Pairing UX (token share)  
  (parallel: **P2-E0-T1** sanitize `dbError`)

## Notes for PM
- **Pass with notes:** iOS is Swift stub sources, not a runnable Xcode app; Desktop still loopback-only (physical phone needs LAN later); live HealthKit/Desktop smoke not run.
- Dev commit: `4c4eae7`. No push this close.
