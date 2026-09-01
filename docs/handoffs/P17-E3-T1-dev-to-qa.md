# Dev|UX → QA: P17-E3-T1

## Meta
- **Task ID:** P17-E3-T1
- **Title:** Chart ranges IPC/UI + Features from wearable Observations
- **Role that built:** Dev|UX
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P17-E3-T1-pm-brief.md` / ADR-018

## What changed
- Desktop IPC `get_feature_series` — recompute-on-read for closed-set ranges `1h|8h|12h|1d|1w` with ADR-018 default `stepSecs`; optional `featureIds` filter; soft-empty on thin history / unknown range; optional in-process memo; **no** Feature-history SQLite.
- `FeatureEngine::run_with_step` + catalog nodes honor context step; tip Observation covered when `max_ts` is off-step.
- `get_feature_snapshot` collapses to **latest Feature per `featureId`** (lists); chart uses series IPC.
- Catalog wearable Features via `register_wearable_v1` / `register_catalog_v1`: **`ActivityBalance`**, **`EnergyScore`**, **`SleepDebt`** (omit when inputs absent; no SpO2 Features; ADR-007 confidence + factors).
- Dashboard range picker wired to `get_feature_series`; calm loading/empty; Snapshot list still from snapshot IPC.
- Docs finalized: `09-api`, `06-feature-catalog`, `12-development` (Phase 17 chart path live).

## Crates / apps / files touched
- `crates/feature-engine/` — window step, `run_with_step`, `activity_balance` / `energy_score` / `sleep_debt`, catalog registration
- `apps/desktop/src-tauri/src/series_host.rs` — range parse, load Observations, recompute, memo, latest-per-id
- `apps/desktop/src-tauri/src/lib.rs` — `get_feature_series` command + DTO tests
- `apps/desktop/src/` — `featureSeries.ts`, `featureChart.ts`, `Dashboard.tsx`, `App.css`
- `docs/06-feature-catalog.md`, `docs/09-api.md`, `docs/12-development.md`

## How to verify (commands)
```bash
cargo test -p feature-engine --lib
cd apps/desktop/src-tauri && cargo test --lib
# optional UI typecheck
cd apps/desktop && npx tsc --noEmit
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `get_feature_series` range ∈ closed set; returns `{ range, stepSecs, window, features }`; empty → calm `[]`; UI ↛ SQLite
- [ ] AC2: Host loads Observations for span; FeatureEngine with default coarser `stepSecs`; optional memo OK; no Feature-history table/migration
- [ ] AC3: `get_feature_snapshot` latest-oriented; chart holds series for selected range
- [ ] AC4: Dashboard range picker 1h/8h/12h/1d/1w; calm empty/loading; no raw biometrics / clinical SpO2/sleep diagnosis copy
- [ ] AC5: `ActivityBalance` + `EnergyScore` + `SleepDebt` shipped; omit when inputs absent; no clinical SpO2 Features
- [ ] AC6: Tests — series empty + synthetic non-empty; Feature omit paths; no busy-loop
- [ ] AC7: Docs shipped for ranges + Features; Phase 17 chart path live
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Physical-device Companion dogfood not re-run in this task (E2 Done).
- Stress `High_Stress` contiguous detection still uses catalog `STEP_SECS` constant (series IPC does not return Signals).
- Chart shows many score series when present — legend can get dense on rich data.

## Notes for QA
- Branch: `phase/17-wearable-charts`
- PR freeze — do not open PR.
- Synthetic series test: `series_host::tests::recompute_synthetic_steps_emits_activity_balance`
