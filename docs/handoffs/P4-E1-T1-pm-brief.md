# PM Brief → Dev: P4-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Phase gate:** Phase 4 opened (user chose path A — Dashboard & local AI). Phase 3 E1–E3 closed in Kanban.

## Task
**P4-E1-T1 — Feature snapshot API + IPC**

## Why
Menubar already shows AlertLevel. Dashboard (E1-T2/T3) needs a **Feature time-series / snapshot** from Core over IPC — UI must not read SQLite or recompute Features.

## Acceptance Criteria
1. Public Core API returns a recent Feature snapshot (+ optional Signals) including ids, feature names/values, time windows, and provenance Observation ids where available.
2. New IPC command (preferred name: `get_feature_snapshot`) — keep `get_status` lean (version / dbStatus / alertLevel only).
3. IPC payload: **no** raw Observation biometric payloads, **no** absolute filesystem paths.
4. Unit tests: empty snapshot (idle / no evidence) + non-empty synthetic Features.
5. Idle-safe: read from worker cache / last EngineOutput — no busy-loop, no spin poll inside the command.
6. Document contract briefly in `docs/09-api.md` (+ note in handoff). Update `docs/12-development.md` only if needed for how to invoke.
7. Handoff: `docs/handoffs/P4-E1-T1-dev-to-qa.md`.

## Out of scope
- Dashboard React shell / Recharts (→ **P4-E1-T2**, **T3**)
- Knowledge Insights / `knowledge-engine` (→ **P4-E2**)
- LLM / `report-engine` (→ **P4-E3**)
- New SQLite tables / Insight persistence (needs ADR — not this task)
- LAN ingest / wearable bridges

## Constraints
- Production: no `unwrap` / `expect`
- UI ↔ IPC ↔ Core only
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Branch: `phase/4-dashboard-ai` (base on latest `main`; if `P3-E3-T3` Menubar commit not merged yet — land that PR first or include as prerequisite)

## After QA Pass
PM → Ready **P4-E1-T2** (Dashboard shell — UX + Dev).
