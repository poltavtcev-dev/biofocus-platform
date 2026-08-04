# PM Brief → Dev: P1-E3-T1

**From:** PM  
**To:** Dev  
**Status:** In Progress (assigned)  
**Date:** 2026-08-03  
**Closed previous:** Epic E2 complete — P1-E2-T4 QA Pass (`docs/handoffs/P1-E2-T4-qa-to-pm.md`)

## Task
**P1-E3-T1 — Scaffold `apps/desktop` (Tauri v2 + React)**

## Modules
`apps/desktop`, `apps/desktop/src-tauri`  
Workspace crates: wire `runtime` and/or `storage` as dependencies (stub usage OK)

## Depends on
P1-E1-T1, P1-E1-T3 (runtime), Epic E2 Done (storage available for later IPC)

## Acceptance Criteria
1. Структура `apps/desktop` с Tauri v2 + React/TS (по `/docs/13-project-structure.md` / `docs/12-development.md`).
2. `pnpm install` в desktop app успешен; `pnpm tauri dev` или `build` поднимает окно и/или tray shell (минимальный UI допустим).
3. `src-tauri` — member/consumer workspace: зависит от `runtime` и/или `storage` (хотя бы compile-time link / trivial call).
4. **Граница:** frontend **не** импортирует `rusqlite`, не открывает `biofocus_main.db`, не пишет SQL. Только будущий IPC.
5. Ошибки в Rust backend — без `unwrap()`/`expect()` в production paths.
6. Кратко обновить команды в handoff (и при необходимости path в root README / `docs/12-development.md` — можно оставить PM на закрытии).

## Out of scope
- Финальный Menubar UX copy/states → **P1-E3-T2** (UX)
- Полноценный IPC `get_status` → **P1-E3-T3** (placeholder command OK)
- Dashboard / Recharts / LLM
- HTTP ingest (Phase 2)

## DoD for Dev step
- Приложение собирается/запускается в dev
- `docs/handoffs/P1-E3-T1-dev-to-qa.md`
- «Передаю QA» — без Done в roadmap/canvas

## After Dev
`как QA: проверь P1-E3-T1`
