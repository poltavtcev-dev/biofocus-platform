# PM Brief → Dev: P3-E3-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E3-T1 (QA Pass; `map_alert_level`)

## Task
**P3-E3-T2 — IPC expose alert level**

## Why
Core already maps Features/Signals → `AlertLevel`. UI/Menubar (T3) must read it only via Tauri IPC — never from SQLite or biometric payloads.

## Acceptance Criteria
1. `get_status` (or adjacent IPC) returns alert level (`green` / `yellow` / `red`) without breaking Idle/Ready/Error (`dbStatus` / existing fields).
2. UI obtains alert level **only** through IPC (TypeScript may parse the field; traffic-light UX → T3).
3. Payload: no absolute paths, no Observation/biometric fields.
4. Host-side test(s) for status JSON / mapping wiring if appropriate.
5. Handoff: `docs/handoffs/P3-E3-T2-dev-to-qa.md`.

## Suggested approach
- Extend `CoreStatus` with `alertLevel`.
- Default **green** when Feature Worker / engine has no evidence.
- Prefer live update via Feature Worker hook → `register_catalog_v1` → `map_alert_level` → shared state read by `get_status`.

## Out of scope
- Menubar 🟢/🟡/🔴 colors / copy (→ **P3-E3-T3**)
- Mi Band 8 / LAN ingest
- Persisting Features/Signals / alert history

## Constraints
- Production: no `unwrap` / `expect`
- Branch: `phase/3-pipeline-features`
- Idle-safe Feature Worker (no busy-loop)

## After QA Pass
PM → Ready **P3-E3-T3** (Menubar traffic-light UX).
