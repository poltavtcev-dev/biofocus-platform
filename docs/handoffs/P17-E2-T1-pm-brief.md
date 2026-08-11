# PM Brief → Dev: P17-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P17-E1-T1** — ADR-018 contracts locked (QA Pass)  
**Evidence:** `docs/handoffs/P17-E1-T1-qa-to-pm.md` · `docs/decision-log.md` ADR-018 · `docs/07-contracts.md`

## Task
**P17-E2-T1 — Expand iOS Companion / HealthKit ingest per ADR-018**

## Why
ADR-018 locked Observation `data_type` payloads and chart IPC stance. Next: emit the new wearable types from the existing Companion autonomy path (HealthKit event → local queue → Desktop ingest) so E3 can chart and compute Features from real Observations. Prefer Mi Fitness samples that already land in Apple Health — no Mi Cloud.

## Acceptance Criteria
1. iOS Companion observes / queries HealthKit for ADR-018 types and enqueues Observations via the existing Auto-sync / queue → `POST /v1/ingest` path (ADR-016 autonomy; **no** busy-loop HK poll).
2. Emit when samples exist (payloads per ADR-018 / `docs/07-contracts.md`):
   - **Required family:** `step_count` (`count` required; optional `window_secs`), `active_energy` (`kcal` required), `sleep_interval` (`start`/`end` required; optional `stage` closed-set).
   - **Keep:** `heart_rate` + soft-optional `hrv` (unchanged contracts).
   - **Soft-optional:** `oxygen_saturation` (`spo2_percent` 0–100) **only** when HK has samples — never invent.
3. Core ingest accepts the new types: `bio_spec` validators (and pipeline normalize if needed) reject malformed payloads calmly; valid samples persist to existing `observations` store.
4. Privacy / idle: no raw HK sample dumps in default logs; prefer Observation `id` / `data_type` counts; personal self-tracking only; calm non-clinical copy (no hypoxia / apnea / diagnosis framing).
5. Docs as shipped for Companion emit path: `12-development` / dogfood notes / contracts as needed. Chart ranges + Feature formulas remain → **P17-E3**.
6. Tests: validators + Companion/queue path coverage where practical (scripted or unit); soft-omit SpO2 / missing sleep stages; after stop / disable, no busy emission.
7. **No** Dashboard range picker / `get_feature_series` host; **no** new Feature formulas beyond existing catalog; **no** SQLite migration; **no** Mi Cloud / unofficial API; **no** workout Observation family (use Life Event + steps/energy).
8. Handoff: `docs/handoffs/P17-E2-T1-dev-to-qa.md`.

## Out of scope
- Chart range picker UI + `get_feature_series` + catalog Features (`EnergyScore` / `SleepDebt` / …) → **P17-E3-T1**
- Ambient light / weather / IDE / App Store / NotificationPressure
- Feature-history SQLite; clinical SpO2 claims; ECG/BP as required types
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-018 is payload SoT; ADR-017 remains sequencing only
- Branch: `phase/17-wearable-charts`
- Existing `observations` table only — schema change needs new ADR + user approve
- LLM remains L5 interpret-only — must not invent bracelet metrics
- SpO2 sparse on Mi is expected — soft-omit by design

## After QA Pass
PM → mark P17-E2-T1 Done; Ready **P17-E3-T1** (chart ranges IPC/UI + Features).
