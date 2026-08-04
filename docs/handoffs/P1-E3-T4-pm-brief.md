# PM Brief → QA: P1-E3-T4

**From:** PM  
**To:** QA (lead)  
**Status:** In Progress (assigned)  
**Date:** 2026-08-03  
**Closed previous:** P1-E3-T3 — QA Pass (`docs/handoffs/P1-E3-T3-qa-to-pm.md`); T2 already Done

## Task
**P1-E3-T4 — Smoke: UI ↔ Core boundary**

## Modules
`apps/desktop` (frontend + `src-tauri` boundary)

## Depends on
P1-E3-T2 ✅ · P1-E3-T3 ✅

## Acceptance Criteria
1. Menubar/tray + minimal window: статус виден (Idle / Ready / Error).
2. Статус меняется при mock error (`?mockStatus=error`) и возвращается / retry path понятен.
3. В `apps/desktop/src` и production `dist` **нет** `rusqlite`, `CREATE TABLE`, хардкода пути `biofocus_main.db`.
4. Live path: `get_status` (или fallback) — UI не получает Observation / biometric fields.
5. Soft check: если `dbError` показан в UI meta — не выглядит как «утечка» абсолютного пути к домашней DB (зафиксировать Pass with notes, если path виден; не Fail AC T3, но отметить для E4/ADR).
6. `cargo test -p desktop` + `pnpm build` зелёные; при возможности короткий smoke `tauri`/binary.

## Process
QA = исполнитель. По завершении сразу `docs/handoffs/P1-E3-T4-qa-to-pm.md`.

## Out of scope
- Phase 2 ingest HTTP
- Dashboard / features
- Notarization / DMG

## After QA Pass
PM → Epic E3 Done → **P1-E4-T1** Phase 1 exit checklist.
