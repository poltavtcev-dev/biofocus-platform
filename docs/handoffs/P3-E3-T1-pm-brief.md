# PM Brief → Dev: P3-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E2-T4 (QA Pass; [PR #22](https://github.com/poltavtcev-dev/biofocus-platform/pull/22) merged)

## Task
**P3-E3-T1 — Alert level mapping (Core)**

## Why
Catalog Features + `High_Stress` Signal ready. Menubar needs a deterministic Core mapping Features/Signals → traffic-light level before IPC (T2) and UX (T3).

## Acceptance Criteria
1. Public deterministic map: `EngineOutput` (Features + Signals) → `AlertLevel` { `Green`, `Yellow`, `Red` } (snake_case serde OK).
2. Rules documented in rustdoc + Dev handoff (and briefly in `docs/12-development.md`).
3. Unit tests: at least Green (empty / calm), Yellow (elevated without High_Stress), Red (`High_Stress` present).
4. No Tauri UI / Menubar / IPC / new SQLite schema / ADR.
5. Handoff: `docs/handoffs/P3-E3-T1-dev-to-qa.md`.

## Suggested v1 rules (Dev may refine if documented)
- **Red** if any Signal `High_Stress` in the output (or Severity High/Critical on that type).
- **Yellow** else if latest `StressIndex` **or** latest `FatigueIndex` scalar **> 60** (below High_Stress contiguous trigger, but elevated).
- **Green** otherwise (including empty output — idle-safe, no alarm without evidence).

## Out of scope
- IPC / `get_status` field (→ **P3-E3-T2**)
- Menubar colors / copy (→ **P3-E3-T3**)
- Mi Band 8 bridge / LAN ingest (owner dogfood — later; see `PROJECT_CANVAS`)
- Persisting alert level

## Constraints
- Production: no `unwrap` / `expect`
- Modules: `feature-engine` (preferred) or `runtime` alert module
- Idle-safe: pure function, no busy-loop
- Branch: `phase/3-pipeline-features`

## After QA Pass
PM → Ready **P3-E3-T2** (IPC expose alert level).
