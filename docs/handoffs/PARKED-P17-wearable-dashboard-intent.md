# PARKED — Phase 17 intent (after P16)

**Date:** 2026-08-11  
**Status:** Parked (not Ready)  
**ADR:** ADR-017 (sequencing) — payload contracts still need a follow-up ADR before build  
**User decision:** Option **B** — finish Phase 16 ambient light first; then this wave

## Why parked

Dogfood asked for (1) maximum bracelet info under Mi Fitness/HealthKit constraints, (2) calm Dashboard charts, (3) analysis **with and without** AI. User chose not to preempt P16 again.

## Must implement in Phase 17 (committed backlog)

### A. Wearable depth (Companion / HealthKit)

- Maximize what Mi Fitness actually writes into Apple Health (not Mi Cloud).
- Keep existing HR + soft-optional HRV SDNN path.
- Candidate new Observations (contract TBD): steps, active energy, sleep intervals; SpO2/workout only if present in Health.
- Same autonomy: HK observer → local queue → Desktop ingest flush; no busy-loop; no clinical claims.

### B. Dashboard chart range UI

- Range picker: **1 hour / 8 hours / 12 hours / 1 day / 1 week**.
- Today ~15m charts come from in-memory `CatalogAlertHook` prune (`WINDOW_SECS + 120`).
- Longer ranges: **recompute-on-read** from SQLite Observations (ADR-008); coarser step for longer spans.
- Snapshot list: show **latest** Features (not a dump of every CSR window). Chart holds the series.
- UI ↔ IPC only (no UI→SQLite).

### C. Analysis ladder

1. Deterministic Features / Insights / Recommendations from new + existing Observations.  
2. Optional L5 report / local LLM = interpret-only over Evidence (never invent bracelet values).

## Out of scope for this park note

- Implementing P17 now  
- New SQLite schema without ADR + approve  
- Mi unofficial API / cloud relay  
- Preempting P16-E1/E2

## Unblock checklist (PM)

1. P16-E1-T1 + P16-E2-T1 Done  
2. Follow-up ADR locking HealthKit `data_type` payloads for Mi depth  
3. PM-brief epic split: Companion ingest → Dashboard ranges/UX → catalog Features  
