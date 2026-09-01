# PM Brief → Dev|UX: P8-E3-T1

**From:** PM  
**To:** Dev (+ UX)  
**Status:** Ready  
**Date:** 2026-08-07  
**Closed previous:** P8-E2-T1 (QA Pass with notes — `focus_vs_recent_baseline_v1` + `pattern_host`); Epic P8-E2 ✅  
**Evidence:** `docs/handoffs/P8-E2-T1-qa-to-pm.md`

## Task
**P8-E3-T1 — Insights IPC / UX for patterns**

## Why
Core already evaluates `focus_vs_recent_baseline_v1` on `get_insights` (ADR-008 recompute-on-read). Phase 8 closes when pattern Insights are calmly visible on the existing Dashboard Insights path — no new product window, no UI→DB.

## Acceptance Criteria
1. Pattern / baseline Insights from Core appear via the existing Insights IPC path (`get_insights` → Dashboard Insights list) with calm non-clinical copy (title/description as returned; no clinical framing in UI chrome).
2. Empty / thin-history state stays calm (omit or empty list — no error noise when baseline rule does not emit).
3. Optional UX polish only if needed for readability: e.g. category/`pattern` affordance that fits existing Insights list — **not** a redesign.
4. Smoke notes in handoff: how to see a pattern Insight (mock and/or dogfood with multi-day FocusScore data); confirm UI still does not touch SQLite.
5. Handoff: `docs/handoffs/P8-E3-T1-dev-to-qa.md`.

## Out of scope
- New “Pattern Discovery” product window / full Dashboard redesign
- Cloud sync; Recommendations (P9)
- Changing ADR-008 mechanics, afternoon UTC bucket, δ / confidence constants (document-only if copy needs a hint)
- Feature-history SQLite tables
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: desktop Insights path / `docs/09-api.md` (+ README mock notes if useful)
- Branch: `phase/8-pattern-discovery`
- Core path already wired in P8-E2 — prefer UI/copy/smoke over re-plumbing unless a real IPC gap appears

## After QA Pass
PM → mark P8-E3-T1 Done; close Epic **P8-E3** and **Phase 8** Kanban if no further P8 tasks — **without** opening a PR during freeze.
