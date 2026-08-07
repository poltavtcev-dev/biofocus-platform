# QA → PM: P8-E1-T1

## Meta
- **Task ID:** P8-E1-T1
- **Title:** ADR-008: Pattern Discovery history / recompute
- **Date:** 2026-08-07
- **Dev/UX handoff:** `docs/handoffs/P8-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg -n "ADR-008" docs/decision-log.md docs/02-domain-model.md docs/04-storage.md docs/05-pipeline.md docs/09-api.md docs/16-glossary.md` — hits in all listed docs + ADR detail section.
  - `git diff --name-only -- crates/ apps/` — empty (docs-only; no migration / no production code).
  - Spot-check: no Feature/baseline `CREATE TABLE` proposed for apply; storage note says **No migration**.
- AC results:
  - **AC1 Pass** — ADR-008 recorded: chosen **recompute-on-read**; Observations / Features / Knowledge relationship table; idle/privacy (local-only, no busy-loop / always-on worker).
  - **AC2 Pass** — Rejected: persisted Feature history for v1; always-on worker; cloud sync; ML training; parallel Correlation Engine crate.
  - **AC3 Pass** — No schema to apply; deferred daily rollup only via future ADR + user approve; no migration in this task.
  - **AC4 Pass** — Knowledge sketch `focus_vs_recent_baseline_v1` in ADR detail + `docs/09-api.md` `get_insights` contracts note; calm non-clinical framing.
  - **AC5 Pass** — `docs/handoffs/P8-E1-T1-dev-to-qa.md` present.
  - **Global DoD Pass** — glossary terms; UI↛DB; no parallel crate; calm copy.
- Extra checks: out of scope respected (no history tables / recompute worker / Recommendations / CircadianOffset / SleepDebt / Dashboard redesign / PR).

## Defects (if any)
- None blocking. Note (pre-existing): `docs/04-storage.md` / `docs/05-pipeline.md` remain partially stub-wrapped; ADR notes are readable after existing content.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P8-E1-T1** to Done; Ready **P8-E2-T1** (no schema approve wait — ADR chose recompute-on-read with no migration)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` Phase 8 line (ADR-008 decided)

## Suggested next Ready task
- **P8-E2-T1** — Pattern Discovery v1 Insight path (`knowledge-engine` baseline rule per ADR-008)

## Notes for PM
- User schema approve **not** required before P8-E2 (no new tables).
- Branch for this pass: `main` (user override); PR freeze still applies — no PR.
- Optional in-process memo is allowed by ADR; do not treat it as Feature history persistence.
