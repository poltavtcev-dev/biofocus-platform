# PM Brief → Dev: P22-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P22-E1** ✅ · **ADR-023** locked  
**Evidence:** `docs/handoffs/P22-E1-T1-qa-to-pm.md` · `docs/handoffs/P22-E1-T1-dev-to-qa.md`  
**Next:** **P22-E2-T1** — `docs/handoffs/P22-E2-T1-pm-brief.md`  
**Closed previous:** PM-GATE-POST-P21 (chose **`AttentionStability`**); Phase 21 complete  
**Phase opened:** Phase 22 AttentionStability (Sprint 43–44) — `docs/SPRINT_ROADMAP.md`

## Task
**P22-E1-T1 — ADR-023: lock `AttentionStability` Feature scope (inputs, formula stance, omit policy)**

## Why
Catalog backlog named **`AttentionStability`** as variance of focus / switches from **FocusScore** + **ContextSwitchRate**. Phase 21 shipped sibling **`DeepWorkScore`** (sustained-focus intensity on the same leaves). PM gate post–P21 chose this composite over IDE / weather / App Store / Companion polish / CircadianOffset: required inputs exist — lock scope in an ADR before shipping math. Must stay **distinct** from DeepWorkScore (variance/stability ≠ intensity).

## Acceptance Criteria
1. Record **ADR-023** in `docs/decision-log.md`: Phase 22 v1 primary = catalog Feature **`AttentionStability`**. Lock input set (prefer Feature-level: `FocusScore` + `ContextSwitchRate`). Rationale vs DeepWorkScore sibling + prior deferrals; personal self-tracking only; calm non-clinical framing.
2. Formula stance sketch for E2: window/step (prefer **15m / 1m** align Focus/CSR); 0–100 output; how missing inputs are handled (omit vs renormalize — pick one); ADR-007 confidence sketch; optional explanation factors. Prefer composing **low variance / high stability** of Focus and/or CSR into a calm stability score — **do not** invent a parallel FocusScore; **do not** redefine DeepWorkScore math.
3. Rejected alternatives documented (clinical ADHD / attention-deficit / “you can’t focus” claims; workplace surveillance scoring; inventing new Observation families this phase; rewriting FocusScore / CSR / DeepWorkScore formulas; IDE / weather / App Store as Phase 22 primary; CircadianOffset as this-phase primary; PR during freeze; applying migration without approve).
4. Schema: **none to apply** — Features from existing inputs; **no** migration; **no** new Observation `data_type` in v1 unless ADR finds a hard gap (prefer zero).
5. Short sketch: E2 ships `AttentionStability` in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard calm surface.
6. Docs touch: `06-feature-catalog` (planned → ADR sketch) / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P22-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature math / DAG registration (→ **P22-E2**)
- New collectors / Observation families
- IDE plugin; weather ambient; App Store packaging product
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Features only with real inputs already shipped
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/22-attention-stability`
- Personal self-tracking only — not workplace monitoring
- Copy: “focus stability in this window” — **not** “you have ADHD” / “you can’t focus” / burnout diagnosis
- Distinct from DeepWorkScore (“sustained focus in this window”)
- LLM remains L5 interpret-only
- **No migration applied in E1**

## After QA Pass
PM → mark P22-E1-T1 Done; Ready **P22-E2-T1** shaped by ADR-023 (no schema approve expected).
