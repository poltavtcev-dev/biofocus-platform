# PM Brief → Dev|UX: P17-E3-T1

**From:** PM  
**To:** Dev|UX  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P17-E2-T1** — Companion HealthKit emit per ADR-018 (QA Pass)  
**Evidence:** `docs/handoffs/P17-E2-T1-qa-to-pm.md` · `docs/decision-log.md` ADR-018 · `docs/09-api.md`

## Task
**P17-E3-T1 — Chart ranges IPC/UI + Features from wearable Observations**

## Why
Contracts (E1) and Companion emits (E2) are Done. Dogfood now needs Dashboard range picker **1h / 8h / 12h / 1d / 1w** via **recompute-on-read** series IPC, Snapshot lists staying **latest**-oriented, and deterministic catalog Features from the new Observations — no LLM required, no Feature-history SQLite.

## Acceptance Criteria
1. Desktop IPC `get_feature_series` per ADR-018 / `docs/09-api.md`:  
   `range` ∈ `1h|8h|12h|1d|1w`; optional `featureIds`; returns `{ range, stepSecs, window, features }` (same Feature wire shape; multi-window series). Empty/thin history → calm empty series. **UI ↛ SQLite.**
2. Host loads Observations for the span, runs `FeatureEngine` with default coarser `stepSecs` by range (ADR-018 table; E3 may tune within same ranges). Optional in-process memo OK. **No** Feature-history table / migration.
3. `get_feature_snapshot` stays **latest**-oriented for lists / Snapshot surfaces; chart holds the **series** for the selected range.
4. Dashboard UX: closed-set range picker (1h/8h/12h/1d/1w) wired to `get_feature_series`; calm empty/loading states; no raw biometric dumps; no clinical SpO2/sleep diagnosis copy.
5. Catalog Features from wearable Observations (register via feature-engine catalog path): ship at least **`ActivityBalance`** (from `step_count` ± Life Event workout) and **`EnergyScore`** and/or **`SleepDebt`** per `docs/06-feature-catalog.md` stubs — omit calmly when inputs absent (incl. sparse SpO2; **do not** ship clinical SpO2 Features). ADR-007 confidence + explanation factors where practical.
6. Tests: series IPC empty + synthetic non-empty; Feature omit paths; no busy-loop in commands.
7. Docs: finalize `09-api` / `06-feature-catalog` / `12-development` as shipped for ranges + Features. Mark Phase 17 chart path live.
8. Handoff: `docs/handoffs/P17-E3-T1-dev-to-qa.md`.

## Out of scope
- Mi Cloud / unofficial API; Feature-history SQLite; UI→DB
- Clinical SpO2/sleep Features or diagnosis framing
- Companion HealthKit emit changes (E2 Done) unless bugfix blocking series
- IDE / weather / App Store / NotificationPressure
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-018 + ADR-008 recompute-on-read SoT
- Branch: `phase/17-wearable-charts`
- LLM L5 interpret-only — must not invent series points or Features
- Personal self-tracking; calm non-clinical copy
- SpO2 sparse on Mi — soft-omit Features that would require it

## After QA Pass
PM → mark P17-E3-T1 Done; close Phase 17 epic cluster (or gate next phase per roadmap).
