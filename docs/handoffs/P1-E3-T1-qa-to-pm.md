# QA → PM: P1-E3-T1

## Meta
- **Task ID:** P1-E3-T1
- **Title:** Scaffold `apps/desktop` (Tauri v2 + React)
- **Date:** 2026-08-03
- **Dev/UX handoff:** `docs/handoffs/P1-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo check` → **ok**
  - `cargo test -p runtime -p storage` → **ok** (5 + 17 tests)
  - `cd apps/desktop && CI=true pnpm install` → **ok** (pnpm 9.15.9, 72 packages)
  - `pnpm build` → **ok** (vite production bundle)
  - `pnpm tauri build` → **ok** → `BioFocus.app` (bundle target `app`)
  - Smoke-start `BioFocus.app` binary ~3s → process stays alive, empty stderr, clean stop
  - `rg -n "rusqlite|biofocus_main\\.db|CREATE TABLE" apps/desktop/src apps/desktop/dist` → **no matches**
- AC results (pass/fail per item):
  - **AC1 Pass** — `apps/desktop` = Tauri v2 + React/TS (Vite); `package.json` `biofocus-desktop`, `@tauri-apps/*` v2.
  - **AC2 Pass** — install/build/tauri build exit 0; `.app` produced; binary starts without immediate crash; tray wired in `lib.rs` (`tooltip("BioFocus")`) + window title `BioFocus`.
  - **AC3 Pass** — `apps/desktop/src-tauri` in root workspace members; deps `runtime` + `storage`; `core_ping` calls `runtime::observation_channel` and reads `storage::{CRATE_NAME, DEFAULT_DB_FILE_NAME, SCHEMA_VERSION}` (без open SQLite).
  - **AC4 Pass** — frontend только `invoke("core_ping")`; в `src`/`dist` нет `rusqlite` / `biofocus_main.db` / SQL.
  - **AC5 Pass** — в `apps/desktop/src-tauri/src/` нет `unwrap()`/`expect()`; `main` → `Result` + `eprintln` + exit 1.
  - **AC6 Pass** — команды в Dev handoff, root `README.md`, `docs/12-development.md`.
  - **Global DoD Pass** — UI↛DB; без Phase 2 ingest / dashboard; `get_status` / Menubar polish осознанно out of scope (T2/T3).
- Extra checks (edge / security):
  - IPC stub не открывает БД и не отдаёт Observation.
  - `dbFile` в payload — только константа имени файла из Core; frontend её не хардкодит и не рендерит.

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E3-T1** to Done; Ready → **P1-E3-T2** (UX) ∥ **P1-E3-T3** (Dev)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` updated; briefs T2/T3 issued

## Suggested next Ready task
- **P1-E3-T2** — Menubar status UX (minimal), Role: UX *(после T1)*  
- **P1-E3-T3** — IPC `get_status`, Role: Dev *(можно ∥ T2 после T1)*

## Notes for PM
- Визуальный клик по tray/окну в GUI session не делался; AC scaffold закрыт через `tauri build` + smoke-start бинаря + code review tray setup.
- Bundle намеренно только `.app` (без DMG) — ок для T1; notarization вне scope.
- Tailwind из `docs/13-project-structure.md` не подключался — не в AC T1.
