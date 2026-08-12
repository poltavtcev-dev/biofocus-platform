# PM Brief → Dev|UX: P24-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Done (2026-08-12) — QA Pass · Phase 24 closed · next **PM-GATE-POST-P24**  
**Date:** 2026-08-12  
**Closed previous:** P24-E2-T1 (`CircadianOffset` Feature shipped; QA Pass)  
**Evidence:** `docs/handoffs/P24-E2-T1-qa-to-pm.md` · `docs/handoffs/P24-E2-T1-dev-to-qa.md`  
**Phase:** Phase 24 CircadianOffset — Epic P24-E3  
**Branch:** `phase/24-circadian-offset`

## Task
**P24-E3-T1 — Dogfood notes + optional calm UI surface for `CircadianOffset`**

## Why
E2 shipped **`CircadianOffset`** via `register_circadian_v1` / `register_catalog_v1` — Snapshot/IPC already pick it up. Phase 24 closes with operator dogfood notes and an optional calm presentation (existing Dashboard Feature path / series) — **not** new math, **not** chronotype diagnosis. Stay distinct from SleepDebt (debt magnitude) and DeskAwayPresence (away-from-desk).

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §): how to verify `CircadianOffset` appears — fixture / ingest of qualifying `sleep_interval` + work/activity timing (`keystrokes` / `context_window`, optional activity reinforcement) → Feature snapshot / series; note omit unless both slots; 15m/1m + 24h lookback; align-near-mid-wake vs large circular offset behavior (high-level).
2. Calm framing only: “schedule alignment in this window” — **not** chronotype / circadian-disorder / “night owl so you fail” in docs or UI chrome. Distinct from SleepDebt and DeskAwayPresence.
3. **Optional** calm Dashboard surface: if existing Feature list / chart path already shows catalog Features from snapshot/series, ensure `CircadianOffset` is visible with calm label **Schedule alignment** when present; empty/omit stays quiet (no error noise). Prefer reuse over redesign. Docs-only dogfood **is enough** if Feature already surfaces via existing IPC without UI gaps — state which path in handoff.
4. Smoke notes in handoff: commands / steps to see emit (unit/fixture and/or app path); confirm UI ↛ SQLite.
5. **Do not** rewrite `CircadianOffset` / leaf / sibling formulas; **no** migration; **no** new Observation family; **no** new Insights/Recommendations rule required.
6. Handoff: `docs/handoffs/P24-E3-T1-dev-to-qa.md`.

## Out of scope
- Formula / DAG / ADR-025 math changes
- IDE plugin; weather ambient; App Store packaging product; TypingRhythm
- Workplace / manager dashboards; clinical diagnosis copy
- New “CircadianOffset” product window / Dashboard redesign
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. circadian DoD)
- Modules: docs (+ optional desktop Feature presentation / `docs/09-api.md` if IPC gap)
- Branch: `phase/24-circadian-offset`
- Personal self-tracking only
- LLM remains L5 interpret-only
- Prefer thin E3 — Phase 24 math is Done

## After QA Pass
PM → mark P24-E3-T1 Done; close Epic **P24-E3** and **Phase 24** if no further P24 tasks; next = **PM-GATE-POST-P24** (IDE · weather · App Store · Companion polish · TypingRhythm · other) **without** opening a PR during freeze.
