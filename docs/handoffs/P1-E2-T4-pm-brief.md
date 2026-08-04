# PM Brief → QA: P1-E2-T4

**From:** PM  
**To:** QA (lead)  
**Status:** In Progress (assigned)  
**Date:** 2026-08-03  
**Closed previous:** P1-E2-T3 — QA Pass (`docs/handoffs/P1-E2-T3-qa-to-pm.md`)

## Task
**P1-E2-T4 — Storage integration tests**

## Modules
`crates/storage` — tests (in-memory / temp dir); production API менять только при Fail → дефект Dev

## Depends on
P1-E2-T3 Done (`ObservationRepository`)

## Acceptance Criteria
1. Покрыть integration-слой из `/docs/11-testing.md` для storage:
   - migrate (уже есть — не регрессировать)
   - insert + read round-trip
   - duplicate id → `DuplicateObservation`
   - `list_by_time_range` / `list_by_data_type` (в т.ч. пустой результат, границы окна)
2. Edge cases обрабатываются **предсказуемо** (явная ошибка или отказ deserialize — задокументировать в handoff):
   - invalid confidence при построении `Observation` (через `bio-spec`)
   - `list_by_time_range` с `end < start` → `InvalidTimeRange`
   - при возможности: битый JSON в БД / ошибка чтения payload (если безопасно симулировать SQL insert)
3. `cargo test -p storage` зелёный; без `unwrap()`/`expect()` в новом production `src/` (в тестах допустимо).
4. Схема `observations` freeze — не менять.

## Process note
Для T4 исполнитель = **QA**. По завершении сразу `docs/handoffs/P1-E2-T4-qa-to-pm.md` (вердикт + что покрыто).  
Если найден дефект API — Fail + список для Dev; не закрывать T4 как Pass.

## Out of scope
- Epic E3 Tauri
- Pipeline / features
- Расширение публичного API «на всякий случай»

## After QA
PM закрывает Epic E2 и выдаёт **P1-E3-T1** Dev.
