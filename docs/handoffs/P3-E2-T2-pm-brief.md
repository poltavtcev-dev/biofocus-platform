# PM Brief → Dev: P3-E2-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E2-T1 (QA Pass; DAG scheduler skeleton)

## Task
**P3-E2-T2 — `ContextSwitchRate` + `FocusScore` (v1)**

## Why
Каркас DAG готов (`FeatureEngine::register` / `run`). Нужны первые catalog Features: `FocusScore` (window 15m / step 1m) и `ContextSwitchRate` из `context_window`, с provenance Observation IDs.

## Acceptance Criteria
1. Реализованы v1 по `/docs/06-feature-catalog.md`: `FocusScore` window 15m / step 1m (упрощения inputs OK если задокументированы); `ContextSwitchRate` из `context_window`.
2. Output Features + `provenance` Observation IDs.
3. Unit-тесты на синтетических данных (`docs/11-testing.md`).
4. Nodes регистрируются в существующий DAG API (`FeatureNode`); нет UI / новой SQLite-схемы / ADR.
5. Handoff: `docs/handoffs/P3-E2-T2-dev-to-qa.md`.

## Out of scope
- Stress/Fatigue + High_Stress Signal (→ **P3-E2-T3**)
- E2E Observation → Feature suite (→ **P3-E2-T4**)
- Wire `FeatureHook` → real engine in desktop (можно позже в E2)
- Menubar alerts (→ **P3-E3**)

## Constraints
- Production: no `unwrap` / `expect`
- Modules: `crates/feature-engine` (primary)
- Features stay in-memory / derived until ADR
- Branch: `phase/3-pipeline-features`
- Catalog: `/docs/06-feature-catalog.md`

## After QA Pass
PM → Ready **P3-E2-T3** (`StressIndex` / `FatigueIndex` + High_Stress).
