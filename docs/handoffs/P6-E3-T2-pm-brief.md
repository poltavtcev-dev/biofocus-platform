# PM Brief → Dev: P6-E3-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-06  
**Closed previous:** P6-E3-T1 (QA Pass with notes — Calendar → Observations via local ICS; dogfood source)  
**Evidence:** `docs/handoffs/P6-E3-T1-qa-to-pm.md`

## Task
**P6-E3-T2 — MeetingDensity + RecoveryBetweenMeetings Features**

## Why
`calendar_event` Observations now land in storage (opt-in ICS). Phase 6 dogfood needs Core Features that turn meeting load and gaps into calm, catalogued metrics — inputs for later CognitiveLoad / Pattern Discovery without clinical claims.

## Acceptance Criteria
1. `MeetingDensity` and `RecoveryBetweenMeetings` are **registered** in `feature-engine` and computed from `calendar_event` Observations (uid/start/end/busy; no titles required).
2. Catalog docs in `docs/06-feature-catalog.md`: formula / window / units / deps / provenance for both Features (move out of “planned backlog” into §1 specs).
3. Unit tests with **synthetic** `calendar_event` Observations (empty window, dense meetings, gaps).
4. Idle-safe: no busy-loop; Features compute on existing snapshot/DAG path (same pattern as Focus/Stress).
5. Calm naming / copy — no burnout or clinical diagnosis claims.
6. Handoff: `docs/handoffs/P6-E3-T2-dev-to-qa.md`.

## Out of scope
- Feature-level confidence / explanation factors (→ Phase 7)
- `CognitiveLoad`, Pattern Discovery, ActivityBalance
- Calendar UI / EventKit live probe / RRULE expansion
- New SQLite schema
- Cloud calendar OAuth

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `crates/feature-engine`, `docs/06-feature-catalog.md`; wire into existing register/snapshot path if IPC consumers already expect catalog Features
- Input contract: `calendar_event` from P6-E3-T1 (`docs/07-contracts.md` / `08-plugin-sdk.md`)
- Branch: `phase/6-life-context`
- Note from QA: ICS dogfood may omit RRULE expansion; floating (non-`Z`) times treated as UTC — Features should tolerate partial/imperfect calendars

## After QA Pass
PM → close Epic **P6-E3** (and Phase 6 Kanban if no further P6 tasks) unless sequencing note says otherwise. Cluster PR on `phase/6-life-context` when ready to ship.
