# QA → PM: P1-E3-T2

## Meta
- **Task ID:** P1-E3-T2
- **Title:** Menubar status UX (minimal)
- **Date:** 2026-08-03
- **Dev/UX handoff:** `docs/handoffs/P1-E3-T2-dev-to-qa.md` (Role: UX)
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cd apps/desktop && CI=true pnpm install` → **ok**
  - `pnpm build` → **ok** (tsc + vite)
  - `cargo check -p desktop` → **ok**
  - `rg -n "rusqlite|biofocus_main\\.db|CREATE TABLE" apps/desktop/src apps/desktop/dist` → **no matches**
  - Contract smoke (mock query + tooltip + banned copy) → **ok**
  - Code review: `App.tsx`, `coreStatus.ts`, `App.css`, tray wire in `src-tauri/src/lib.rs`, window `300×168` / label `main`
- AC results (pass/fail per item):
  - **AC1 Pass** — Neutral states **Idle** / **Ready** / **Error**; copy: “Waiting for Core.” / “Core is available.” / “Could not reach Core.”; no judgmental phrasing.
  - **AC2 Pass** — Compact shell (brand + status dot + detail + optional meta; Error → “Try again”); no charts / dashboard widgets / Phase 4 clutter; light/dark via `prefers-color-scheme`.
  - **AC3 Pass** — Status via IPC only (`fetchCoreStatus`): prefers `get_status`, fallback `core_ping`; QA mock `?mockStatus=idle|ready|error` bypasses Core/DB. Tray tooltip `BioFocus — {label}`; left-click shows/focuses window `main`.
  - **AC4 Pass** — UI↛DB: no rusqlite / db file / SQL in `src`/`dist`; status path is `invoke` only.
  - **AC5 Pass** — Handoff with states checklist present.
  - **Global DoD Pass** — Phase 1 menubar shell only; Observation not over UI.
- Extra checks (edge / security):
  - Retry only on Error; busy disables button.
  - `unwrap`/`expect` in `src-tauri` only under `#[cfg(test)]` (T3 unit tests) — ok for production path.
  - Tree already contains `get_status` (T3 in progress/parallel) — UI correctly prefers it; not a T2 defect.

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E3-T2** to Done; **P1-E3-T3** kept open → QA
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` status line

## Suggested next Ready task
- Finish/verify **P1-E3-T3** (IPC `get_status`) if not yet QA’d → then **P1-E3-T4** boundary smoke (needs T2+T3)

## Notes for PM
- Интерактивный GUI hover/click tray не прогонялся в этой сессии; AC закрыты build + boundary + code/state contract (+ mock helper).
- Note for T3/T4: при `dbStatus: error` UI может показать `dbError` в meta — убедиться, что сообщение не тащит абсолютные пути к DB в пользовательский текст (sanitization на host).
- Mock `?mockStatus=` оставлен для T4 — ок.
