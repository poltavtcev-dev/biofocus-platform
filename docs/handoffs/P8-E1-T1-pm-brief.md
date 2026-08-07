# PM Brief → Dev: P8-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-06  
**Closed previous:** P7-E3-T1 (QA Pass — `RecoveryScore`); Phase 7 complete  
**Evidence:** `docs/handoffs/P7-E3-T1-qa-to-pm.md`

## Task
**P8-E1-T1 — ADR-008: Pattern Discovery history / recompute**

## Why
North star is Personal Pattern Discovery. Phase 7 gave trusted Features; Phase 8 needs a **decision** on multi-day baselines — evaluate-on-read recompute vs persisted Feature/baseline history — **before** any new SQLite tables or background workers.

## Acceptance Criteria
1. Record **ADR-008** in `docs/decision-log.md`: chosen approach for Pattern Discovery v1 (recompute-on-read / Feature history store / hybrid); relationship to existing Observations + Features + Knowledge Insights; idle/privacy constraints (local-only, no busy-loop).
2. Rejected alternatives documented (e.g. cloud sync of patterns, ML model training, parallel “Correlation Engine” crate).
3. If schema is proposed: tables/columns sketched in ADR + storage/domain notes — **do not apply migration** until user approve.
4. Short sketch: how Knowledge would consume the approach for ≥1 calm baseline Insight (contracts note OK).
5. Handoff: `docs/handoffs/P8-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing history tables / recompute worker (→ **P8-E2** after ADR + approve if needed)
- Recommendations engine (Phase 9)
- CircadianOffset / SleepDebt Features
- Dashboard redesign
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer evolve `knowledge-engine` + existing Feature snapshot path — no new parallel registry/crate without ADR
- Branch: `phase/8-pattern-discovery` (or continue `phase/7-trust-layer` tip)
- Calm copy only — patterns are personal observations, not clinical diagnosis

## After QA Pass
PM → mark P8-E1-T1 Done; if ADR requires schema approve, wait for user; else Ready **P8-E2-T1**.
