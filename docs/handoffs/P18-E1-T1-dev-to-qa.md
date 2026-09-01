# Dev → QA: P18-E1-T1

## Meta
- **Task ID:** P18-E1-T1
- **Title:** Lock notification Observation + `NotificationPressure` scope as ADR-019 + docs
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P18-E1-T1-pm-brief.md`

## What changed
- **ADR-019** in `docs/decision-log.md` (summary table + detail): Phase 18 v1 locks
  - Observation family `data_type: "notification_event"` (not a separate `notification_burst` family — coalescing via `count`)
  - Privacy bar: required `count` ≥ 1; optional closed-set `category` / `interruption_level` / `app_kind`; **forbid** body/title/subtitle/message/screenshots/attachments/userInfo
  - Opt-in `BIOFOCUS_NOTIFICATION_EVENTS` default **off**; event or rare ≥5s poll; no busy-loop; Capability Plugin `com.biofocus.macos.notifications` via `plugin-sdk` + `macos-collector`
  - Existing `observations` store; **no** migration
  - E3 Feature = **`NotificationPressure`** (omit-when-empty + ADR-007 single-family confidence sketch)
  - Rejected: IDE / weather / App Store primary; content capture; workplace surveillance; `CognitiveLoad` as E3; PR during freeze; etc.
- Contract sketches updated: `07-contracts`, `08-plugin-sdk`, `06-feature-catalog` (§1.15 stub + backlog cleanup), `10-security`, `12-development`, `16-glossary`, `04-storage`; Phase 18 epic text aligned in `SPRINT_ROADMAP`.
- **No** collector Swift/Rust emit; **no** Feature formula implementation; **no** SQLite migration applied.

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/18-notification-pressure`

## How to verify (commands)
```bash
# ADR + contracts present
rg -n "ADR-019" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/06-feature-catalog.md docs/10-security.md docs/12-development.md docs/16-glossary.md docs/04-storage.md docs/SPRINT_ROADMAP.md

# Locked data_type + privacy + env
rg -n "notification_event|BIOFOCUS_NOTIFICATION_EVENTS|NotificationPressure" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/06-feature-catalog.md

# Rejected alts
rg -n "IDE as Phase 18|Content capture|CognitiveLoad as E3|workplace|PR during freeze" docs/decision-log.md

# No collector / Feature code / migration for this task
rg -n "notification_event|validate_notification_event|NotificationPressure" crates apps || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-019 locks one Observation family, privacy bar, opt-in/plugin model, no migration, NotificationPressure E3 sketch, rejected alts
- [ ] AC2: Contracts / catalog / security / development / glossary updated; impl → E2/E3
- [ ] AC3: ADR text aligned with Phase 18 epic split (E1 ADR · E2 collector · E3 Feature)
- [ ] AC4: No collector emit code; no Feature formula beyond catalog stub; no SQLite migration applied
- [ ] AC5: Handoff `docs/handoffs/P18-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB stance unchanged; personal self-tracking; PR freeze respected

## Risks / not covered
- macOS Notification Center API feasibility is for **P18-E2** (soft-fail idle when unavailable — locked in ADR).
- Exact intensity map anchors for `NotificationPressure` deferred to E3.

## Notes for QA
- Do **not** expect `cargo` crates to know `notification_event` yet — validators ship in E2.
- Kanban Done / canvas are PM-only after QA Pass.
