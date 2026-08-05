# PM Brief → UX (+ Dev if needed): P4-E1-T3

**From:** PM  
**To:** UX (+ Dev IPC glue if needed)  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P4-E1-T1 (QA Pass) · P4-E1-T2 (QA Pass with notes — separate Dashboard window)

## Task
**P4-E1-T3 — Recharts Feature series**

## Why
Dashboard shell is openable from Menubar with calm snapshot states and a chart placeholder. Next: real Recharts series over `get_feature_snapshot` so Focus / Stress / Fatigue are visible without Insights or LLM.

## Acceptance Criteria
1. Charts v1 from Feature snapshot: at least `FocusScore`, `StressIndex`, `FatigueIndex` (and `ContextSwitchRate` if present in snapshot).
2. Calm labels / units; no medical or evaluative claims.
3. Refresh on open and/or rare timer — no busy-loop (align with shell’s ~30s idle-safe pattern if kept).
4. Data only via existing IPC (`get_feature_snapshot`); UI ↛ SQLite.
5. Menubar alert UX unchanged; Dashboard remains the separate window from T2.
6. `tsc --noEmit` + smoke steps in handoff `docs/handoffs/P4-E1-T3-dev-to-qa.md` (mockSnapshot ready path recommended).

## Out of scope
- Insight cards / `knowledge-engine` (→ **P4-E2**)
- LLM / report UI (→ **P4-E3**)
- New SQLite tables
- Redesign of Dashboard window lifecycle (T2 design stands)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Idle-safe refresh only
- Production Rust: no new `unwrap` / `expect` if Dev touches host
- Branch: `phase/4-dashboard-ai`
- Prefer filling T2 `ChartSlot`; keep non-evaluative copy

## Shell reference (from T2)
- Window: `label: dashboard`, `?view=dashboard`
- Open: Menubar → `invoke("open_dashboard")`
- Mocks: `?view=dashboard&mockSnapshot=empty|ready|error`
- README: `apps/desktop/README.md` § Dashboard shell

## After QA Pass
PM → Ready **P4-E2-T1** (knowledge-engine skeleton + Insight types — Dev).

## Next chat (скопируй в новый чат)
```text
как агент: режим build-qa для P4-E1-T3.
Brief: docs/handoffs/P4-E1-T3-pm-brief.md
1) Как UX (+ Dev IPC glue if needed) — собери по AC, создай docs/handoffs/P4-E1-T3-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P4-E1-T3-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
