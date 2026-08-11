# PM Brief → Dev: P18-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P18-E1-T1** — ADR-019 contracts locked (QA Pass)  
**Evidence:** `docs/handoffs/P18-E1-T1-qa-to-pm.md` · `docs/decision-log.md` ADR-019 · `docs/07-contracts.md`

## Task
**P18-E2-T1 — Implement notification collector plugin per ADR-019**

## Why
ADR-019 locked `notification_event` payloads and `NotificationPressure` Feature scope. Next: emit real Observations from an opt-in macOS Capability Plugin so E3 can compute interruption intensity from evidence — not invented LLM values.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.notifications` with Capability covering `notification_event`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` / ADR-019.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_NOTIFICATION_EVENTS=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-019 / `docs/07-contracts.md`: required `count` (≥1); optional closed-set `category` / `interruption_level` / `app_kind` only. **Never** persist or log notification body, title text, message content, screenshots, or always-on content dumps.
4. Soft-fail OS probe when mapping unavailable (idle; **no** emit / **no** busy-loop); injectable **scripted probe** for tests; emit on change / coalesce via `count` or rare poll (≥5s); `stop_stream` joins background work.
5. Core ingest accepts the type: `bio_spec` validator (e.g. `validate_notification_event_payload`) + pipeline normalize if needed; reject malformed calmly; persist to existing `observations` store.
6. Docs as shipped for collector path: `12-development` / `08-plugin-sdk` / contracts as needed. Feature math remains → **P18-E3**.
7. Tests: validators + scripted probe emit → channel path; soft-fail idle; after `stop_stream`, emission freezes.
8. **No** `NotificationPressure` Feature formula; **no** SQLite migration; **no** IDE/weather/App Store work; **no** content capture “for accuracy”.
9. Handoff: `docs/handoffs/P18-E2-T1-dev-to-qa.md`.

## Out of scope
- `NotificationPressure` / feature-engine DAG (→ **P18-E3-T1**)
- `CognitiveLoad` wiring
- IDE / weather / App Store / Companion dogfood as this phase primary
- Workplace / employee-surveillance framing
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-019 is payload SoT
- Branch: `phase/18-notification-pressure`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel marketplace crate
- Personal self-tracking only
- LLM remains L5 interpret-only — must not invent notification payloads or Features

## After QA Pass
PM → mark P18-E2-T1 Done; Ready **P18-E3-T1** (`NotificationPressure`).
