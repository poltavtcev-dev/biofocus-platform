# QA → PM: P1-E2-T4

## Meta
- **Task ID:** P1-E2-T4
- **Title:** Storage integration tests
- **Date:** 2026-08-03
- **Dev/UX handoff:** n/a (QA = исполнитель по brief `docs/handoffs/P1-E2-T4-pm-brief.md`)
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p storage` → **ok** (17 tests: 2 migrate + 3 open + 12 observation_repo; 0 failed)
- AC results (pass/fail per item):
  - **AC1 Pass** — Integration layer (`docs/11-testing.md`):
    - migrate: без регрессии (`open_creates_observations_table_and_indexes`, `migrate_twice_is_idempotent`)
    - insert + read round-trip (`insert_then_get_by_id_round_trip`)
    - duplicate id → `DuplicateObservation` (`insert_duplicate_pk_returns_explicit_error`)
    - `list_by_time_range` / `list_by_data_type`: inclusive bounds, point window, empty window, empty type (`list_by_time_range_*`, `list_by_data_type_filters_and_empty`)
  - **AC2 Pass** — Edge cases предсказуемы (см. Notes):
    - invalid confidence при построении `Observation` → `SpecError::InvalidConfidence`
    - `end < start` → `StorageError::InvalidTimeRange`
    - битый JSON в БД (raw SQL) → `StorageError::PayloadDeserialize` на `get_by_id` и `list_by_time_range`
    - invalid confidence в строке БД → `StorageError::Domain(SpecError::InvalidConfidence)`
  - **AC3 Pass** — `cargo test -p storage` зелёный; в `crates/storage/src/` нет `unwrap()`/`expect()`
  - **AC4 Pass** — схема `observations` не менялась
- Extra checks (edge / security):
  - `get_by_id` missing → `Ok(None)`
  - corrupt row в list: fail-fast на `PayloadDeserialize` (не partial list) — задокументировано ниже
  - production API не расширялся; только тесты + комментарий в `lib.rs`

## Defects (if any)
- Нет.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E2-T4** to Done; закрыть Epic E2; Ready → **P1-E3-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` — storage suite note + desktop commands hint

## Suggested next Ready task
- **P1-E3-T1** — Scaffold `apps/desktop` (Tauri v2 + React), Role: Dev  
  (параллельно UX: **P1-E3-T2** после T1)

## Notes for PM
- Файл тестов: `crates/storage/tests/observation_repo.rs` (расширен с 3 → 12 кейсов).
- Задокументированное поведение edge:
  1. `Observation::try_new` / `Confidence::try_new` → `SpecError::InvalidConfidence` (не доходит до storage).
  2. `list_by_time_range(end < start)` → `InvalidTimeRange` до SQL.
  3. Corrupt payload JSON (симуляция raw INSERT) → `PayloadDeserialize`; list не возвращает partial Vec.
  4. Out-of-range confidence в БД → `Domain(InvalidConfidence)` при map row → Observation.
- Epic E2 (Storage Foundation) можно закрывать после PM sync.
