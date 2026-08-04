# Dev → QA: P1-E3-T3

## Meta
- **Task ID:** P1-E3-T3
- **Title:** IPC: `get_status` command
- **Role that built:** Dev
- **Date:** 2026-08-03
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P1-E3 / P1-E3-T3; brief `docs/handoffs/P1-E3-T3-pm-brief.md`

## What changed
- Summary:
  - Добавлена Tauri-команда **`get_status`**: `version` + `dbStatus` `ok|error` (+ optional `dbError`).
  - Реальный probe: `storage::default_db_path()` → `Database::open` (WAL + migrate-on-open); ошибки → soft `error`, без panic.
  - UI (T2 `coreStatus.ts`) предпочитает `get_status`; `core_ping` оставлен как fallback.
  - Tray wire T2 сохранён: id `main`, template icon, click → show window, tooltip обновляет frontend.
  - Контракт: комментарии в `src-tauri/src/lib.rs` + `docs/09-api.md` § Desktop Tauri IPC.
  - Unit-тесты desktop: mapping ok/error, temp DB probe, JSON без Observation-полей.
- Files:
  - `apps/desktop/src-tauri/src/lib.rs`
  - `apps/desktop/src/coreStatus.ts` (consume `dbError` in meta)
  - `apps/desktop/README.md` (note)
  - `docs/09-api.md`
  - `docs/handoffs/P1-E3-T3-dev-to-qa.md` (этот файл)

## How to verify (commands)
```bash
cargo test -p desktop
cargo check -p desktop

cd apps/desktop
pnpm install   # если нужно
pnpm build
# optional GUI:
# pnpm tauri dev
# Ожидание: Status Ready, meta v0.1.0, tray "BioFocus — Ready"
# (первый запуск создаёт ~/.biofocus/data/biofocus_main.db)

# Boundary
rg -n "rusqlite|CREATE TABLE" apps/desktop/src apps/desktop/dist || true
# NOTE: legacy core_ping payload still mentions dbFile name in Rust; frontend
# must not open that path. Prefer checking frontend does not call SQLite APIs.
```

### Example JSON (`get_status`)
Success:
```json
{ "version": "0.1.0", "dbStatus": "ok" }
```
Failure:
```json
{
  "version": "0.1.0",
  "dbStatus": "error",
  "dbError": "cannot resolve home directory for default BioFocus data path"
}
```

### How to invoke
```ts
import { invoke } from "@tauri-apps/api/core";
const status = await invoke("get_status");
// { version, dbStatus, dbError? }
```

UI path: `apps/desktop/src/coreStatus.ts` → `fetchCoreStatus()` → `invoke("get_status")`.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Команда `get_status` возвращает version + `dbStatus` `ok|error` (error reason optional)
- [ ] AC2: В ответе нет raw Observation / biometric payload
- [ ] AC3: UI обновляет статус только через IPC; контракт в коде и/или `docs/09-api.md`
- [ ] AC4: Реальный DB probe через `storage`; soft-fail; нет `unwrap`/`expect` в production paths
- [ ] AC5: Frontend UI↛DB
- [ ] AC6: Handoff с example JSON + invoke
- [ ] Global DoD: glossary; без HTTP `/v1/status` (Phase 2); Menubar polish → T2 (уже в parallel)

## Risks / not covered
- Первый успешный `get_status` создаёт `~/.biofocus/data/biofocus_main.db` (ожидаемо Local-First).
- Forced mock error UI: `?mockStatus=error` (T2) — без порчи реальной DB.
- Полная smoke-матрица tray/mock → **T4**.
- `core_ping` ещё зарегистрирован (T2 fallback); удаление — follow-up после T2/T4.

## Notes for QA
- Параллельная **T2** уже consumer `get_status`; Dev не заменял UX shell — только IPC + тонкий wire в `coreStatus.ts`.
- Unit-тесты используют `expect` только внутри `#[cfg(test)]`.
- `docs/09-api.md` нормализован (раньше был nested markdown fence) + IPC section.
