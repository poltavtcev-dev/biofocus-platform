# PM Brief → Dev: P28-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Queued (after P28-E1-T1 Pass)  
**Date:** 2026-08-31  
**Phase:** Phase 28 — Local device reliability **(B)** — **ADR-029**  
**Branch:** `phase/28-local-reliability`

## Task
**P28-E2-T1 — iPhone buffer hardening: retry flush, reachability, durability**

## Why
ADR-029 pillar **(B):** часы → HealthKit → **очередь на телефоне** → flush когда Mac доступен. Сейчас очередь есть, но flush хрупкий: один fail → данные висят; cap 200 без calm policy; нет retry при появлении сети.

## Acceptance Criteria
1. **Retry/backoff** на failed flush (exponential cap, e.g. max 5 min); очередь **не** очищается до успешного POST.
2. **Reachability trigger:** при foreground / network available / HK observer — попытка flush если есть pending + valid base URL + token.
3. **Queue durability:** сохранить dedupe; при cap overflow — calm drop oldest with counter surfaced in status (не silent loss).
4. **Anchors:** HK anchors обновляются только после successful enqueue path (не терять прогресс на failed flush).
5. Tests: Swift unit tests for retry policy / queue cap; existing `cargo test -p companion` green.
6. Handoff: `docs/handoffs/P28-E2-T1-dev-to-qa.md`.

## Out of scope
- Cloud relay; Mac Launch Agent (P28-E3); Feature replay (P28-E4)
- SQLite schema; ingest auth changes

## Constraints
- ADR-005 LAN opt-in; Bearer on POST
- No busy-loop poll — event + reachability driven
- No clinical copy

## Next chat (после P28-E1 Pass)
```
как агент: режим build-qa для P28-E2-T1.
Brief: docs/handoffs/P28-E2-T1-pm-brief.md
1) Как Dev — собери по AC, создай docs/handoffs/P28-E2-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P28-E2-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
