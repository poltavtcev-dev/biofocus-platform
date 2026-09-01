# PM Brief → Dev|UX: P28-E5-T1 (optional)

**From:** PM  
**To:** Dev + UX  
**Status:** Queued (optional; after P28-E1, parallel ok)  
**Date:** 2026-08-31  
**Phase:** Phase 28 — Sync health / first-week trust — **ADR-029**  
**Branch:** `phase/28-local-reliability`

## Task
**P28-E5-T1 — Calm sync health surface (Desktop + iOS)**

## Why
Продукт должен **приносить пользу** — юзер на 1-й неделе должен видеть: данные доезжают, Mac доступен, очередь не молчит. Без cloud dashboard — только local status.

## Acceptance Criteria
1. **iOS:** pending count, last flush time, last error (calm), auto-sync on/off — уже частично есть; дополнить «Mac reachable» после preflight (E1).
2. **Desktop Companion section:** ingest up/down, bind mode, last ingest timestamp (no biometrics in payload).
3. Copy спокойный: «Waiting for Mac», «Last sync 2h ago», не «sync failed you broke it».
4. No busy-loop status poll — refresh on mount + manual reload + reasonable interval (≥30s).
5. Handoff: `docs/handoffs/P28-E5-T1-dev-to-qa.md`.

## Out of scope
- Push notifications; weekly email; cloud status page

## Next chat
```
как агент: режим build-qa для P28-E5-T1.
Brief: docs/handoffs/P28-E5-T1-pm-brief.md
…
```
