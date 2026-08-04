# PM Brief → Dev: P2-E0-T1

**From:** PM  
**To:** Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E3-T2 — QA Pass with notes (`docs/handoffs/P2-E3-T2-qa-to-pm.md`); Epic E3 ✅

## Task
**P2-E0-T1 — Sanitize IPC `dbError` paths**

## Why
`get_status` may surface absolute filesystem paths in `dbError`, leaking host layout into the UI. Close Phase 2 hygiene before sprint gate.

## Acceptance Criteria
1. UI / `get_status` does not show absolute filesystem paths in `dbError`.
2. Message is short and safe.
3. Test(s) cover the mapping.
4. Optional: also sanitize HTTP `/v1/status` `db_error` the same way.
5. Handoff: `docs/handoffs/P2-E0-T1-dev-to-qa.md`.

## Out of scope
- Changing Observation / ingest contracts
- Phase 3 Features / dashboard
- LAN bind for physical phone

## Hygiene
- Commit after build; push only at sprint gate (`SPRINT-GATE.md`).
