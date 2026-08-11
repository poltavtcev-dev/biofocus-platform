# Dev → QA: P18-E2-T1

## Meta
- **Task ID:** P18-E2-T1
- **Title:** Implement notification collector plugin per ADR-019
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P18-E2-T1-pm-brief.md`

## What changed
- **`NotificationPlugin`** in `macos-collector`: id `com.biofocus.macos.notifications`, capability `notification_events` → `notification_event`; opt-in `BIOFOCUS_NOTIFICATION_EVENTS` (default off); poll ≥5s; soft-fail `SystemNotificationEventProbe`; injectable `ScriptedNotificationEventProbe`; `stop_stream` joins.
- Desktop **`ingest_host`** arms the plugin only when env is set; same Observation channel → persist worker.
- **`bio_spec::validate_notification_event_payload`** + ingest reject `invalid_notification_event`; pipeline `normalize_notification_event` keeps `count` + closed-set labels and strips `title` / `body` / `subtitle` / `message` / `screenshot` / `userInfo`.
- Docs marked collector shipped: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`.
- **No** `NotificationPressure` Feature math; **no** SQLite migration; **no** content capture.

## Crates / apps / files touched
- `crates/bio-spec` — `notification_event.rs`, router, contracts test
- `crates/ingest` — reject code
- `crates/pipeline` — normalize + tests
- `crates/macos-collector` — probe / stream / plugin / payload + `collector_integration` tests
- `apps/desktop/src-tauri/src/ingest_host.rs`
- Docs: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`
- Branch: `phase/18-notification-pressure`

## How to verify (commands)
```bash
cargo test -p bio-spec
cargo test -p pipeline
cargo test -p ingest
cargo test -p macos-collector --test collector_integration notification_event

# Plugin id / env / privacy
rg -n "com.biofocus.macos.notifications|BIOFOCUS_NOTIFICATION_EVENTS|notification_events" \
  crates/macos-collector/src/notification_plugin.rs apps/desktop/src-tauri/src/ingest_host.rs

# No Feature formula / no migration for this task
rg -n "NotificationPressure" crates/feature-engine crates/features 2>/dev/null || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Plugin id `com.biofocus.macos.notifications` + capability covering `notification_event` in `macos-collector`
- [ ] AC2: `ingest_host` starts only when `BIOFOCUS_NOTIFICATION_EVENTS=1` (default off); channel → persist (no UI→SQLite)
- [ ] AC3: Payload = required `count` ≥1 + optional closed-set labels only; no body/title/message/screenshots persisted
- [ ] AC4: Soft-fail OS probe idle; scripted probe for tests; ≥5s / change emit; `stop_stream` joins (no busy-loop)
- [ ] AC5: bio-spec validator + pipeline normalize + ingest reject; existing `observations` store
- [ ] AC6: Docs as shipped for collector path; Feature math remains → P18-E3
- [ ] AC7: Tests: validators + emit→storage (no content keys) + soft-fail idle + stop freezes polls
- [ ] AC8: No NotificationPressure formula; no migration; no IDE/weather/App Store; no content capture
- [ ] AC9: Handoff `docs/handoffs/P18-E2-T1-dev-to-qa.md`
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary; PR freeze respected

## Risks / not covered
- Real macOS Notification Center mapping remains soft-fail idle in v1 (`SystemNotificationEventProbe` → `Ok(None)`); intensity Feature → **P18-E3**.

## Notes for QA
- Prefer filter `notification_event` on `collector_integration` — full suite is longer.
- Kanban Done / canvas are PM-only after QA Pass.
