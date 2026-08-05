# PM Brief → UX + Dev: P4-E1-T2

**From:** PM  
**To:** UX + Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P4-E1-T1 (QA Pass; `get_feature_snapshot` + Core `FeatureSnapshot`)

## Task
**P4-E1-T2 — Dashboard shell (window / route)**

## Why
Feature snapshot IPC is live. Next: openable Dashboard surface from Menubar so charts (T3) and Insights (E2) have a home — shell first, no Recharts series yet.

## Acceptance Criteria
1. Dashboard view opens from Menubar (separate window **or** in-app route — pick one; document in handoff).
2. Calm shell states: **loading** / **empty** / **error** (non-evaluative copy).
3. Data only via IPC (`get_feature_snapshot` and/or existing `get_status`); UI ↛ SQLite. Mocks OK for layout if documented and real invoke wired or clearly stubbed.
4. Menubar alert UX (P3-E3-T3) still works — opening Dashboard must not break tray / alert colors.
5. Layout slots / empty chart area OK; **no** Recharts Feature series yet (→ T3).
6. Handoff `docs/handoffs/P4-E1-T2-dev-to-qa.md` with **manual smoke** steps.

## Out of scope
- Full Recharts series (→ **P4-E1-T3**)
- Insights list / `knowledge-engine` (→ **P4-E2**)
- LLM / report UI (→ **P4-E3**)
- New SQLite tables

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Idle-safe: no busy-loop poll; refresh on open or rare timer
- Production: no `unwrap` / `expect` in Rust paths touched
- Branch: `phase/4-dashboard-ai`

## IPC reference (from T1)
- `invoke("get_feature_snapshot")` — pure cache read; empty when idle
- Contract: `docs/09-api.md` · note in `docs/12-development.md`

## After QA Pass
PM → Ready **P4-E1-T3** (Recharts Feature series — UX).
