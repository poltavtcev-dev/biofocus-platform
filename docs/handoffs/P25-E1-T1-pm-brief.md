# PM Brief → Dev: P25-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · **ADR-026** locked · next **P25-E2-T1**  
**Date:** 2026-08-12  
**Closed previous:** **PM-GATE-POST-P24** (chose **`SustainedLoadIndicator`**); Phase 24 CircadianOffset complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P24-pm-brief.md` · `docs/handoffs/P24-E3-T1-qa-to-pm.md`  
**Phase:** Phase 25 SustainedLoadIndicator — Epic P25-E1  
**Branch:** `phase/25-sustained-load`

## Task
**P25-E1-T1 — ADR-026: lock `SustainedLoadIndicator` Feature scope (inputs, formula stance, omit policy)**

## Why
Catalog backlog named **`SustainedLoadIndicator`** as calm rename of “burnout risk” from **Stress, Fatigue, schedule**. Phase 20 shipped **`CognitiveLoad`** (current combined demand). StressIndex, FatigueIndex, and MeetingDensity have long shipped. PM gate post–P24 chose this Feature over IDE / weather / App Store / Companion polish / TypingRhythm / DeepFocusLikelihood: required Feature inputs exist — lock scope in an ADR before shipping math. Must stay **distinct** from CognitiveLoad (prolonged / persistence ≠ instantaneous demand). Framing must stay calm and non-clinical.

## Acceptance Criteria
1. Record **ADR-026** in `docs/decision-log.md`: Phase 25 v1 primary = catalog Feature **`SustainedLoadIndicator`**. Lock preferred input set (Feature-level: prefer **StressIndex** + **FatigueIndex** + **MeetingDensity** or justified schedule proxy — pick explicitly). Rationale vs CognitiveLoad sibling + prior deferrals; personal self-tracking only; calm non-clinical framing.
2. Formula stance sketch for E2: window/step (state preference — may use longer lookback than 15m if “prolonged” requires it; justify); 0–100 output; how missing inputs are handled (omit vs renormalize — pick one); ADR-007 confidence sketch; optional explanation factors. Prefer composing **elevated load persistence** into a calm score — **do not** invent clinical burnout labels; **do not** rewrite StressIndex / FatigueIndex / MeetingDensity / CognitiveLoad formulas.
3. Rejected alternatives documented (clinical burnout / “you are burned out” claims; workplace surveillance scoring; inventing new Observation families this phase; rewriting leaf / CognitiveLoad formulas; IDE / weather / App Store / TypingRhythm / DeepFocusLikelihood as Phase 25 primary; PR during freeze; applying migration without approve).
4. Schema: **none to apply** — Features from existing inputs; **no** migration; **no** new Observation `data_type` in v1 unless ADR finds a hard gap (prefer zero).
5. Short sketch: E2 ships `SustainedLoadIndicator` in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard calm surface.
6. Docs touch: `06-feature-catalog` (planned → ADR sketch) / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P25-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature math / DAG registration (→ **P25-E2**)
- New collectors / Observation families
- IDE plugin; weather ambient; App Store packaging product; TypingRhythm
- Workplace / manager dashboards; clinical burnout diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Features only with real inputs already shipped
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/25-sustained-load`
- Personal self-tracking only — not workplace monitoring
- Copy: “prolonged load in this window” — **not** “you are burned out” / clinical diagnosis
- Distinct from CognitiveLoad (“combined demand in this window”)
- LLM remains L5 interpret-only
- **No migration applied in E1**

## After QA Pass
PM → mark P25-E1-T1 Done; Ready **P25-E2-T1** shaped by ADR-026 (no schema approve expected).
