# PM Brief → Dev: P7-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-06  
**Closed previous:** P7-E2-T1 (QA Pass — ExplanationFactor + FocusScore factors); Epic P7-E2 ✅  
**Evidence:** `docs/handoffs/P7-E2-T1-qa-to-pm.md`

## Task
**P7-E3-T1 — First bio-backed Trust Features**

## Why
Confidence + factors are in place. Phase 7 Trust layer still needs ≥1 new catalog Feature that uses real Observation inputs already in dogfood (HR/HRV and/or Focus+CSR) — not backlog placeholders.

## Acceptance Criteria
1. Move **≥1** Feature from `docs/06-feature-catalog.md` planned backlog → §1 with formula / window / units / deps / provenance + ADR-007 confidence. Prefer one of:
   - **`RecoveryScore`** — short-term physiological recovery proxy from `hrv` / optional `heart_rate` (no sleep required for v1 if sleep Observations absent)
   - **`DeepWorkScore`** or **`AttentionStability`** — from `FocusScore` + `ContextSwitchRate` (already shipped)
2. Register in `register_catalog_v1` (or a helper wired into it).
3. Unit tests: synthetic Observations — empty/thin/rich; confidence behaves per ADR-007; idle-safe.
4. Calm non-clinical copy (no burnout / clinical diagnosis claims).
5. Optional: emit `ExplanationFactor`s if the formula has clear weighted components (reuse E2 shape).
6. Handoff: `docs/handoffs/P7-E3-T1-dev-to-qa.md`.

## Out of scope
- CognitiveLoad, SleepDebt (no sleep Observations), CircadianOffset (P8)
- Pattern Discovery, Recommendations engine
- Dashboard redesign / chart series for the new Feature (snapshot path is enough)
- New SQLite schema
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `crates/feature-engine`, `docs/06-feature-catalog.md` (+ API/dev notes if needed)
- Branch: `phase/7-trust-layer`
- No parallel Feature registry rewrite

## After QA Pass
PM → mark P7-E3-T1 Done; close Epic **P7-E3** and **Phase 7** Kanban if no further P7 tasks; then open Phase 8 (or next Ready) via PM gate — **without** opening a PR during freeze.
