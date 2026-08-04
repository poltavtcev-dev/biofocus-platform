# PM Brief → Dev: P2-E1-T3

**From:** PM  
**To:** Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E1-T2 — QA Pass with notes (`docs/handoffs/P2-E1-T2-qa-to-pm.md`)

## Task
**P2-E1-T3 — Persist ingest → `ObservationRepository` (+ mid-batch 503 contract)**

## Why
T1 queues into a channel only. Product needs immutable facts in SQLite. T1/T2 QA notes require an explicit mid-batch `queue_full` contract before collectors land.

## Acceptance Criteria
1. Successful ingest path writes each accepted `Observation` via `ObservationRepository::insert` (async worker from bounded channel is OK).
2. Duplicate PK → explicit error / non-silent response (no overwrite).
3. Backpressure: bounded channel; when full → **503** with documented JSON body.
4. **Mid-batch contract (required):** if some items in a request array were enqueued and a later send hits `queue_full`, choose and document **one** of:
   - **A)** all-or-nothing (nothing enqueued / rollback channel sends if possible), or
   - **B)** partial success: `202` with `{"status":"partial","accepted":N,"rejected":M}` (or equivalent), or
   - **C)** `503` with `accepted`/`rejected` counts after stopping further enqueue  
   Update `docs/09-api.md` + tests covering the chosen behaviour.
5. Round-trip tests: HTTP ingest → DB `get_by_id` / list (temp DB / `BIOFOCUS_HOME`).
6. No Feature/pipeline; no Tauri host wire (→ **T4**).
7. Handoff: `docs/handoffs/P2-E1-T3-dev-to-qa.md`.

## Out of scope
- `IngestConfig::load()` at desktop startup → **T4**
- QR / LAN bind / collectors / iOS
- Changing Observation schema (ADR)

## Constraints
- Production: no `unwrap`/`expect`
- Idle: writer must not busy-spin (block on recv / select)
- **Before or with this PR:** commit T1+T2 on `phase/2-ingest-http` and open PR if still missing
- Branch → PR → `main` after CI

## After QA Pass
PM → Ready **P2-E1-T4** (host wire + `GET /v1/status` + `IngestConfig::load`).
