# PM Brief → Dev: P20-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P20-E1-T1** (ADR-021 locked; QA Pass)  
**Evidence:** `docs/handoffs/P20-E1-T1-qa-to-pm.md` · `docs/handoffs/P20-E1-T1-dev-to-qa.md`  
**Contract:** ADR-021 detail in `docs/decision-log.md`

## Task
**P20-E2-T1 — Ship catalog Feature `CognitiveLoad` per ADR-021**

## Why
ADR-021 locked Feature-level composite demand from **MeetingDensity + ContextSwitchRate + NotificationPressure**. E1 docs/catalog stub are in place; E2 implements math + DAG registration so Snapshot/IPC can emit the Feature. No new Observation family; no migration; leaf formulas stay untouched.

## Acceptance Criteria
1. Implement **`CognitiveLoad`** in `feature-engine` per ADR-021: Feature-level inputs only; window **15m / 1m**; output **0–100**.
2. Normalization + weights (ADR-021):  
   - `meeting = MeetingDensity × 100`  
   - `switches = clamp(ContextSwitchRate × 50, 0, 100)`  
   - `notify = NotificationPressure` (already 0–100)  
   - Equal thirds; **renormalize** over present inputs; **omit** only when **none** of the three are present.
3. **ADR-007** confidence: `expected_slots = 3`; `coverage = present/3`; `confidence = clamp(coverage × mean(upstream.confidence), 0, 1)`.
4. Optional explanation factors for present components (meeting / switches / notifications) with calm labels — **not** clinical overload / burnout copy.
5. Register in catalog / DAG after calendar + focus + notification nodes (follow existing composite patterns, e.g. DistractionScore / FatigueIndex). Finalize `docs/06-feature-catalog.md` §1.16 (stub → shipped formula).
6. Unit tests: rich emit (all three) · partial renormalize (≥1) · omit when empty · confidence · factors present when emitted.
7. **Do not** rewrite MeetingDensity / CSR / NotificationPressure formulas; **no** new Observation `data_type`; **no** migration; **no** UI/Dashboard surface (→ E3 optional).
8. Handoff: `docs/handoffs/P20-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard calm surface (→ **P20-E3**)
- IDE plugin; weather ambient; App Store packaging
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)
- New collectors / Observation families

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/20-cognitive-load`
- Personal self-tracking only — not workplace monitoring
- Copy: “combined demand in this window” — **not** “you are overloaded” / ADHD / burnout
- LLM remains L5 interpret-only
- **No migration**
- Important: NotificationPressure often empty when opt-in is off — partial CognitiveLoad via renormalize must still emit

## After QA Pass
PM → mark P20-E2-T1 Done; Ready **P20-E3-T1** (dogfood / optional Dashboard).
