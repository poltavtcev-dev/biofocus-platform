# PM Brief → Dev/UX: P15-E3-T1

**From:** PM  
**To:** Dev/UX  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** P15-E2-T1 (QA Pass — Core SDNN + iOS Auto-sync); Epic **P15-E2** ✅  
**Evidence:** `docs/handoffs/P15-E2-T1-qa-to-pm.md`  
**Branch:** `phase/15-companion-hrv-autonomy`

## Task
**P15-E3-T1 — Dogfood companion + Auto-sync UI/status (ADR-016)**

## Why
E2 shipped Core SDNN-or-RMSSD + iOS queue/observers. Phase 15 still needs an operator **dogfood path** and calm Companion UI/status (Auto-sync, pending, last flush, empty HRV) so autonomy is usable without clinical claims — then close the Phase.

## Acceptance Criteria
1. Dogfood runbook in `docs/12-development.md` (and/or iOS README): LAN Desktop + Auto-sync on + verify Observations / HRV path in SQLite where practical; no busy-loop guidance.
2. Companion UI exposes Auto-sync / pending / last flush / send latest / flush queue (calm copy).
3. Calm empty-HRV messaging when Mi Fitness / HealthKit HRV sparse — no clinical claims.
4. ADR-015 ambient light remains **parked** in docs notes (not active Sprint work).
5. Handoff: `docs/handoffs/P15-E3-T1-dev-to-qa.md`.

## Out of scope
- Ambient light collector / `AmbientLightShare`
- Sleep / steps / SpO2 / ECG
- Cloud relay; workplace framing
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-016 in `docs/decision-log.md`
- Personal self-tracking only
- LLM remains L5 interpret-only

## Build status (PM note)
Dev + QA already Pass on disk (`docs/handoffs/P15-E3-T1-dev-to-qa.md`, `docs/handoffs/P15-E3-T1-qa-to-pm.md`). Prefer **pm-close** next — do not re-build unless defects.

## After QA Pass
PM → mark P15-E3-T1 Done; **Phase 15** ✅; Ready **PM-GATE-POST-P15**.
