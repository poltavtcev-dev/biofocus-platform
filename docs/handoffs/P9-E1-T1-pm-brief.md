# PM Brief → Dev: P9-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-08  
**Closed previous:** P8-E3-T1 (QA Pass — pattern Insights Dashboard); Phase 8 complete  
**Evidence:** `docs/handoffs/P8-E3-T1-qa-to-pm.md`

## Task
**P9-E1-T1 — ADR-009: Recommendations domain / engine shape**

## Why
Vision L4 is *Deterministic Recommendations with Evidence*. Today L4 is only optional thin text on `Insight.actionRecommendation`. Phase 8 closed the Knowledge/pattern path; Phase 9 needs an **ADR** before new types, crates, IPC, or schema — so Recommendations stay explainable, local-first, and non-clinical.

## Acceptance Criteria
1. Record **ADR-009** in `docs/decision-log.md`: chosen approach for Recommendations v1 (first-class `Recommendation` vs evolve `Insight.actionRecommendation` vs `RecommendationRule` host in `knowledge-engine` / thin module); relationship to Features, Insights, and Evidence; evaluate-on-read vs persist; idle/privacy (local-only, no busy-loop).
2. Rejected alternatives documented (e.g. LLM as source of truth for actions; clinical/prescription framing; parallel Coach Engine without Evidence; cloud sync of recommendations).
3. If schema is proposed: tables/columns sketched in ADR + storage/domain notes — **do not apply migration** until user approve.
4. Short sketch: how Core would emit ≥1 calm deterministic recommendation for E2 (inputs from existing Feature/Insight Evidence; non-clinical copy).
5. Glossary / domain note: `Recommendation` term updated in `docs/16-glossary.md` (and domain/API touch as needed) — or ADR explicitly defers rename and keeps thin text only (must be stated).
6. Handoff: `docs/handoffs/P9-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing recommendation rules / evaluator (→ **P9-E2**)
- IPC / Dashboard Recommendations surface (→ **P9-E3**)
- Plugin wave-1 (Phase 10); AI coaching polish (Phase 11)
- CircadianOffset / SleepDebt Features
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer evolve `knowledge-engine` + existing Insight/Evidence path — no new parallel registry/crate without ADR justification
- Branch: `phase/9-recommendations` (or continue `phase/8-pattern-discovery` tip)
- Calm copy only — suggestions are optional personal hints, not medical advice
- LLM remains L5 interpret-only — must not compute Recommendations

## After QA Pass
PM → mark P9-E1-T1 Done; if ADR requires schema approve, wait for user; else Ready **P9-E2-T1**.
