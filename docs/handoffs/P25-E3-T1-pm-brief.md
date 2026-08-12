# PM Brief → Dev|UX: P25-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Ready  
**Date:** 2026-08-12  
**Closed previous:** P25-E2-T1 (`SustainedLoadIndicator` Feature shipped; QA Pass)  
**Evidence:** `docs/handoffs/P25-E2-T1-qa-to-pm.md` · `docs/handoffs/P25-E2-T1-dev-to-qa.md`  
**Phase:** Phase 25 SustainedLoadIndicator — Epic P25-E3  
**Branch:** `phase/25-sustained-load`

## Task
**P25-E3-T1 — Dogfood notes + optional calm UI surface for `SustainedLoadIndicator`**

## Why
E2 shipped **`SustainedLoadIndicator`** via `register_sustained_load_v1` / `register_catalog_v1` — Snapshot/IPC already pick it up. Phase 25 closes with operator dogfood notes and an optional calm presentation (existing Dashboard Feature path / series) — **not** new math, **not** burnout / clinical diagnosis. Stay distinct from CognitiveLoad (current combined demand).

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §): how to verify `SustainedLoadIndicator` appears — fixture / path that yields StressIndex + FatigueIndex (MeetingDensity optional) over **4h lookback** → Feature snapshot / series; note **omit** when both Stress and Fatigue absent (meetings-alone must not emit); 15m/1m cadence; renormalize when MeetingDensity absent (high-level).
2. Calm framing only: “prolonged load in this window” — **not** burnout / “you are burned out” / clinical claims in docs or UI chrome. Distinct from CognitiveLoad (“combined demand in this window”).
3. **Optional** calm Dashboard surface: if existing Feature list / chart path already shows catalog Features from snapshot/series, ensure `SustainedLoadIndicator` is visible with calm label **Prolonged load** when present; empty/omit stays quiet (no error noise). Prefer reuse over redesign. Docs-only dogfood **is enough** if Feature already surfaces via existing IPC without UI gaps — state which path in handoff.
4. Smoke notes in handoff: commands / steps to see emit (unit/fixture and/or app path); confirm UI ↛ SQLite.
5. **Do not** rewrite `SustainedLoadIndicator` / leaf / sibling / CognitiveLoad formulas; **no** migration; **no** new Observation family; **no** new Insights/Recommendations rule required.
6. Handoff: `docs/handoffs/P25-E3-T1-dev-to-qa.md`.

## Out of scope
- Formula / DAG / ADR-026 math changes
- IDE plugin; weather ambient; App Store packaging product; TypingRhythm
- Workplace / manager dashboards; clinical burnout diagnosis copy
- New “SustainedLoad” product window / Dashboard redesign
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. sustained-load DoD)
- Modules: docs (+ optional desktop Feature presentation / `docs/09-api.md` if IPC gap)
- Branch: `phase/25-sustained-load`
- Personal self-tracking only
- LLM remains L5 interpret-only
- Prefer thin E3 — Phase 25 math is Done

## After QA Pass
PM → mark P25-E3-T1 Done; close Epic **P25-E3** and **Phase 25** if no further P25 tasks; next = **PM-GATE-POST-P25** (IDE · weather · App Store · Companion polish · TypingRhythm · other) **without** opening a PR during freeze.
