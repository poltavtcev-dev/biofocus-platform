# PARKED → SHIPPED — Phase 17 intent

**Date opened:** 2026-08-11 (P16 Done)  
**Date closed:** 2026-08-11 (P17-E1–E3 QA Pass)  
**Status:** **Shipped** — Phase 17 Done  
**ADR:** ADR-017 (sequencing) · **ADR-018** (payload + chart IPC lock)  
**User decision:** Option **B** — finish Phase 16 ambient light first; then this wave ✅

## Shipped

### A. Wearable depth (Companion / HealthKit) — ADR-018 · P17-E2

- Maximize what Mi Fitness writes into Apple Health (not Mi Cloud).
- Keep HR + soft-optional HRV; emit `step_count`, `active_energy`, `sleep_interval`; soft-optional `oxygen_saturation`.
- Autonomy: HK observer → local queue → Desktop ingest; no busy-loop; no clinical claims.

### B. Dashboard chart range UI — ADR-018 · P17-E3

- Range picker: **1h / 8h / 12h / 1d / 1w** via `get_feature_series` recompute-on-read.
- Snapshot = **latest** Features; chart holds the series. UI ↔ IPC only.

### C. Analysis ladder — P17-E3

1. Deterministic Features: `ActivityBalance` / `EnergyScore` / `SleepDebt` (`register_wearable_v1`).
2. Optional L5 report / local LLM remains interpret-only.

**Evidence:** `docs/handoffs/P17-E*-qa-to-pm.md` · branch `phase/17-wearable-charts`  
**Next:** `docs/handoffs/PM-GATE-POST-P17-pm-brief.md`
