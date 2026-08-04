# PM Brief → Dev|QA: P2-E2-T3

**From:** PM  
**To:** QA (lead) + Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E2-T2 — QA Pass with notes (`docs/handoffs/P2-E2-T2-qa-to-pm.md`)

## Task
**P2-E2-T3 — Collector integration tests + pause idle**

## Why
T1/T2 collectors exist with unit/mocks. Close the E2 epic with integration coverage and explicit pause/idle confirmation before iOS companion work.

## Acceptance Criteria
1. Integration tests cover emit Observation path for active window and (when enabled) keystroke aggregates into channel/storage where practical.
2. Pause/stop confirmed: paused/stopped collector does **not** keep periodic work (no busy-loop; join clean).
3. Document how to run the suite (`docs/12-development.md` / `11-testing` if present).
4. Handoff: `docs/handoffs/P2-E2-T3-dev-to-qa.md` (or QA-authored if QA leads implementation of tests).

## Out of scope
- iOS / HealthKit → **P2-E3-***
- Feature pipeline, dashboard
- Changing T1/T2 payload contracts

## Constraints
- Idle DoD; no unwrap/expect in prod
- Prefer extending existing `macos-collector` tests + optional desktop host test hooks

## Hygiene
- Commit after build; push only at sprint gate (`SPRINT-GATE.md`).

## After QA Pass
PM → Ready **P2-E3-T1** (or **P2-E0-T1** if hygiene first); consider sprint PR if E2 epic gate desired.
