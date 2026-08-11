# PM Brief → Dev: P24-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **PM-GATE-POST-P23** (chose **`CircadianOffset`**); Phase 23 Personal Context Layer complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P23-pm-brief.md` · `docs/handoffs/P23-E3-T1-qa-to-pm.md`  
**Phase:** Phase 24 CircadianOffset — Epic P24-E1  
**Branch:** `phase/24-circadian-offset`

## Task
**P24-E1-T1 — ADR-025: lock `CircadianOffset` Feature scope (inputs, formula stance, omit policy)**

## Why
Catalog backlog named **`CircadianOffset`** as alignment of work vs chronotype proxy from **sleep + activity timing**, deferred until after Personal Context. Phase 23 shipped presence/context; P17 shipped `SleepDebt` / `EnergyScore` / `ActivityBalance` (and sleep/activity Observations). PM gate post–P23 chose this Feature over IDE / weather / App Store / Companion polish / TypingRhythm: required timing inputs exist — lock scope in an ADR before shipping math. Framing must stay calm and non-clinical.

## Acceptance Criteria
1. Record **ADR-025** in `docs/decision-log.md`: Phase 24 v1 primary = catalog Feature **`CircadianOffset`**. Lock preferred input set (Feature-level and/or Observation timing from already-shipped sleep/activity families — pick explicitly). Rationale vs Personal Context sibling + prior deferrals; personal self-tracking only; calm non-clinical framing.
2. Formula stance sketch for E2: window/step (state preference, e.g. longer window if sleep timing needs it); 0–100 or signed offset units — **pick one and justify**; how missing inputs are handled (omit vs renormalize — pick one); ADR-007 confidence sketch; optional explanation factors. Prefer composing **work/activity timing vs sleep timing** into a calm alignment score — **do not** invent clinical chronotype labels; **do not** rewrite SleepDebt / EnergyScore / FocusScore formulas.
3. Rejected alternatives documented (clinical chronotype / circadian-disorder / “you are a night owl so you fail” claims; workplace schedule surveillance; inventing new Observation families this phase unless ADR finds a hard gap; rewriting sleep/activity leaf Features; IDE / weather / App Store / TypingRhythm as Phase 24 primary; precise GPS; PR during freeze; applying migration without approve).
4. Schema: **none to apply** — prefer Features / Observations already shipped; **no** migration; **no** new Observation `data_type` in v1 unless ADR finds a hard gap (prefer zero).
5. Short sketch: E2 ships `CircadianOffset` in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard calm surface.
6. Docs touch: `06-feature-catalog` (planned → ADR sketch) / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P24-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature math / DAG registration (→ **P24-E2**)
- New collectors / Observation families (unless ADR proves a hard gap — prefer zero)
- IDE plugin; weather ambient; App Store packaging product; TypingRhythm
- Workplace / manager dashboards; clinical sleep/chronotype diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Features only with real inputs already shipped
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/24-circadian-offset`
- Personal self-tracking only — not workplace monitoring
- Copy: “schedule alignment in this window” — **not** “wrong chronotype” / sleep-disorder / burnout diagnosis
- Distinct from SleepDebt (debt magnitude ≠ timing alignment) and DeskAwayPresence (away-from-desk ≠ circadian offset)
- LLM remains L5 interpret-only
- **No migration applied in E1**

## After QA Pass
PM → mark P24-E1-T1 Done; Ready **P24-E2-T1** shaped by ADR-025 (no schema approve expected).
