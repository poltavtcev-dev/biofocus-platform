# PM Brief → Dev|UX: P20-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P20-E3** ✅ · **Phase 20** ✅  
**Evidence:** `docs/handoffs/P20-E3-T1-qa-to-pm.md` · `docs/handoffs/P20-E3-T1-dev-to-qa.md`  
**Next:** **PM-GATE-POST-P20** — `docs/handoffs/PM-GATE-POST-P20-pm-brief.md`

## Task
**P20-E3-T1 — Dogfood notes + optional calm Dashboard surface for `CognitiveLoad`**

## Why
E2 shipped `CognitiveLoad` via `register_cognitive_v1` / `register_catalog_v1` — Snapshot/IPC already pick it up. Phase 20 closes with operator dogfood notes and an optional calm presentation (existing Dashboard Feature path / series) — **not** new math, **not** clinical framing.

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §): how to verify `CognitiveLoad` appears — fixture / ingest of leaf inputs **or** live MeetingDensity + CSR (+ optional NotificationPressure) → Feature snapshot / series; note partial emit when notifications opt-in is off.
2. Calm framing only: “combined demand in this window” — **not** overloaded / burnout / ADHD / clinical cognitive-load diagnosis in docs or UI chrome.
3. **Optional** calm Dashboard surface: if existing Feature list / chart path already shows catalog Features from snapshot/series, ensure `CognitiveLoad` is visible with calm label when present; empty/omit stays quiet (no error noise). Prefer reuse over redesign. Docs-only dogfood **is enough** if Feature already surfaces via existing IPC without UI gaps — state which path in handoff.
4. Smoke notes in handoff: commands / steps to see emit (unit/fixture and/or app path); confirm UI ↛ SQLite.
5. **Do not** rewrite `CognitiveLoad` / leaf Feature formulas; **no** migration; **no** new Observation family; **no** new Insights/Recommendations rule required.
6. Handoff: `docs/handoffs/P20-E3-T1-dev-to-qa.md`.

## Out of scope
- Formula / DAG / ADR-021 math changes
- IDE plugin; weather ambient; App Store packaging product
- Workplace / manager dashboards; clinical diagnosis copy
- New “CognitiveLoad” product window / Dashboard redesign
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: docs (+ optional desktop Feature presentation / `docs/09-api.md` if IPC gap)
- Branch: `phase/20-cognitive-load`
- Personal self-tracking only
- LLM remains L5 interpret-only
- Prefer thin E3 — Phase 20 math is Done

## After QA Pass
PM → mark P20-E3-T1 Done; close Epic **P20-E3** and **Phase 20** if no further P20 tasks; next = **PM-GATE-POST-P20** (IDE · weather · App Store · polish) **without** opening a PR during freeze.
