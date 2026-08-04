# Dev → QA: P1-E2-T3

## Meta
- **Task ID:** P1-E2-T3
- **Title:** `ObservationRepository`
- **Role that built:** Dev
- **Date:** 2026-08-03
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P1-E2 / P1-E2-T3; brief `docs/handoffs/P1-E2-T3-pm-brief.md`

## What changed
- Summary:
  - Добавлен `ObservationRepository` с API: `insert`, `get_by_id`, `list_by_time_range`, `list_by_data_type`.
  - `insert` — append-only; дубликат PK → `StorageError::DuplicateObservation` (не UPSERT).
  - `payload` сериализуется/десериализуется как JSON text; mapping в `bio_spec::Observation`.
  - `created_at` пишется при insert (Unix UTC seconds), отдельно от `Observation.timestamp`.
  - Typed errors расширены (`DuplicateObservation`, `InvalidTimeRange`, payload/id mapping, `Domain`).
  - Smoke-тесты: insert+get, duplicate PK, list by range/type.
- Crates / files:
  - `crates/storage/src/observation_repo.rs` (new)
  - `crates/storage/src/clock.rs` (new — shared unix clock)
  - `crates/storage/src/error.rs`
  - `crates/storage/src/lib.rs`
  - `crates/storage/src/migrate.rs` (uses shared clock)
  - `crates/storage/Cargo.toml` (`serde_json`, `uuid`)
  - `crates/storage/tests/observation_repo.rs` (new)

## How to verify (commands)
```bash
cargo test -p storage
```

Ожидание: все тесты зелёные (migrate + open + observation_repo).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: API `insert`, `get_by_id`, `list_by_time_range`, `list_by_data_type` публичны
- [ ] AC2: duplicate PK → явная `StorageError::DuplicateObservation`; строка не перезаписывается
- [ ] AC3: payload JSON round-trip в `Observation`
- [ ] AC4: `created_at` заполняется при insert и не равен/не путается с `timestamp`
- [ ] AC5: нет `unwrap()`/`expect()` в `crates/storage/src/`
- [ ] AC6: smoke-тесты insert+get и duplicate PK присутствуют и зелёные
- [ ] Global DoD: схема `observations` не менялась; UI/Tauri вне scope; полный suite → T4

## Risks / not covered
- Битый JSON / invalid confidence в БД / расширенные edge cases → **P1-E2-T4**.
- `list_by_time_range` при `end < start` → `InvalidTimeRange` (задокументировано; матрица в T4).
- Concurrent writers / WAL contention не тестировались.

## Notes for QA
- Репозиторий: `ObservationRepository::new(&db)` после `Database::open` / `open_in_memory`.
- `created_at` читается только SQL-ом в тесте; в доменной модели `Observation` поля нет — так и задумано.
