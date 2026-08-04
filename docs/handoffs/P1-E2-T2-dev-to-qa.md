# Dev → QA: P1-E2-T2

## Meta
- **Task ID:** P1-E2-T2
- **Title:** Migration: `observations` table
- **Role that built:** Dev
- **Date:** 2026-08-03
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P1-E2 / P1-E2-T2; brief `docs/handoffs/P1-E2-T2-pm-brief.md`

## What changed
- Summary:
  - Добавлен модуль миграций `crates/storage/src/migrate.rs` (schema v1).
  - Таблица `observations` + индексы `idx_obs_ts`, `idx_obs_type_ts` — SQL как в `/docs/04-storage.md` (freeze).
  - Версионирование через `schema_migrations` + повторный `CREATE … IF NOT EXISTS` для идемпотентности.
  - Миграции вызываются автоматически при `Database::open` / `open_in_memory`; публичный `Database::migrate()` — для явного повторного прогона (тесты).
  - Typed error `StorageError::Clock` для метаданных `applied_at`.
- Crates / files:
  - `crates/storage/src/migrate.rs` (new)
  - `crates/storage/src/db.rs`
  - `crates/storage/src/error.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/tests/migrate.rs` (new)

## Migration path (единственный)
`Database::open` / `Database::open_in_memory` → pragmas → `migrate::run`.  
Отдельный шаг migrate для production-хоста не нужен. `Database::migrate()` только re-apply (idempotent).

## How to verify (commands)
```bash
cargo test -p storage
```

Ожидание: все тесты зелёные, в т.ч. `migrate_twice_is_idempotent` и `open_creates_observations_table_and_indexes`.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: таблица `observations` с колонками `id`, `timestamp`, `provider_id`, `data_type`, `payload`, `confidence`, `created_at`; индексы `idx_obs_ts`, `idx_obs_type_ts` (точные имена/типы по `/docs/04-storage.md`)
- [ ] AC2: повторный open / `migrate()` идемпотентен (нет ошибки, схема не ломается)
- [ ] AC3: миграции на open (единый путь); `migrate()` не обязателен для хоста
- [ ] AC4: ошибки через `StorageResult` / `StorageError`; нет `unwrap()`/`expect()` в production-путях `src/`
- [ ] AC5: колонки не «улучшались» относительно freeze
- [ ] Global DoD: UI↛DB; glossary terms; out of scope T3/T4 не тронуты

## Risks / not covered
- Нет `ObservationRepository` (T3) — insert/get не проверялись.
- Полный integration suite repository (T4) — вне scope.
- Таблица `schema_migrations` — служебная (не в freeze §2); нужна для version gate. Schema change `observations` без ADR по-прежнему запрещён.

## Notes for QA
- Проверка колонок в тесте через `PRAGMA table_info(observations)`.
- `payload` объявлен как `JSON` (как в docs); SQLite хранит affinity по своим правилам.
