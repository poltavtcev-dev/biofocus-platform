# Dev → QA: P6-E3-T2

**From:** Dev  
**To:** QA  
**Date:** 2026-08-06  
**Branch:** `phase/6-dogfood-fixes` (tip with T1 / dogfood fixes; brief also named `phase/6-life-context`)

## Meta
- **Task ID:** P6-E3-T2
- **Title:** MeetingDensity + RecoveryBetweenMeetings Features
- **Role that built:** Dev
- **AC source:** `docs/handoffs/P6-E3-T2-pm-brief.md` / `docs/SPRINT_ROADMAP.md` → P6-E3-T2

## What changed
- Registered `MeetingDensity` and `RecoveryBetweenMeetings` in `feature-engine` (DAG nodes + `register_calendar_v1`; included in `register_catalog_v1` so existing Feature Worker / snapshot path picks them up).
- Both Features read `calendar_event` Observations (`uid` / `start` / `end`; optional `busy` / `all_day`). Titles not required. Malformed rows skipped (partial calendars OK).
- Catalog specs moved from planned backlog → §1.5 / §1.6 in `docs/06-feature-catalog.md` (formula / window / units / deps / provenance). Entrypoint note in `docs/12-development.md`.
- Unit tests: empty window, dense/full booking, half density, free gaps, back-to-back (no recovery Feature), idle empty snapshot.

### Formula (v1)
| Feature | Value |
| :--- | :--- |
| `MeetingDensity` | Merged busy overlap secs clipped to 15m window ÷ 900 → `[0.0, 1.0]` |
| `RecoveryBetweenMeetings` | Mean free gap minutes between consecutive busy meetings for gaps intersecting the window |

Busy rules: missing `busy` ⇒ busy; `busy: false` skipped; `all_day: true` skipped.

### Crates / files
- `crates/feature-engine/src/catalog/calendar_meeting.rs` (new helpers)
- `crates/feature-engine/src/catalog/meeting_density.rs` (new)
- `crates/feature-engine/src/catalog/recovery_between_meetings.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs` — `register_calendar_v1` + wire into `register_catalog_v1`
- `crates/feature-engine/src/lib.rs` — re-exports
- `docs/06-feature-catalog.md`, `docs/12-development.md`

## How to verify (commands)
```bash
cargo test -p feature-engine
cargo test -p pipeline --test pipeline_e2e
cargo check -p feature-engine
```

Dev ran all three — pass (37 feature-engine tests; 3 pipeline e2e).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `MeetingDensity` + `RecoveryBetweenMeetings` registered and computed from `calendar_event` (uid/start/end/busy; no titles)
- [ ] AC2: Catalog docs in `docs/06-feature-catalog.md` §1 (formula / window / units / deps / provenance); removed from planned backlog table
- [ ] AC3: Unit tests with synthetic calendar Observations (empty, dense, gaps)
- [ ] AC4: Idle-safe — empty snapshot / no calendar → empty Features; same DAG/`register_catalog_v1` path as Focus/Stress; no busy-loop
- [ ] AC5: Calm naming / copy — no burnout or clinical diagnosis claims
- [ ] AC6: This handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- RRULE expansion still absent (ICS dogfood note from T1) — Features see only expanded/emitted instances.
- Floating (non-`Z`) ICS times still treated as UTC upstream — Features use payload ints as-is.
- Dashboard charts do **not** add these series (out of scope; unknown Feature ids are ignored by chart filter).
- Alert mapping unchanged (still Stress/Fatigue/`High_Stress` only).
- No CognitiveLoad / Pattern Discovery / ActivityBalance.

## Notes for QA
- Prefer `register_calendar_v1` for focused tests; production path uses `register_catalog_v1` (already in Feature Worker).
- Out of scope confirmed: confidence/explanation factors, new SQLite schema, EventKit live probe, cloud OAuth, Calendar UI.
