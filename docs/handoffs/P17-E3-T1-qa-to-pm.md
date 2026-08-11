# QA → PM: P17-E3-T1

## Meta
- **Task ID:** P17-E3-T1
- **Title:** Chart ranges IPC/UI + Features from wearable Observations
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P17-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine --lib` → **ok** (106)
  - `cargo test -p desktop --lib` (from `apps/desktop/src-tauri`) → **ok** (44) — includes series DTO empty/non-empty, latest-per-id collapse, `series_host` range/step/memo/synthetic ActivityBalance
  - `npx tsc --noEmit` in `apps/desktop` → **ok**
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 `get_feature_series` wire + empty calm + UI↛SQLite | **Pass** — command registered; DTO `{ range, stepSecs, window, features }`; soft-empty |
| AC2 load Observations + FeatureEngine step + memo; no Feature-history | **Pass** — `series_host`; ADR-018 default steps; memo TTL; no migration |
| AC3 Snapshot latest; chart = series | **Pass** — `latest_features_per_id` in snapshot DTO; Dashboard chart from series IPC |
| AC4 Range picker UX + calm empty/loading; no clinical dump | **Pass** — closed-set picker; placeholders; calm labels (“Sleep shortfall”) |
| AC5 ActivityBalance + EnergyScore + SleepDebt; omit paths; no SpO2 Features | **Pass** — `register_wearable_v1`; unit omit tests |
| AC6 Tests empty + synthetic; omit; no busy-loop | **Pass** |
| AC7 Docs finalized; Phase 17 chart path live | **Pass** — `09-api` / `06-feature-catalog` / `12-development` |
| Global DoD | **Pass** — prod series/catalog paths soft-fail / Result; UI via IPC only |

- Extra checks: unknown range soft-empty; optional `featureIds` filter in recompute; no Feature-history / migration; PR not opened (freeze).

## Defects (if any)
- None blocking.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P17-E3-T1** Done; close Phase 17 epic cluster (or gate next phase)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE / todos / stats / callout / DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS.md` (E3 Done; Phase 17 chart live already noted in `12-development`)

## Suggested next Ready task
- Per roadmap after Phase 17 close (PM to pick next phase / parked intent).

## Notes for PM
- Branch: `phase/17-wearable-charts`
- PR freeze until 2026-09-01 — do not open PR.
- Companion physical-device dogfood remains recommended (E2); not a blocker for E3 Pass.
