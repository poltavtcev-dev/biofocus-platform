# PM Brief → Dev: P7-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-06  
**Closed previous:** P6-E3-T2 (QA Pass — MeetingDensity + RecoveryBetweenMeetings); Phase 6 complete  
**Evidence:** `docs/handoffs/P6-E3-T2-qa-to-pm.md`

## Task
**P7-E1-T1 — Feature confidence contract + ADR + wire**

## Why
Observation.confidence exists; Feature values still look equally trustworthy whether HRV/context were present or not. Trust layer needs Feature-level confidence so later Insights / UI can down-weight thin windows — without inventing a parallel Feature registry.

## Acceptance Criteria
1. Record **ADR-007**: Feature-level confidence (relationship to Observation.confidence; v1 formula — coverage / mean obs confidence / missing-input policy). Rejected alternatives noted (e.g. UI-only heuristics, second registry).
2. Domain + contracts: `Feature` (and `get_feature_snapshot` JSON) expose confidence in `[0.0, 1.0]`; Ubiquitous Language intact; **no new SQLite schema** unless ADR + user approve.
3. Catalog path (`register_catalog_v1` or Focus/Stress at minimum) **computes** confidence; thin/empty input windows behave per ADR (low confidence or omit — explicit + tested).
4. Unit tests: rich inputs → higher confidence; missing HRV/context → lower; idle-safe (no busy-loop).
5. Docs: `docs/02-domain-model.md`, catalog rule in `docs/06-feature-catalog.md`, contracts/API as needed.
6. Handoff: `docs/handoffs/P7-E1-T1-dev-to-qa.md`.

## Out of scope
- Explanation factor breakdown (→ **P7-E2-T1**)
- New bio Features / RecoveryScore / DeepWorkScore (→ **P7-E3**)
- Pattern Discovery, CognitiveLoad, Dashboard redesign
- Action / automation framework

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `crates/bio-spec`, `crates/feature-engine`, decision-log + domain/contracts docs
- Do **not** parallel-rewrite Feature registry (triage: evolve metadata on existing `FeatureNode`)
- Branch: `phase/7-trust-layer` (or continue from Phase 6 tip if cluster not yet on `main`)
- Calm copy only — confidence is data quality, not a clinical claim

## After QA Pass
PM → mark P7-E1-T1 Done; Ready **P7-E2-T1** (Explanation factors) unless sequencing note says otherwise.
