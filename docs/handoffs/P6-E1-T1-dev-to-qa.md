# Dev → QA: P6-E1-T1

## Meta
- **Task ID:** P6-E1-T1
- **Title:** Life Events Observation kinds + ADR-006
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P6-E1 / P6-E1-T1 (also `docs/handoffs/P6-E1-T1-pm-brief.md`)
- **Branch:** `phase/6-life-context` (from `origin/main`)

## What changed
- **ADR-006** in `docs/decision-log.md`: Life Events as Observation kinds (`data_type: "life_event"` + `payload.kind`); no parallel store; local-only default; v1 kinds `coffee` / `walk` / `lunch` / `workout`.
- Contracts + API docs: `docs/07-contracts.md` (JSON examples), `docs/09-api.md` (`400 invalid_life_event`), glossary note in `docs/16-glossary.md`.
- `bio-spec`: `life_event` module — `validate_life_event_payload` / `validate_observation_payload`, v1 kind constants, `SpecError::InvalidLifeEventPayload`.
- `ingest`: `POST /v1/ingest` validates Life Event payloads **before** enqueue; whole batch rejected on failure with `{"error":"invalid_life_event"}`.
- Tests: bio-spec contract + malformed reject; ingest HTTP accept/reject; ingest→SQLite round-trip for `coffee`.
- **No new SQLite schema.**

## Crates / apps / files touched
- `crates/bio-spec/src/life_event.rs` (new)
- `crates/bio-spec/src/lib.rs`, `crates/bio-spec/src/error.rs`
- `crates/bio-spec/tests/contracts.rs`
- `crates/ingest/src/routes.rs`
- `crates/ingest/tests/ingest_http.rs`, `crates/ingest/tests/ingest_persist.rs`
- `docs/decision-log.md`, `docs/07-contracts.md`, `docs/09-api.md`, `docs/16-glossary.md`

## How to verify (commands)
```bash
cargo test -p bio-spec -p ingest --tests
# Focused:
cargo test -p bio-spec --test contracts life_event
cargo test -p ingest --test ingest_http life_event
cargo test -p ingest --test ingest_persist life_event
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-006 in `docs/decision-log.md` — Observation kinds, not parallel table; local-only; v1 kinds + payload shape
- [ ] AC2: `07-contracts` / `09-api` document JSON shape + examples (+ glossary if needed)
- [ ] AC3: Ingest/bio-spec accepts valid Life Events; rejects malformed with explicit error (no `unwrap`/`expect` in prod paths)
- [ ] AC4: Round-trip ingest → Observation repository for ≥1 Life Event kind
- [ ] AC5: No new SQLite schema
- [ ] AC6: Idle-safe (validation is sync, no busy-loop)
- [ ] AC7: This handoff present
- [ ] Global DoD: no unwrap/expect in prod; UI↛DB; glossary terms

## Risks / not covered
- Desktop quick-log UI → **P6-E2-T1** (out of scope).
- Pipeline `normalize` does not yet special-case `life_event` (pass-through as unknown) — OK for T1; Features later.
- Error code for payload failures is always `invalid_life_event` (only Life Events are schema-validated at this layer today).

## Notes for QA
- Branch may not include Phase 6 Kanban docs from PM WIP on `phase/5-wearable-dogfood` (stashed separately); AC for this task is self-contained in brief + this handoff + ADR/contracts.
- Sample provider_id in contracts: `com.biofocus.desktop`.
