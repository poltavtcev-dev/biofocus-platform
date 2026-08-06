# Dev → QA: P6-E3-T1

## Meta
- **Task ID:** P6-E3-T1
- **Title:** Calendar → Observations (dogfood source)
- **Role that built:** Dev
- **Date:** 2026-08-06
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P6-E3 / P6-E3-T1; brief `docs/handoffs/P6-E3-T1-pm-brief.md`

## What changed
- Opt-in local Calendar collector (`CalendarPlugin`) reads a **local `.ics` file** and emits `calendar_event` Observations into the shared channel → persist worker (same path as active-window / keystrokes).
- Contract: `data_type: "calendar_event"`, provider `com.biofocus.macos.calendar`, payload `{uid, start, end, all_day?, busy?}` — **no titles/bodies**.
- Env: `BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS=/path/to/file.ics` (default **off**). Rare poll ≥60s; emit-once per `(uid,start,end)`; lookaround past 24h / future 48h.
- Host wire in `ingest_host` (start/stop lifecycle). Soft-fail if enabled without ICS path.
- `bio-spec` validation + ingest `400 invalid_calendar_event`; pipeline normalize strips title/description/attendees.
- Docs: `07-contracts`, `08-plugin-sdk`, `09-api`, `10-security`, `12-development`, `16-glossary`, `06-feature-catalog`.

### Crates / apps / files touched
- `crates/bio-spec` — `calendar_event.rs`, error, validate hook, contracts test
- `crates/macos-collector` — `calendar_*`, `ics`, payload, lib; integration tests
- `crates/pipeline` — normalize `calendar_event`
- `crates/ingest` — error code mapping
- `apps/desktop/src-tauri/src/ingest_host.rs` — host lifecycle
- docs listed above

## How to verify (commands)
```bash
cargo test -p bio-spec -p macos-collector -p pipeline -p ingest
cargo check -p desktop
```

Dev ran these successfully (2026-08-06): all listed crate tests **pass**; `cargo check -p desktop` **ok**.

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Opt-in local Calendar source produces Calendar/meeting Observations (shape in contracts / API)
- [ ] AC2 No cloud calendar sync (Google/Outlook OAuth) required for dogfood
- [ ] AC3 Idle-safe: rare poll (≥60s) / `select!` stop — no busy-loop
- [ ] AC4 Privacy: no event titles/bodies leaked to logs beyond Observation needs (payload has no title; logs use Observation `id` on drop)
- [ ] AC5 Tests with fixtures (synthetic calendar + ICS → Observations)
- [ ] AC6 Handoff with operator smoke notes (this file)
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Floating ICS datetimes (non-`Z`) treated as UTC for dogfood — document limitation; EventKit / Calendar.app live API not wired (ICS export is the dogfood path).
- MeetingDensity / RecoveryBetweenMeetings Features → **P6-E3-T2**.
- Recurring `RRULE` expansion not implemented (only explicit VEVENT instances in the ICS).

## Notes for QA / operator smoke
1. Export or craft a small `.ics` with 1–2 VEVENTs near “now” (UTC `Z` times preferred).
2. Restart Desktop with:
   ```bash
   export BIOFOCUS_CALENDAR=1
   export BIOFOCUS_CALENDAR_ICS="$HOME/path/to/dogfood.ics"
   ```
3. Confirm logs: collector armed; **no** SUMMARY/title strings in tracing.
4. Query storage / status path for `data_type=calendar_event` rows with `uid`/`start`/`end` only.
5. Without env flags, collector stays off (default).
6. Unit/integration without Desktop: `cargo test -p macos-collector --test collector_integration`.
