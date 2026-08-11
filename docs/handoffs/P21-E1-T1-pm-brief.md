# PM Brief → Dev: P21-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P21-E1** ✅ · **ADR-022** locked  
**Evidence:** `docs/handoffs/P21-E1-T1-qa-to-pm.md` · `docs/handoffs/P21-E1-T1-dev-to-qa.md`  
**Next:** **P21-E2-T1** — `docs/handoffs/P21-E2-T1-pm-brief.md`

## Task
**P21-E1-T1 — ADR-022: lock `DeepWorkScore` Feature scope (inputs, formula stance, omit policy)**

## Why
Catalog backlog named **`DeepWorkScore`** as sustained-focus windows from **FocusScore** + **ContextSwitchRate** (P7-era candidate). Phase 20 shipped demand-side `CognitiveLoad`; FocusScore + CSR have long been stable leaves. PM gate post–P20 chose this composite over IDE / weather / App Store / Companion polish / CircadianOffset / AttentionStability-as-primary: required inputs exist — lock scope in an ADR before shipping math.

## Acceptance Criteria
1. Record **ADR-022** in `docs/decision-log.md`: Phase 21 v1 primary = catalog Feature **`DeepWorkScore`**. Lock input set (prefer Feature-level: `FocusScore` + `ContextSwitchRate` — catalog also mentions “idle”; either map idle to an existing signal **or** explicitly drop it for v1 with rationale). Rationale vs prior deferrals; personal self-tracking only; calm non-clinical framing.
2. Formula stance sketch for E2: window/step (prefer **15m / 1m** align Focus/CSR); 0–100 output; how missing inputs are handled (omit vs renormalize — pick one); ADR-007 confidence sketch; optional explanation factors. Prefer composing Focus high + CSR low into sustained-focus intensity — **do not** invent a parallel FocusScore.
3. Rejected alternatives documented (clinical “flow state” / burnout / ADHD claims; workplace surveillance scoring; inventing new Observation families this phase; rewriting FocusScore / CSR formulas; IDE / weather / App Store as Phase 21 primary; AttentionStability or CircadianOffset as this-phase primary; PR during freeze; applying migration without approve).
4. Schema: **none to apply** — Features from existing inputs; **no** migration; **no** new Observation `data_type` in v1 unless ADR finds a hard gap (prefer zero).
5. Short sketch: E2 ships `DeepWorkScore` in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard calm surface.
6. Docs touch: `06-feature-catalog` (planned → ADR sketch) / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P21-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature math / DAG registration (→ **P21-E2**)
- New collectors / Observation families
- IDE plugin; weather ambient; App Store packaging product
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Features only with real inputs already shipped
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/21-deep-work-score`
- Personal self-tracking only — not workplace monitoring
- Copy: “sustained focus in this window” — **not** “you are in flow” / burnout / ADHD diagnosis
- LLM remains L5 interpret-only
- **No migration applied in E1**

## After QA Pass
PM → mark P21-E1-T1 Done; Ready **P21-E2-T1** shaped by ADR-022 (no schema approve expected).
