# PM Brief → UX + Dev: P3-E3-T3

**From:** PM  
**To:** UX + Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E3-T2 (QA Pass; IPC `alertLevel`)

## Task
**P3-E3-T3 — Menubar traffic-light UX**

## Acceptance Criteria
1. Shell / Menubar reflects alert level from IPC (`green` / `yellow` / `red`) via indicator color.
2. Tray tooltip includes calm alert wording (non-evaluative; no “you burned out”).
3. Idle/Ready/Error core status still works; no charts/dashboard.
4. Handoff with manual smoke steps (`?mockAlert=…` OK for QA).

## Out of scope
- Wearable bridges / LAN ingest
- Changing Core alert rules
- Dashboard

## Constraints
- Data only via IPC `get_status`
- Branch: `phase/3-pipeline-features`
