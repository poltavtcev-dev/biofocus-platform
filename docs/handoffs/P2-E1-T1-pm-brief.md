# PM Brief → Dev: P2-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes 2026-08-04)  
**Date:** 2026-08-04  
**Closed previous:** Phase 1 / P1-E4-T2  
**QA:** `docs/handoffs/P2-E1-T1-qa-to-pm.md`

## Task
**P2-E1-T1 — Local ingest HTTP skeleton**

## Why
Без двери `/v1/ingest` нет потока Observation с companion/collectors. Сначала скелет HTTP + auth + queue response; persist — T3.

## Acceptance Criteria
1. Axum (или согласованный с `docs/03-runtime.md` HTTP stack) слушает **только** `127.0.0.1:<port>` (порт константа/config, задокументировать в handoff).
2. `POST /v1/ingest` + header `Authorization: Bearer <token>`:
   - body = JSON array `Observation` (`bio-spec`);
   - success → **202** + `{"status":"queued","count":N}` (пока in-memory / channel placeholder — **не** обязаны писать SQLite в T1);
   - missing/wrong token → **401**;
   - invalid JSON / domain validation fail → **400**.
3. Нет busy-loop: после bind процесс в idle ждёт I/O (tokio).
4. Тесты: auth reject, 202 happy path, bad JSON (без Tauri).
5. Handoff: `docs/handoffs/P2-E1-T1-dev-to-qa.md`.

## Out of scope
- Persist to SQLite (→ **P2-E1-T3**)
- Pairing file / QR (→ **T2** / E3)
- LAN bind beyond loopback
- macOS collector, iOS, Feature pipeline
- Architecture ADR changes (stack Axum уже в docs)

## Constraints
- Production: no `unwrap`/`expect`
- PR workflow: branch `phase/2-ingest-http` → PR → `main` after CI
- Idle footprint — first-class AC

## After QA Pass
PM → Ready **P2-E1-T2** (pairing token persistence).
