# PM Brief → Dev: P28-E4-T1

**From:** PM  
**To:** Dev  
**Status:** Queued (after P28-E3-T1 Pass)  
**Date:** 2026-08-31  
**Phase:** Phase 28 — Catch-up replay — **ADR-029**  
**Branch:** `phase/28-local-reliability`

## Task
**P28-E4-T1 — Catch-up replay: drain Observation cursor after offline**

## Why
Feature Worker сегодня стартует с **DB tip** — backlog не пересчитывается. После offline (ноут спал, companion накопил flush) юзер открывает Dashboard и не видит свежие Features/Insights сразу.

## Acceptance Criteria
1. **Cursor semantics:** on start / after ingest batch / on wake hook — worker drains `list_after_created_cursor` until empty (bounded batch loop, idle between empty polls).
2. **No data loss:** all Observations inserted while app was running (or flushed on wake) produce Features within reasonable time (tests with mock source).
3. **Snapshot refresh:** `get_feature_snapshot` / `get_insights` reflect replayed batch without manual restart.
4. **Idle-safe:** no tight spin; existing poll interval when empty.
5. Tests: `cargo test -p desktop` / `runtime` relevant; document intentional change from “tip only on cold start”.
6. Handoff: `docs/handoffs/P28-E4-T1-dev-to-qa.md`.

## Out of scope
- Persisted Feature history SQLite; cloud sync
- iOS buffer (P28-E2)

## Constraints
- No new SQLite migration
- No `unwrap()` in production paths

## Operator smoke
1. Stop Desktop → ingest N observations via HTTP/curl → start Desktop → Features appear without second restart.
2. Companion flush batch while Desktop running → snapshot updates within one worker cycle.

## Next chat (после P28-E3 Pass)
```
как агент: режим build-qa для P28-E4-T1.
Brief: docs/handoffs/P28-E4-T1-pm-brief.md
1) Как Dev — собери по AC, создай docs/handoffs/P28-E4-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P28-E4-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
