# QA → PM: P1-E3-T3

## Meta
- **Task ID:** P1-E3-T3
- **Title:** IPC: `get_status` command
- **Date:** 2026-08-03
- **Dev/UX handoff:** `docs/handoffs/P1-E3-T3-dev-to-qa.md`
- **Verdict:** Pass
- **Re-verified:** 2026-08-03 (повтор по `P1-E3-T3-dev-to-qa.md`) — тот же Pass, регрессий нет

## What was verified
- Commands run + results:
  - `cargo test -p desktop` → **ok** (4 tests: ok/error mapping, temp probe, JSON без Observation-полей) — re-run ok
  - `cargo check -p desktop` → **ok**
  - `cd apps/desktop && pnpm build` → **ok**
  - `rg -n "rusqlite|CREATE TABLE" apps/desktop/src apps/desktop/dist` → **no matches**
  - `rg -n "biofocus_main\\.db" apps/desktop/src apps/desktop/dist` → **no matches**
  - `unwrap`/`expect` только в `#[cfg(test)]` (строки 202–203) — production path чист
- AC results (pass/fail per item):
  - **AC1 Pass** — `get_status` → camelCase `{ version, dbStatus: "ok"|"error", dbError? }` via `build_status` + `probe_default_database`.
  - **AC2 Pass** — `CoreStatus` только version/db_status/db_error; unit `status_json_has_no_observation_fields` подтверждает отсутствие observations/payload/hrv.
  - **AC3 Pass** — UI consumer `coreStatus.ts` → `invoke("get_status")` (fallback `core_ping`); контракт в `lib.rs` module docs + `docs/09-api.md` § Desktop Tauri IPC.
  - **AC4 Pass** — probe `storage::default_db_path` + `Database::open`; soft-fail в `dbStatus: "error"`; production path без `unwrap`/`expect` (есть только в `#[cfg(test)]`).
  - **AC5 Pass** — frontend UI↛DB (boundary rg clean).
  - **AC6 Pass** — handoff содержит example JSON + invoke snippet.
  - **Global DoD Pass** — desktop IPC only; нет HTTP `/v1/status`; tray wire T2 сохранён.
- Extra checks (edge / security):
  - Нет Observation/biometric в IPC payload types.
  - `core_ping` остаётся зарегистрированным как documented fallback — ок для T2/T4.
  - Первый успешный probe создаёт local DB path — ожидаемо Local-First (задокументировано Dev).

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E3-T3** to Done; **P1-E3-T4** assigned
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` status; T4 brief issued

## Suggested next Ready task
- **P1-E3-T4** — Smoke: UI ↔ Core boundary (QA lead; depends T2+T3)  
  Note: T2 QA Pass already filed (`docs/handoffs/P1-E3-T2-qa-to-pm.md`)

## Notes for PM
- Soft note → T4: `dbError` = `err.to_string()` из storage; при `CreateDir` и подобных может содержать абсолютный path. Не блокер AC T3 («short error reason optional»), но в smoke стоит проверить, что UI не выглядит как «открыли путь к DB», и при желании санитизировать на host.
- GUI `pnpm tauri dev` Ready/tray не крутился в этой сессии; покрытие — unit + build + docs/code contract.
