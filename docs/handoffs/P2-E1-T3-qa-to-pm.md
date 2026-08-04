# QA → PM: P2-E1-T3

## Meta
- **Task ID:** P2-E1-T3
- **Title:** Persist ingest → `ObservationRepository` (+ mid-batch 503 contract)
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E1-T3-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/2-ingest-http` (T1–T3 на ветке; commit/PR → `main` всё ещё открыт)

## What was verified
- Commands run + results:
  - `cargo check -p ingest` → **ok**
  - `cargo test -p ingest` → **ok** — **24** tests
    - unit (`lib`): 8
    - `ingest_http`: 11 (incl. mid-batch `accepted:1,rejected:1` + first-item `accepted:0,rejected:2`)
    - `ingest_persist`: 2 (HTTP→SQLite round-trip + duplicate no-overwrite)
    - `pairing_token`: 3
- AC results (pass/fail per item):
  - **AC1 Pass** — `spawn_persist_worker` drains via `blocking_recv` → `ObservationRepository::insert`
  - **AC2 Pass** — duplicate PK → `StorageError::DuplicateObservation` (log, no overwrite); payload unchanged in test
  - **AC3 Pass** — bounded `try_send`; full → **503** `{"error":"queue_full","accepted":N,"rejected":M}`
  - **AC4 Pass** — mid-batch **contract C** in `docs/09-api.md` + tests (`post_ingest_mid_batch_…`, `…_accepted_zero`)
  - **AC5 Pass** — `http_ingest_persists_to_sqlite_round_trip`: `get_by_id` + `list_by_data_type` / time range (temp DB)
  - **AC6 Pass** — no Feature/pipeline; no Tauri host wire (→ **T4**)
  - **Global DoD Pass** — no `unwrap`/`expect` in `persist.rs` / `routes.rs` prod; UI↛DB; idle blocks on recv
- Extra checks (edge / security):
  - HTTP `202` = queued, not committed — documented in `09-api.md` § Persist
  - Already-queued mid-batch items stay in channel (no rollback) — matches contract C
  - `queue_closed` same JSON shape in code; no dedicated integration test (same path as `queue_full`)

## Defects (if any)
- Нет блокеров AC.
- **Note:** duplicate после enqueue → HTTP всё ещё `202`; отказ на persist (log + no overwrite), не HTTP `409` — intentional / documented.
- **Note:** `phase/2-ingest-http` — T3 changes (+ T1/T2) всё ещё **uncommitted**; нужен **commit + PR → main** (carry с T1/T2).
- **Note (out of scope):** host не стартует ingest + persist → **P2-E1-T4**.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P2-E1-T3 → Done; Ready → **P2-E1-T4**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs: `ARCHITECTURE_STATUS.md`, `docs/14-roadmap.md`, `docs/12-development.md` — persist + contract C landed; next T4
- [ ] Brief для Dev: `docs/handoffs/P2-E1-T4-pm-brief.md` (если ещё нет)
- [ ] Напомнить Dev: commit + PR `phase/2-ingest-http` → `main` (T1+T2+T3 вместе ок)

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T3 Done; Ready T4; contract C locked
- [x] ARCHITECTURE_STATUS, 14-roadmap, 12-development
- [x] Brief `docs/handoffs/P2-E1-T4-pm-brief.md`
- [x] Execution canvas
- [ ] Commit T3 on branch (not sprint PR yet — per sprint workflow)

## Suggested next Ready task
- **P2-E1-T4** — Host wire + `GET /v1/status` + `IngestConfig::load` (+ idle DoD)

## Notes for PM
- Публичный API: `ingest::spawn_persist_worker(rx, db)`.
- Host pattern для T4: open DB → `observation_channel` → `spawn_persist_worker(rx, db)` → `serve_*` with `tx`.
- Contract choice locked: **C** (503 + counts), not A/B.
- Параллельно по-прежнему доступен маленький **P2-E0-T1** (sanitize `dbError`), если нужен.
