# PARKED → OPENED — Phase 17 intent

**Date opened:** 2026-08-11 (P16 Done)  
**Status:** **Active** — contracts **ADR-018** (P17-E1); implement E2/E3 next  
**ADR:** ADR-017 (sequencing) · **ADR-018** (payload + chart IPC lock)  
**User decision:** Option **B** — finish Phase 16 ambient light first; then this wave ✅

## Why was parked

Dogfood asked for (1) maximum bracelet info under Mi Fitness/HealthKit constraints, (2) calm Dashboard charts, (3) analysis **with and without** AI. User chose not to preempt P16 again.

## Must implement in Phase 17 (committed backlog)

### A. Wearable depth (Companion / HealthKit) — ADR-018

- Maximize what Mi Fitness actually writes into Apple Health (not Mi Cloud).
- Keep existing HR + soft-optional HRV SDNN path.
- Locked Observations: `step_count`, `active_energy`, `sleep_interval`; soft-optional `oxygen_saturation`.
- Same autonomy: HK observer → local queue → Desktop ingest flush; no busy-loop; no clinical claims.

### B. Dashboard chart range UI — ADR-018

- Range picker: **1 hour / 8 hours / 12 hours / 1 day / 1 week**.
- Longer ranges: **recompute-on-read** via `get_feature_series` (ADR-008); coarser step for longer spans.
- Snapshot list: show **latest** Features. Chart holds the series.
- UI ↔ IPC only (no UI→SQLite).

### C. Analysis ladder

1. Deterministic Features / Insights / Recommendations from new + existing Observations.  
2. Optional L5 report / local LLM = interpret-only over Evidence (never invent bracelet values).

## Unblock checklist (PM)

1. ~~P16-E1-T1 + P16-E2-T1 Done~~ ✅  
2. ~~Follow-up ADR locking HealthKit `data_type` payloads~~ → **ADR-018** (P17-E1)  
3. ~~PM-brief epic split~~ — E1 contracts · E2 Companion · E3 ranges/Features in `SPRINT_ROADMAP`  

**Brief:** `docs/handoffs/P17-E1-T1-pm-brief.md`
