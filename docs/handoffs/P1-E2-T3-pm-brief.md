# PM Brief → Dev: P1-E2-T3

**From:** PM  
**To:** Dev  
**Status:** In Progress (assigned)  
**Date:** 2026-08-03  
**Closed previous:** P1-E2-T2 — QA Pass (`docs/handoffs/P1-E2-T2-qa-to-pm.md`)

## Task
**P1-E2-T3 — `ObservationRepository`**

## Modules
`crates/storage` (+ types from `bio-spec`)

## Depends on
P1-E2-T2 (Done — migrate-on-open), P1-E1-T2 (`Observation`)

## Acceptance Criteria
1. API минимум: `insert`, `get_by_id`, `list_by_time_range`, `list_by_data_type`.
2. `insert` — immutable append; дубликат PK → **явная** ошибка (`StorageError`), не silent overwrite / UPSERT.
3. `payload` хранится и читается как JSON; mapping в `bio_spec::Observation`.
4. `created_at` заполняется при insert (Unix UTC seconds); не путать с `Observation.timestamp`.
5. Ошибки — typed `Result`; без `unwrap()`/`expect()` в `src/`.
6. Минимальные unit/integration smoke-тесты на insert+get и duplicate PK (полный suite — T4).

## Out of scope
- Расширенный QA suite (битый JSON matrix и т.п.) → **P1-E2-T4**
- UI / Tauri / HTTP ingest
- Изменение схемы `observations` (freeze)

## DoD for Dev step
- `cargo test -p storage` зелёный
- `docs/handoffs/P1-E2-T3-dev-to-qa.md` по шаблону
- В чате: «Передаю QA» — **без** Done в roadmap/canvas

## After Dev
`как QA: проверь P1-E2-T3`
