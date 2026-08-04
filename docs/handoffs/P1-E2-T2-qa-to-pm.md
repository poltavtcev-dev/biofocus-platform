# QA → PM: P1-E2-T2

## Meta
- **Task ID:** P1-E2-T2
- **Title:** Migration: `observations` table
- **Date:** 2026-08-03
- **Dev/UX handoff:** `docs/handoffs/P1-E2-T2-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p storage` → **ok** (5 tests: 2 migrate + 3 open; 0 failed)
- AC results (pass/fail per item):
  - **AC1 Pass** — SQL в `migrate.rs` совпадает с freeze `/docs/04-storage.md` §2: колонки `id TEXT PK`, `timestamp INTEGER`, `provider_id TEXT`, `data_type TEXT`, `payload JSON`, `confidence REAL DEFAULT 1.0`, `created_at INTEGER`; индексы `idx_obs_ts`, `idx_obs_type_ts`. Тест `open_creates_observations_table_and_indexes` подтверждает через `PRAGMA table_info` + `sqlite_master`.
  - **AC2 Pass** — `migrate_twice_is_idempotent`: повторный `migrate()` + reopen без ошибок, схема стабильна; version gate + `IF NOT EXISTS`.
  - **AC3 Pass** — единый путь: `Database::open` / `open_in_memory` → pragmas → `migrate::run`. Публичный `Database::migrate()` — идемпотентный re-apply (для тестов).
  - **AC4 Pass** — ошибки через `StorageResult` / `StorageError` (в т.ч. `Clock`); в `crates/storage/src/` нет `unwrap()`/`expect()`.
  - **AC5 Pass** — колонки не «улучшены» относительно freeze.
  - **Global DoD Pass** — UI↛DB соблюдён (только `storage`); glossary; `ObservationRepository` / T4 не реализованы (out of scope).
- Extra checks (edge / security):
  - WAL/open regression из T1 зелёная после migrate-on-open.
  - Служебная `schema_migrations` вне freeze §2 — допустима для version gate; схема `observations` не менялась.
  - Нет UI→DB; нет логирования payload/биометрии в migrate path (только `version` в `tracing::info`).

## Defects (if any)
- Нет.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E2-T2** to Done; Ready → **P1-E2-T3**; refresh Next
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` — `cargo test -p storage`

## Suggested next Ready task
- **P1-E2-T3** — `ObservationRepository` (`insert` / `get_by_id` / list; duplicate PK → явная ошибка)

## Notes for PM
- Scope T2 закрыт полностью; repository и полный integration suite — следующие задачи.
- Handoff Dev корректен; дополнительных тестов от QA не потребовалось.
