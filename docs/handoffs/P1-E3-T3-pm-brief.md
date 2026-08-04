# PM Brief → Dev: P1-E3-T3

**From:** PM  
**To:** Dev  
**Status:** In Progress (assigned, parallel with T2)  
**Date:** 2026-08-03  
**Closed previous:** P1-E3-T1 — QA Pass (`docs/handoffs/P1-E3-T1-qa-to-pm.md`)

## Task
**P1-E3-T3 — IPC: `get_status` command**

## Modules
`apps/desktop/src-tauri`, frontend invoke consumer; may use `runtime` / `storage` (open DB status OK — **no** Observation payloads)

## Depends on
P1-E3-T1, P1-E1-T3. Parallel with T2.

## Acceptance Criteria
1. Tauri command `get_status` (name exact or documented alias) returns at least: app/core **version**, **db_status** `ok|error` (and short error reason optional).
2. Response contains **no** raw `Observation` / biometric payload.
3. UI (or stub consumer) updates status **only** via IPC — document contract briefly in code comments and/or `docs/09-api.md` (Internal IPC note).
4. Prefer real check: open/default DB path or migrate-on-open probe via `storage` — fail soft to `error` without panic; no `unwrap()`/`expect()` in production paths.
5. Frontend still UI↛DB.
6. Handoff: `docs/handoffs/P1-E3-T3-dev-to-qa.md` with example JSON response + how to invoke.

## Out of scope
- Menubar visual polish → **T2**
- Full QA smoke matrix → **T4**
- HTTP `/v1/status` public ingest API (Phase 2) — desktop IPC only

## Stop
«Передаю QA» + handoff. Не Done в roadmap/canvas.
