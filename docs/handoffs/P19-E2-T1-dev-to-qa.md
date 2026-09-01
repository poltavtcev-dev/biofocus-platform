# Dev → QA: P19-E2-T1

## Meta
- **Task ID:** P19-E2-T1
- **Title:** Implement live `SystemNotificationEventProbe` mapping (ADR-020)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P19-E2-T1-pm-brief.md`

## What changed
- Live **`SystemNotificationEventProbe`** reads macOS **usernoted** Notification Center SQLite (ADR-020):
  - Primary path: `~/Library/Group Containers/group.com.apple.usernoted/db2/db`
  - Fallback: `$DARWIN_USER_DIR/com.apple.notificationcenter/db2/db`
  - **Allowlisted SELECT only:** `record.delivered_date` + `app.identifier` (bundle → closed-set labels in-memory)
  - **Never** selects `record.data` / title / body / userInfo / attachments
  - Soft-fail when missing / TCC denied / schema mismatch; first open sets watermark (no history dump)
- `ScriptedNotificationEventProbe` kept; `with_db_path` + `write_fixture_nc_db` for tests
- `ingest_host` env gate unchanged; pipeline strip still applies
- Docs marked live probe **shipped**: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`
- **No** `NotificationPressure` rewrite; **no** migration; **no** `bundle_id` in Observation payload

## Crates / apps / files touched
- `crates/macos-collector/src/notification_nc_db.rs` (new)
- `crates/macos-collector/src/notification_probe.rs`, `notification_plugin.rs`, `lib.rs`
- `crates/macos-collector/Cargo.toml` (`rusqlite`)
- `crates/macos-collector/tests/collector_integration.rs`
- Docs: `07`, `08`, `10`, `12`, `16`
- Branch: `phase/19-live-nc-mapping`

## How to verify (commands)
```bash
cargo test -p macos-collector notification
cargo test -p macos-collector --test collector_integration notification_event

# Named OS surface + allowlist
rg -n "usernoted|delivered_date|record.data|USERNOTED_DB" \
  crates/macos-collector/src/notification_nc_db.rs docs/07-contracts.md docs/08-plugin-sdk.md

# No Feature rewrite / no bundle_id payload
rg -n "NotificationPressure|bundle_id" crates/feature-engine/src/catalog/notification_pressure.rs \
  crates/macos-collector/src/payload.rs
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Live mapping emits ADR-019 payloads only; never reads/logs/persists content fields
- [ ] AC2: Soft-fail idle when unavailable; ≥5s / change emit; stop_stream joins
- [ ] AC3: ingest_host env gate unchanged; channel → validate/normalize → persist
- [ ] AC4: Scripted + fixture tests; soft-fail no emit; stop freezes; no content keys
- [ ] AC5: Docs name OS surface (usernoted SQLite) as shipped; allowlist obeyed
- [ ] AC6: No Feature rewrite; no migration; no payload widening
- [ ] AC7: Handoff `docs/handoffs/P19-E2-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB; personal self-tracking; PR freeze

## Risks / not covered
- Real Mac dogfood may need **Full Disk Access** for Group Containers; without it probe soft-fails (documented). Calm status UX → **P19-E3**.
- `interruption_level` omitted from live path in v1 (would require content-bearing blobs).

## Notes for QA
- Prefer filter `notification` / `notification_event` — full git live tests may need sandbox `all` for `git init`.
- Kanban Done / canvas are PM-only after QA Pass.
