# QA → PM: P23-E2-T1

## Meta
- **Task ID:** P23-E2-T1
- **Title:** Ship first slice per ADR-024: `DeskAwayPresence` (+ health→prompt)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P23-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine desk_away` — **9/9 ok** (emit walk/steps; omit quiet-alone / weak+typing; no geo keys; `register_catalog_v1`)
  - `cargo test -p report-engine` health/packs — **ok** (toml load; inject; empty none; Feature values unchanged with health)
  - Catalog §1.19 **shipped**; `register_desk_away_v1` wired
  - Health config + pack injection present; Feature crate has **no** `HealthContext` import
  - No migration / desktop UI dirt; Focus/Stress files untouched
  - Literature library **not** implemented (deferred as required)
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 DeskAwayPresence Feature + tests + catalog | **Pass** |
| AC2 Health→prompt secondary **landed** (not leftover) | **Pass** |
| AC3 Literature library deferred | **Pass** |
| AC4 Docs 12 / 16 ship notes | **Pass** |
| AC5 No diagnosis / leaf rewrite / UI / CircadianOffset / PR | **Pass** |
| AC6 Handoff states secondary landed | **Pass** |
| Global DoD | **Pass** |

- Extra checks: omit-without-positive-away locked; L5 prompt forbids inventing conditions from declared context.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P23-E2-T1** to Done; Ready **P23-E3-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — DeskAwayPresence + health→prompt shipped
- [x] Brief for **P23-E3-T1** (dogfood / optional calm UI for desk-away and/or health declare)

## Suggested next Ready task
- **P23-E3-T1** — Dogfood notes + optional calm Dashboard / health-declare surface (no formula rewrite; no GPS; no migration).

## Notes for PM
- Secondary health→prompt **shipped in E2** (not deferred).
- Phase 4 `build_report` (non-pack) path does not auto-inject health — pack path does (`build_report_with_pack`).
- PR freeze still active until 2026-09-01 — no PR.
