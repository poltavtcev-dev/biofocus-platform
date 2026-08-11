# Dev → QA: P18-E3-T1

## Meta
- **Task ID:** P18-E3-T1
- **Title:** Ship catalog Feature `NotificationPressure`
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P18-E3-T1-pm-brief.md`

## What changed
- Catalog Feature **`NotificationPressure`**: sum in-window `notification_event.count` → intensity `clamp(100 × sum / 20, 0, 100)` (saturation 20 / 15m); omit empty; ADR-007 single-family confidence; optional label factors (`category` → else `interruption_level` → else `app_kind`; shares sum to 1.0).
- Registered via `feature_engine::register_notification_v1` and wired into `register_catalog_v1` (Feature Worker / `get_feature_snapshot` / series path pick it up automatically — no new Dashboard UI).
- Docs finalized: `06-feature-catalog` §1.15 shipped; glossary; `12-development`; `08-plugin-sdk`; ADR-019 E3 note.
- **No** SQLite migration; **no** new ADR; **no** live NC OS probe expansion; **no** CognitiveLoad / Insights / dedicated chart.

## Crates / apps / files touched
- `crates/feature-engine/src/catalog/notification_pressure.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`, `crates/feature-engine/src/lib.rs`
- Docs: `06-feature-catalog.md`, `16-glossary.md`, `12-development.md`, `08-plugin-sdk.md`, `decision-log.md` (E3 shipped note)
- Branch: `phase/18-notification-pressure`

## How to verify (commands)
```bash
cargo test -p feature-engine notification_pressure
cargo test -p feature-engine

# Registration + formula anchors
rg -n "NotificationPressure|register_notification_v1|SATURATION_COUNT" \
  crates/feature-engine/src docs/06-feature-catalog.md

# No migration / no CognitiveLoad wiring
rg -n "CognitiveLoad" crates/feature-engine || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Catalog §1.15 finalized (goal, 15m/1m, 0–100, inputs, formula, omit, provenance, ADR-007, DAG); calm framing
- [ ] AC2: Inputs `notification_event`; sum count → 0–100; empty omit; optional label factors when present
- [ ] AC3: `register_notification_v1` in `register_catalog_v1`; appears on Feature Worker / snapshot path when inputs present
- [ ] AC4: Unit tests — rich emit; empty omit; confidence; no content keys required
- [ ] AC5: Docs catalog + glossary / 12-development; Phase 18 Feature marked shipped
- [ ] AC6: No migration; no new ADR; soft-fail OS probe OK
- [ ] AC7: Handoff `docs/handoffs/P18-E3-T1-dev-to-qa.md`
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; personal self-tracking; PR freeze

## Risks / not covered
- Live Notification Center mapping may remain soft-fail idle — Feature verified on fixture Observations (same as AmbientLightShare / GitActivityRate).
- Saturation anchor **20** is v1-documented; change would need catalog/ADR note if revisited.

## Notes for QA
- Kanban Done / canvas are PM-only after QA Pass.
- Pre-existing dirty docs from PM E2 close (`00-vision` / roadmap / etc.) may exist — out of this task unless needed for AC5.
