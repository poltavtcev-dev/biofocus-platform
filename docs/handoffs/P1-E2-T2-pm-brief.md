# PM Brief → Dev: P1-E2-T2

**From:** PM  
**To:** Dev  
**Status:** In Progress (assigned)  
**Date:** 2026-07-28

## Task
**P1-E2-T2 — Migration: `observations` table**

## Modules
`crates/storage` (migrations поверх уже готового `Database::open`)

## Depends on
P1-E2-T1 (Done), schema freeze `/docs/04-storage.md`

## Acceptance Criteria
1. Миграция создаёт таблицу `observations` и индексы **точно** по `/docs/04-storage.md`:
   - columns: `id`, `timestamp`, `provider_id`, `data_type`, `payload`, `confidence`, `created_at`
   - indexes: `idx_obs_ts`, `idx_obs_type_ts`
2. Повторный запуск идемпотентен (versioned migrations и/или `IF NOT EXISTS`).
3. Миграции вызываются при open (или явный `migrate()` на `Database`) — путь должен быть один и документирован в handoff.
4. Ошибки — typed `Result` / `StorageError`; без `unwrap()`/`expect()` в production.
5. Schema change без ADR запрещён — не «улучшать» колонки.

## Out of scope
- `ObservationRepository` insert/get/list → **P1-E2-T3**
- Полный набор integration tests repository → **P1-E2-T4** (для T2 достаточно тестов идемпотентности миграции)
- UI / Tauri / HTTP ingest

## DoD for Dev step
- `cargo test -p storage` зелёный (включая тест «migrate twice»)
- Создан `docs/handoffs/P1-E2-T2-dev-to-qa.md` по шаблону
- В чате: «Передаю QA» — **без** перевода задачи в Done

## After Dev
Пользователь/агент: `как QA: проверь P1-E2-T2`
