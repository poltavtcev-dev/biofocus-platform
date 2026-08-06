# QA → PM: P6-E3-T1

## Meta
- **Task ID:** P6-E3-T1
- **Title:** Calendar → Observations (dogfood source)
- **Date:** 2026-08-06
- **Dev/UX handoff:** `docs/handoffs/P6-E3-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p bio-spec -p macos-collector -p pipeline -p ingest` → **all pass** (incl. synthetic + ICS fixture → storage; calendar stop/idle; normalize strips titles; contracts round-trip).
  - `cargo check -p desktop` → **ok** (host wire compiles).
- AC results:
  - **AC1** Pass — opt-in `CalendarPlugin` emits `calendar_event` Observations; shape in `docs/07-contracts.md` / `09-api.md`.
  - **AC2** Pass — local ICS only; no Google/Outlook OAuth code path.
  - **AC3** Pass — rare poll (≥60s production; tests use short interval); `select!` + stop join; `calendar_stop_halts_periodic_probe_work` asserts no post-stop polls.
  - **AC4** Pass — payload builders omit title/body; ICS parser ignores SUMMARY/DESCRIPTION; channel-full logs Observation `id` only; pipeline strips title/description/attendees.
  - **AC5** Pass — scripted probe + ICS fixture integration tests → SQLite via persist worker.
  - **AC6** Pass — `docs/handoffs/P6-E3-T1-dev-to-qa.md` with operator smoke notes.
- Extra checks:
  - Global DoD: no `unwrap`/`expect` in new calendar production modules; UI↛DB (collector → channel → persist); glossary updated.
  - Out of scope respected: no MeetingDensity Features, no Life Event UI, no new SQLite schema.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P6-E3-T1 → Done; Ready → **P6-E3-T2**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` + brief `P6-E3-T2-pm-brief.md`

## Suggested next Ready task
- **P6-E3-T2** — MeetingDensity + RecoveryBetweenMeetings Features (from `calendar_event` Observations)

## Notes for PM
- Dogfood source is **local ICS export** (`BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS`), not live EventKit — acceptable for T1 AC; note for future if Calendar.app direct probe is desired.
- Non-blocking limits: no `RRULE` expansion; floating (non-`Z`) ICS datetimes treated as UTC.
- Branch remains `phase/6-life-context`; no Done/canvas updates by QA.
