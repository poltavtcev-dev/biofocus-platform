# PM Brief → Dev: P6-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P6-E2-T1 (QA Pass with notes — Desktop quick-log Life Events; Epic **P6-E2** ✅)  
**Evidence:** `docs/handoffs/P6-E2-T1-qa-to-pm.md`

## Task
**P6-E3-T1 — Calendar → Observations (dogfood source)**

## Why
Life Events can be logged manually. Meeting-density Features need Calendar-derived Observations as inputs — dogfood needs an opt-in local Calendar path into the same Observation store, without cloud OAuth.

## Acceptance Criteria
1. Opt-in local Calendar source produces Calendar/meeting `Observation`s (shape documented in contracts / API).
2. No cloud calendar sync (Google/Outlook OAuth) required for dogfood.
3. Idle-safe: event-driven refresh or rare poll — no busy-loop.
4. Privacy: no event titles/bodies leaked to logs beyond what’s needed for the Observation payload.
5. Tests with fixtures (synthetic calendar → Observations).
6. Handoff with operator smoke notes: `docs/handoffs/P6-E3-T1-dev-to-qa.md`.

## Out of scope
- Google/Outlook cloud OAuth
- `MeetingDensity` / `RecoveryBetweenMeetings` formula (→ **P6-E3-T2**)
- Life Event UI changes
- New SQLite schema without ADR + approve

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: collector / host plugin path + contracts docs
- Observation contract discipline (same store as ingest / Life Events; UI ↛ SQLite)
- Branch: `phase/6-life-context`

## After QA Pass
PM → Ready **P6-E3-T2** (MeetingDensity + RecoveryBetweenMeetings Features) unless sequencing note says otherwise.
