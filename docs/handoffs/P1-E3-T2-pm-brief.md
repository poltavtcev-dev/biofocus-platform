# PM Brief → UX: P1-E3-T2

**From:** PM  
**To:** UX (lead); Dev — minimal wire if needed  
**Status:** In Progress (assigned, parallel with T3)  
**Date:** 2026-08-03  
**Closed previous:** P1-E3-T1 — QA Pass (`docs/handoffs/P1-E3-T1-qa-to-pm.md`)

## Task
**P1-E3-T2 — Menubar status UX (minimal)**

## Modules
`apps/desktop/src/**` (tray/window UI); optional thin wire in `src-tauri` only if required for tray labels

## Depends on
P1-E3-T1 Done. May proceed in parallel with T3 (mock IPC until `get_status` ready).

## Acceptance Criteria
1. Menubar/tray (and/or minimal window) shows **neutral** Core status: e.g. Idle / Ready / Error (labels calm, non-judgmental — no «ты выгорел»).
2. States readable per Apple HIG; no charts, no dashboard widgets, no Phase 4 clutter.
3. Status updates driven by IPC invoke (mock OK until T3; prefer integrating `get_status` if T3 already merged).
4. UI still UI↛DB — no rusqlite / db paths in frontend.
5. Handoff: `docs/handoffs/P1-E3-T2-dev-to-qa.md` (Role: UX) with screenshots/states checklist for QA.

## Out of scope
- Full IPC contract design → **T3**
- Boundary smoke suite → **T4**
- Dashboard / metrics graphs

## Stop
«Передаю QA» + handoff. Не Done в roadmap/canvas.
