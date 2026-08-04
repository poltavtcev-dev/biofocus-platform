# Dev → QA: P1-E3-T1

## Meta
- **Task ID:** P1-E3-T1
- **Title:** Scaffold `apps/desktop` (Tauri v2 + React)
- **Role that built:** Dev
- **Date:** 2026-08-03
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P1-E3 / P1-E3-T1; brief `docs/handoffs/P1-E3-T1-pm-brief.md`

## What changed
- Summary:
  - Добавлен `apps/desktop`: Tauri v2 + React/TypeScript (Vite).
  - `src-tauri` — member workspace (`Cargo.toml`); зависит от `runtime` + `storage`.
  - Placeholder IPC `core_ping` (линкует Core без открытия SQLite / без Observation).
  - Минимальный UI shell (BioFocus + Status Idle/Error) + system tray tooltip `BioFocus`.
  - Production paths без `unwrap()`/`expect()`; старт через `Result` + `eprintln` + exit code.
  - Bundle target: macOS `.app` (без DMG — меньше хрупкости в CI/локали).
  - Обновлены root `README.md` и `docs/12-development.md` с командами desktop.
- Apps / crates / files:
  - `apps/desktop/**` (new)
  - `Cargo.toml` (workspace member `apps/desktop/src-tauri`)
  - `Cargo.lock` (Tauri + deps)
  - `README.md`, `docs/12-development.md`

## How to verify (commands)
```bash
# Core still green
cargo check
cargo test -p runtime -p storage

# Desktop
cd apps/desktop
pnpm install
pnpm build
pnpm tauri build          # → BioFocus.app
# Interactive (optional):
# pnpm tauri dev

# Boundary smoke (frontend must stay clean)
rg -n "rusqlite|biofocus_main\\.db|CREATE TABLE" apps/desktop/src apps/desktop/dist || true
```

Ожидание:
- `pnpm install` / `pnpm build` / `pnpm tauri build` — exit 0
- В окне: «BioFocus», Status Idle (после IPC), meta `runtime + storage`
- Tray с tooltip BioFocus
- В `src`/`dist` нет `rusqlite` / прямого SQL

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Структура `apps/desktop` с Tauri v2 + React/TS
- [ ] AC2: `pnpm install` успешен; `pnpm tauri build` или `dev` поднимает окно и/или tray
- [ ] AC3: `src-tauri` — workspace member; зависит от `runtime` и/или `storage` (compile-time / trivial call)
- [ ] AC4: Frontend не импортирует `rusqlite`, не открывает `biofocus_main.db`, не пишет SQL
- [ ] AC5: Нет `unwrap()`/`expect()` в production paths `apps/desktop/src-tauri/src/`
- [ ] AC6: Команды задокументированы (handoff + README / `docs/12-development.md`)
- [ ] Global DoD: UI↛DB; glossary; без Phase 2 ingest / dashboard

## Risks / not covered
- Финальный Menubar UX copy/states → **P1-E3-T2**
- Полноценный IPC `get_status` (version/db ok|error) → **P1-E3-T3** (`core_ping` — временный stub)
- DMG/`targets: all` не собираются намеренно (только `.app`); notarization/signing вне scope
- Tailwind из `docs/13-project-structure.md` не подключался (не в AC T1)
- `pnpm tauri dev` в headless/CI может требовать GUI session — локально проверен start бинаря (~3s)

## Notes for QA
- IPC команда: `core_ping` → `{ status, runtime, storage, dbFile, schemaVersion }` (camelCase).
- `dbFile` — только имя константы из `storage`, БД не открывается.
- Package: `biofocus-desktop` / identifier `com.biofocus.desktop` / productName `BioFocus`.
