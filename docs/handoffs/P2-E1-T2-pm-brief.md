# PM Brief → Dev: P2-E1-T2

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes 2026-08-04)  
**Date:** 2026-08-04  
**Closed previous:** P2-E1-T1 — QA Pass with notes (`docs/handoffs/P2-E1-T1-qa-to-pm.md`)  
**QA:** `docs/handoffs/P2-E1-T2-qa-to-pm.md`

## Task
**P2-E1-T2 — Pairing token persistence**

## Why
Skeleton использует env/const token. Для companion нужен стабильный локальный секрет без git и без хардкода в production path.

## Acceptance Criteria
1. При первом старте ingest/host генерируется pairing token (криптостойкий), сохраняется под `~/.biofocus/` (путь задокументировать).
2. Повторный старт читает тот же token; ingest принимает только его (Bearer).
3. Неверный / отсутствующий Bearer → **401** (как T1).
4. Секрет **не** коммитится; `.gitignore` покрывает файл/директорию при необходимости.
5. Docs: `docs/10-security.md` и/или `docs/12-development.md` — где лежит token, как задать override (`BIOFOCUS_INGEST_TOKEN` если остаётся).
6. Тесты: generate→persist→reload; reject wrong token (temp dir).
7. Handoff: `docs/handoffs/P2-E1-T2-dev-to-qa.md`.

## Out of scope
- QR / pairing UX UI → **P2-E3-T2**
- Persist Observations → **T3**
- Host wire in Tauri → **T4**
- Mid-batch 503 contract → **T3** (carry note from T1)

## Constraints
- Loopback-only bind остаётся (T1)
- No `unwrap`/`expect` in production
- Branch → PR → `main` after CI
- Prefer continuing `phase/2-ingest-http` if T1 not yet merged

## After QA Pass
PM → Ready **P2-E1-T3** (persist + mid-batch backpressure contract).
