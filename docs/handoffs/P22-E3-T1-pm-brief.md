# PM Brief → Dev|UX: P22-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P22-E3** ✅ · **Phase 22** ✅  
**Evidence:** `docs/handoffs/P22-E3-T1-qa-to-pm.md` · `docs/handoffs/P22-E3-T1-dev-to-qa.md`  
**Next:** **PM-GATE-POST-P22** — `docs/handoffs/PM-GATE-POST-P22-pm-brief.md`  
**Closed previous:** P22-E2-T1 (`AttentionStability` Feature shipped; QA Pass)  
**Phase:** Phase 22 AttentionStability — Epic P22-E3

## Task
**P22-E3-T1 — Dogfood notes + optional calm Dashboard surface for `AttentionStability`**

## Why
E2 shipped `AttentionStability` via `register_attention_stability_v1` / `register_catalog_v1` — Snapshot/IPC already pick it up. Phase 22 closes with operator dogfood notes and an optional calm presentation (existing Dashboard Feature path / series) — **not** new math, **not** clinical framing. Stay distinct from DeepWorkScore (“sustained focus”).

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §): how to verify `AttentionStability` appears — fixture / ingest of FocusScore inputs (+ optional CSR) → Feature snapshot / series; note Focus-only emit when CSR absent; omit when Focus absent; single-vs-multi Focus sample behavior (range vs 100).
2. Calm framing only: “focus stability in this window” — **not** ADHD / “you can’t focus” / burnout diagnosis in docs or UI chrome. Distinct from DeepWorkScore “sustained focus in this window”.
3. **Optional** calm Dashboard surface: if existing Feature list / chart path already shows catalog Features from snapshot/series, ensure `AttentionStability` is visible with calm label when present; empty/omit stays quiet (no error noise). Prefer reuse over redesign. Docs-only dogfood **is enough** if Feature already surfaces via existing IPC without UI gaps — state which path in handoff.
4. Smoke notes in handoff: commands / steps to see emit (unit/fixture and/or app path); confirm UI ↛ SQLite.
5. **Do not** rewrite `AttentionStability` / leaf / DeepWorkScore formulas; **no** migration; **no** new Observation family; **no** new Insights/Recommendations rule required.
6. Handoff: `docs/handoffs/P22-E3-T1-dev-to-qa.md`.

## Out of scope
- Formula / DAG / ADR-023 math changes
- IDE plugin; weather ambient; App Store packaging product
- Workplace / manager dashboards; clinical diagnosis copy
- New “AttentionStability” product window / Dashboard redesign
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: docs (+ optional desktop Feature presentation / `docs/09-api.md` if IPC gap)
- Branch: `phase/22-attention-stability`
- Personal self-tracking only
- LLM remains L5 interpret-only
- Prefer thin E3 — Phase 22 math is Done

## After QA Pass
PM → mark P22-E3-T1 Done; close Epic **P22-E3** and **Phase 22** if no further P22 tasks; next = **PM-GATE-POST-P22** (IDE · weather · App Store · polish · CircadianOffset · other) **without** opening a PR during freeze.
