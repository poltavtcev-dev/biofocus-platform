# PM Brief → Dev: P2-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E1-T4 — QA Pass with notes (`docs/handoffs/P2-E1-T4-qa-to-pm.md`); Epic **P2-E1** Done

## Task
**P2-E2-T1 — Active window Observation stream**

## Why
Ingest path is live on Desktop. Phase 2 collectors must emit local context as `Observation` (`context_window`) into the same channel/repo path — without keylogging content.

## Acceptance Criteria
1. macOS (or adapter behind `plugin-sdk`) detects active application/window changes and emits `Observation` with `data_type` = `context_window` (schema: `docs/04-storage.md`).
2. Payload is context metadata only (app/window identifiers as designed) — **not** keystroke content / clipboard / screenshots.
3. Polling/event interval ≥ ~1s **or** event-driven; no busy-loop when idle/paused.
4. Stop/pause stops emitting (clean lifecycle; joins without orphan spin).
5. Observations reach storage via existing Core path (channel → persist worker / repo) — UI still ↛ SQLite.
6. Tests: unit/integration covering emit on change + pause/stop idle where practical (mock OS APIs OK).
7. Handoff: `docs/handoffs/P2-E2-T1-dev-to-qa.md`.

## Out of scope
- Keystroke / input aggregates → **P2-E2-T2**
- Collector integration test epic → **P2-E2-T3**
- iOS / HealthKit → **P2-E3-***
- Feature pipeline, dashboard
- Changing E1 ingest contracts

## Constraints
- No `unwrap`/`expect` in production
- Idle footprint DoD
- Privacy: no content capture; document any Accessibility permission needs if required for window title/app id
- Prefer extending stub `plugin-sdk` toward `BioFocusPlugin` / stream traits from `docs/08-plugin-sdk.md` rather than ad-hoc UI coupling

## Hygiene
- Git: **commit per task** after PM Done; **push + PR once per sprint** (`docs/12-development.md`).
- Optional one manual GUI smoke before sprint PR: `pnpm tauri dev` + `curl 127.0.0.1:8787/v1/status` + quit (QA note from T4).

## After QA Pass
PM → Ready **P2-E2-T2** (or **P2-E2-T3** if T2 deferred); ∥ **P2-E0-T1** still available.
