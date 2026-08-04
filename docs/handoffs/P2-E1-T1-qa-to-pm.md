# QA → PM: P2-E1-T1

## Meta
- **Task ID:** P2-E1-T1
- **Title:** Local ingest HTTP skeleton (`POST /v1/ingest`)
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E1-T1-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/2-ingest-http` (изменения на ветке; commit/PR — у Dev/PM по workflow Phase 2)

## What was verified
- Commands run + results:
  - `cargo check --workspace --exclude desktop` → **ok**
  - `cargo test -p bio-spec -p runtime -p storage -p ingest` → **ok**
    - ingest: 3 unit (auth) + 8 integration (bind / 401×2 / 400 JSON / 400 domain / 202 enqueue / live loopback HTTP / default config)
    - regression bio-spec / runtime / storage — без регрессий
- AC results (pass/fail per item):
  - **AC1 Pass** — `bind_loopback` → `SocketAddrV4(INGEST_BIND_HOST=127.0.0.1, port)`; тест `bind_loopback_is_localhost_only`; default port `8787`
  - **AC2 Pass** — Bearer → 202 `{"status":"queued","count":N}`; missing/wrong → 401; invalid JSON / confidence domain → 400; enqueue в `runtime::observation_channel` (без SQLite)
  - **AC3 Pass** — `axum::serve` / tokio accept; нет spin-loop в production-пути
  - **AC4 Pass** — тесты auth reject, 202, bad JSON (+ domain + live HTTP); без Tauri
  - **AC5 Pass** — нет `storage`/SQLite dep; нет Feature/pipeline; нет UI
  - **Global DoD Pass** — в `crates/ingest/src/` нет `unwrap`/`expect` (только tests); glossary `Observation`; UI↛DB
- Extra checks (edge / security):
  - Loopback-only: нет `0.0.0.0` / LAN bind
  - CI: `.github/workflows/ci.yml` включает `-p ingest`
  - Constant-time-ish token compare в skeleton (harden → T2)
  - Логи: rejection/channel full — без payload Observation
  - `503 queue_full` / `queue_closed` реализованы как скелет (формализация → T3)

## Defects (if any)
- Нет блокеров AC.
- **Note (known, out of T1):** при `503 queue_full` mid-batch часть Observation уже может быть в канале (частичный enqueue). Зафиксировать поведение в **P2-E1-T3**.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P2-E1-T1 → Done; Ready → **P2-E1-T2** (pairing token persistence)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs (optional): `ARCHITECTURE_STATUS.md` (Phase 2 ingest skeleton landed); при необходимости упоминание порта `8787` / crate `ingest` в `docs/12-development.md` или `docs/09-api.md`
- [ ] Напомнить Dev: commit + PR `phase/2-ingest-http` → `main` (Phase 2 git workflow), если ещё не запушено

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T1 Done; Ready T2; T3 AC + mid-batch note
- [x] ARCHITECTURE_STATUS, 12-development, 09-api
- [x] Brief `docs/handoffs/P2-E1-T2-pm-brief.md`
- [x] Execution canvas
- [ ] Dev: commit + PR for `crates/ingest` on `phase/2-ingest-http` (still required)

## Suggested next Ready task
- **P2-E1-T2** — Pairing token persistence (depends on P2-E1-T1)

## Notes for PM
- Публичные константы: `DEFAULT_INGEST_PORT=8787`, `DEFAULT_SKELETON_TOKEN`, `BIOFOCUS_INGEST_TOKEN`, `INGEST_BIND_HOST`.
- API: `bind_loopback`, `serve_listener`, `serve_with_shutdown`, `ingest_router`, `IngestConfig::from_env`.
- Out of scope подтверждён: persist, QR/LAN, collector, Tauri host wire (T2–T4).
