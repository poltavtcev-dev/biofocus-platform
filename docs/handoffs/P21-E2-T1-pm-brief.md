# PM Brief → Dev: P21-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P21-E1-T1** (ADR-022 locked; QA Pass)  
**Evidence:** `docs/handoffs/P21-E1-T1-qa-to-pm.md` · `docs/handoffs/P21-E1-T1-dev-to-qa.md`  
**Contract:** ADR-022 detail in `docs/decision-log.md`

## Task
**P21-E2-T1 — Ship catalog Feature `DeepWorkScore` per ADR-022**

## Why
ADR-022 locked Feature-level sustained-focus composite from **FocusScore (required) + ContextSwitchRate (optional)**; idle dropped for v1. E1 docs/catalog stub are in place; E2 implements math + DAG registration so Snapshot/IPC can emit the Feature. No new Observation family; no migration; leaf formulas stay untouched.

## Acceptance Criteria
1. Implement **`DeepWorkScore`** in `feature-engine` per ADR-022: Feature-level inputs only; window **15m / 1m**; output **0–100**.
2. Composition + weights (ADR-022):  
   - `focus = FocusScore` (already 0–100)  
   - `stability = clamp(100 - ContextSwitchRate × 50, 0, 100)`  
   - Catalog weights **w_focus = 0.60**, **w_stability = 0.40**  
   - **Omit** when FocusScore absent; **renormalize** (Focus-only) when CSR absent; both present → catalog weights.  
   - Do **not** emit CSR-only; do **not** invent Focus from Observations.
3. **ADR-007** confidence: `expected_slots = 2`; coverage = present/2; `confidence = clamp(coverage × mean(upstream.confidence), 0, 1)`.
4. Optional explanation factors for present components (focus / stability) with calm labels — **not** flow / burnout / ADHD copy.
5. Register in catalog / DAG after focus nodes (follow existing composite patterns). Finalize `docs/06-feature-catalog.md` §1.17 (stub → shipped formula).
6. Unit tests: Focus+CSR emit · Focus-only renormalize · omit without Focus · confidence · factors when emitted.
7. **Do not** rewrite FocusScore / CSR formulas; **no** new Observation `data_type`; **no** migration; **no** UI/Dashboard surface (→ E3 optional); **no** parallel FocusScore.
8. Handoff: `docs/handoffs/P21-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard calm surface (→ **P21-E3**)
- IDE plugin; weather ambient; App Store packaging
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)
- New collectors / Observation families / idle leaf

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/21-deep-work-score`
- Personal self-tracking only — not workplace monitoring
- Copy: “sustained focus in this window” — **not** “you are in flow” / burnout / ADHD
- LLM remains L5 interpret-only
- **No migration**
- Important: omit-without-Focus is locked — CSR alone must not emit DeepWorkScore

## After QA Pass
PM → mark P21-E2-T1 Done; Ready **P21-E3-T1** (dogfood / optional Dashboard).
