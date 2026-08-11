# PM Brief → Dev: P20-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P20-E1** ✅ · **ADR-021** locked  
**Evidence:** `docs/handoffs/P20-E1-T1-qa-to-pm.md` · `docs/handoffs/P20-E1-T1-dev-to-qa.md`  
**Next:** **P20-E2-T1** — `docs/handoffs/P20-E2-T1-pm-brief.md`

## Task
**P20-E1-T1 — ADR-021: lock `CognitiveLoad` Feature scope (inputs, formula stance, omit policy)**

## Why
Catalog backlog named **`CognitiveLoad`** as a combined demand proxy from **MeetingDensity**, **CSR**, and notifications. Phase 18–19 shipped `notification_event` + `NotificationPressure` (including live NC mapping). PM gate post–P19 chose this composite Feature over IDE / weather / App Store / polish-as-primary: all required input families exist — lock scope in an ADR before shipping math.

## Acceptance Criteria
1. Record **ADR-021** in `docs/decision-log.md`: Phase 20 v1 primary = catalog Feature **`CognitiveLoad`**. Lock input set (prefer Feature-level: `MeetingDensity` + `ContextSwitchRate` + `NotificationPressure` — or justify Observation-level mix). Rationale vs prior deferrals; personal self-tracking only; calm non-clinical framing.
2. Formula stance sketch for E2: window/step (prefer **15m / 1m** align Focus/CSR); 0–100 output; how missing inputs are handled (omit vs renormalize — pick one); ADR-007 confidence sketch; optional explanation factors.
3. Rejected alternatives documented (clinical “cognitive overload” / burnout claims; workplace surveillance scoring; inventing new Observation families this phase; rewriting MeetingDensity / CSR / NotificationPressure formulas; IDE / weather / App Store as Phase 20 primary; PR during freeze; applying migration without approve).
4. Schema: **none to apply** — Features from existing inputs; **no** migration; **no** new Observation `data_type` in v1 unless ADR finds a hard gap (prefer zero).
5. Short sketch: E2 ships `CognitiveLoad` in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard calm surface.
6. Docs touch: `06-feature-catalog` (planned → ADR sketch) / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P20-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature math / DAG registration (→ **P20-E2**)
- New collectors / Observation families
- IDE plugin; weather ambient; App Store packaging product
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Features only with real inputs already shipped
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/20-cognitive-load`
- Personal self-tracking only — not workplace monitoring
- Copy: “combined demand in this window” — **not** “you are overloaded” / ADHD / burnout diagnosis
- LLM remains L5 interpret-only
- **No migration applied in E1**

## After QA Pass
PM → mark P20-E1-T1 Done; Ready **P20-E2-T1** shaped by ADR-021 (no schema approve expected).
