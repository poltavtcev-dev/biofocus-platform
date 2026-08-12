# PM Brief → Dev: P25-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · next **P25-E3-T1**  
**Date:** 2026-08-12  
**Closed previous:** P25-E1-T1 (**ADR-026** locked; QA Pass)  
**Evidence:** `docs/handoffs/P25-E1-T1-qa-to-pm.md` · `docs/handoffs/P25-E1-T1-dev-to-qa.md`  
**Phase:** Phase 25 SustainedLoadIndicator — Epic P25-E2  
**Branch:** `phase/25-sustained-load`

## Task
**P25-E2-T1 — Ship catalog Feature `SustainedLoadIndicator` per ADR-026**

## Why
ADR-026 locked Feature-level **persistence** composite from **StressIndex + FatigueIndex + MeetingDensity** (schedule proxy) — distinct from CognitiveLoad current demand. E1 docs/catalog stub §1.21 are in place; E2 implements math + DAG registration so Snapshot/IPC can emit the Feature. No new Observation family; no migration; leaf / CognitiveLoad formulas stay untouched.

## Acceptance Criteria
1. Implement **`SustainedLoadIndicator`** in `feature-engine` per ADR-026: Feature-level inputs only; cadence **15m / 1m**; persistence math **4h lookback** ending at window end; output **0–100**.
2. Composition (ADR-026):  
   - `stress_term` = mean(StressIndex samples with end in lookback)  
   - `fatigue_term` = mean(FatigueIndex samples with end in lookback)  
   - `meeting_term` = MeetingDensity mean mapped to 0–100 (exact scale normalization OK if catalog unit differs)  
   - Catalog weights sketch **0.40 / 0.40 / 0.20** (stress / fatigue / meeting); **renormalize** over present terms  
   - **Omit** when both StressIndex and FatigueIndex absent — **do not** emit from MeetingDensity alone  
   - **Do not** use CognitiveLoad / FocusScore / CircadianOffset as inputs; **do not** invent Stress from CognitiveLoad or Fatigue from SleepDebt
3. **ADR-007** confidence: `expected_slots = 3`; coverage = present/3; `confidence = clamp(coverage × mean(upstream.confidence), 0, 1)`.
4. Optional explanation factors for present components (`stress` / `fatigue` / `meeting`) with calm labels — **not** burnout / “you are burned out” copy.
5. Register in catalog / DAG after stress / calendar nodes (follow existing composite patterns; exact `register_*` helper name chosen in E2). Finalize `docs/06-feature-catalog.md` §1.21 (stub → shipped formula).
6. Unit tests: Stress+Fatigue+Meeting emit · Stress+Fatigue renormalize (no Meeting) · omit when both Stress&Fatigue absent · meetings-alone omit · confidence · factors when emitted · no CognitiveLoad-as-input path.
7. Docs: `12-development` / `16-glossary` ship notes for SustainedLoadIndicator.
8. **Do not** rewrite StressIndex / FatigueIndex / MeetingDensity / CognitiveLoad formulas; **no** new Observation `data_type`; **no** migration; **no** UI/Dashboard surface (→ E3 optional); **no** parallel engine crate.
9. Handoff: `docs/handoffs/P25-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard calm surface (→ **P25-E3**; suggested chart label **Prolonged load**)
- IDE plugin; weather ambient; App Store packaging; TypingRhythm
- Workplace / manager dashboards; clinical burnout diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)
- New collectors / Observation families

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. sustained-load DoD)
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/25-sustained-load`
- Personal self-tracking only — not workplace monitoring
- Copy: “prolonged load in this window” — **not** “you are burned out” / clinical diagnosis
- Distinct from CognitiveLoad (“combined demand in this window”)
- LLM remains L5 interpret-only
- **No migration**
- Important: omit-when-both-Stress-and-Fatigue-absent is locked — MeetingDensity alone must not emit

## After QA Pass
PM → mark P25-E2-T1 Done; Ready **P25-E3-T1** (dogfood / optional Dashboard).
