# Dev → QA: P2-E1-T3

## Meta
- **Task ID:** P2-E1-T3
- **Title:** Persist ingest → `ObservationRepository` (+ mid-batch 503 contract)
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E1-T3; brief `docs/handoffs/P2-E1-T3-pm-brief.md`
- **Branch:** `phase/2-ingest-http`

## What changed
- Persist worker: `spawn_persist_worker(rx, db)` drains bounded channel via `blocking_recv` (idle, no spin) and appends with `ObservationRepository::insert`.
- Duplicate PK → `StorageError::DuplicateObservation` logged; **no overwrite** (HTTP `202` = queued, not committed).
- Mid-batch contract **C**: on `queue_full` / `queue_closed`, stop further enqueue; respond **503** with `{"error":"…","accepted":N,"rejected":M}`. Already-queued items stay in the channel.
- API doc: `docs/09-api.md` documents contract C + persist semantics.
- Tests: mid-batch / first-item full 503 counts; HTTP→SQLite round-trip (`get_by_id` / list); duplicate no-overwrite.

### Crates / files
- `crates/ingest/src/persist.rs` (new)
- `crates/ingest/src/{lib,routes}.rs`, `Cargo.toml` (+ `storage`, `tempfile` dev)
- `crates/ingest/tests/ingest_persist.rs` (new)
- `crates/ingest/tests/ingest_http.rs` (mid-batch cases)
- `docs/09-api.md`
- `Cargo.lock`

## How to verify (commands)
```bash
cargo check -p ingest
cargo test -p ingest
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Accepted Observations written via `ObservationRepository::insert` (async worker from bounded channel)
- [ ] AC2: Duplicate PK → explicit error / no silent overwrite
- [ ] AC3: Bounded channel; full → **503** with documented JSON body
- [ ] AC4: Mid-batch contract **C** documented in `docs/09-api.md` + covered by tests (`accepted`/`rejected`)
- [ ] AC5: Round-trip HTTP ingest → DB `get_by_id` / list (temp DB)
- [ ] AC6: No Feature/pipeline; no Tauri host wire (→ **T4**)
- [ ] Global DoD: no `unwrap`/`expect` in prod paths; UI↛DB; idle worker blocks on recv

## Risks / not covered
- Desktop does **not** start ingest + persist worker yet → **P2-E1-T4** (`IngestConfig::load` + lifecycle).
- Duplicate after queue: HTTP still `202`; rejection is persist-time (log + no overwrite), not HTTP `409`.
- `queue_closed` mid-batch shape is implemented but not separately integration-tested (same code path as `queue_full`).
- Collectors must retry `rejected` items after backpressure.

## Notes for QA
- Contract choice: **C** (503 + counts), not A/B — see brief options and `docs/09-api.md`.
- Host wiring pattern for T4: open DB → `observation_channel` → `spawn_persist_worker(rx, db)` → `serve_*` with `tx`.
- Prefer temp DB paths in manual checks (tests use `tempfile`).
