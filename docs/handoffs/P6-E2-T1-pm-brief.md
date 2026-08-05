# PM Brief → UX + Dev: P6-E2-T1

**From:** PM  
**To:** UX + Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P6-E1-T1 (QA Pass — Life Events Observation kinds + ADR-006; Epic **P6-E1** ✅)  
**Evidence:** `docs/handoffs/P6-E1-T1-qa-to-pm.md`

## Task
**P6-E2-T1 — Desktop quick-log Life Events**

## Why
ADR-006 and ingest validation ship: Life Events are ordinary Observations (`data_type: "life_event"`, `payload.kind` ∈ coffee / walk / lunch / workout). Desktop still has no calm path to create them without raw HTTP — dogfood needs a quick-log UI over IPC.

## Acceptance Criteria
1. User can log a **v1** Life Event (`coffee` / `walk` / `lunch` / `workout`) from Desktop with calm, non-evaluative copy.
2. Event becomes an `Observation` via **IPC** (UI ↛ SQLite); payload matches ADR-006 / `docs/07-contracts.md`.
3. Logged event is visible via an existing path (status / Dashboard / storage read as appropriate) — document which in handoff.
4. Idle-safe: no busy-loop poll; no continuous timers for logging itself.
5. Manual smoke steps in handoff (at least one kind end-to-end).
6. Handoff: `docs/handoffs/P6-E2-T1-dev-to-qa.md`.

## Out of scope
- Calendar import / sync (→ **P6-E3-T1**)
- Wearable auto-detect of workouts
- New Insight rules / `MeetingDensity` / `ActivityBalance`
- New SQLite schema
- Changing Life Event contract / ADR-006 kinds list

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `apps/desktop` (+ IPC / host glue); Core path that creates Observation (reuse ingest/storage Observation path — do not invent a parallel store)
- Contract: ADR-006 · `docs/07-contracts.md` · `docs/09-api.md` (`invalid_life_event` for HTTP; IPC errors should be explicit and calm)
- Branch: `phase/6-life-context`

## After QA Pass
PM → Ready **P6-E3-T1** (Calendar → Observations) unless sequencing note says otherwise.
