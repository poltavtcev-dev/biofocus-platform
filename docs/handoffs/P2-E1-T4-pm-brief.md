# PM Brief → Dev: P2-E1-T4

**From:** PM  
**To:** Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E1-T3 — QA Pass with notes (`docs/handoffs/P2-E1-T3-qa-to-pm.md`)

## Task
**P2-E1-T4 — Host wire + `GET /v1/status` + idle DoD**

## Why
Ingest + persist exist as a crate, but Desktop does not start them. Companion/debug need local HTTP status; app lifecycle must own bind/shutdown without busy-loop.

## Acceptance Criteria
1. Desktop (`apps/desktop/src-tauri`) on startup:
   - opens default DB (WAL + migrate);
   - `IngestConfig::load()` (pairing file / env from T2);
   - `runtime::observation_channel` → `ingest::spawn_persist_worker(rx, db)` → serve ingest with `tx`;
   - loopback bind only (`127.0.0.1`, default port 8787 unless config).
2. Clean shutdown stops accept + worker (no orphan spin).
3. `GET /v1/status` on ingest server returns version + db ok|error **without** Observation / biometric payload.
4. Idle: no busy-loop after bind (tokio accept + blocking_recv worker).
5. Shell UI still uses IPC `get_status` for menubar; HTTP status is for companion/debug only.
6. Tests: at least unit/integration for status JSON shape + soft db error mapping where practical without flaky full-app launch; document manual smoke if needed.
7. Handoff: `docs/handoffs/P2-E1-T4-dev-to-qa.md`.

## Out of scope
- QR / pairing UX → **P2-E3-T2**
- macOS collector / iOS
- Feature pipeline
- Changing mid-batch contract C or duplicate-at-persist semantics (locked in T3)
- Sprint PR to `main` (commit on branch OK; PR after E1 or sprint end)

## Constraints
- No `unwrap`/`expect` in production
- UI ↛ SQLite
- Prefer soft-fail db problems into status `error` (same spirit as IPC `get_status`)

## After QA Pass
PM → Epic **P2-E1** Done → Ready **P2-E2-T1** (and optionally PR batch for E1).
