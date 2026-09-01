# PM Brief → Dev: P13-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P13-E2-T1 (QA Pass — Git activity plugin); Epic **P13-E2** ✅  
**Evidence:** `docs/handoffs/P13-E2-T1-qa-to-pm.md`

## Task
**P13-E3-T1 — Catalog Feature `GitActivityRate`**

## Why
ADR-013 + P13-E2 shipped opt-in `git_activity` Observations (`GitActivityPlugin`, `BIOFOCUS_GIT_ACTIVITY`, coarse `activity_kind` + optional `event_count`; pipeline already strips forbidden keys). Vision rule: Features only with real inputs. Catalog / ADR already name **`GitActivityRate`**. Close Phase 13 with a calm, confidence-aware catalog node — distinct from Browser `DistractionScore`.

## Acceptance Criteria
1. Move **`GitActivityRate`** from `docs/06-feature-catalog.md` § Planned → §1 with: goal, window/step (**15m / 1m**, align Focus/CSR / DistractionScore / AmbientMediaShare), units, inputs, formula strategy (v1), provenance, ADR-007 confidence, DAG registration note. Calm framing only (“version-control cadence in this window” — **not** “you commit too little” / workplace policing / clinical).
2. Inputs: `git_activity` Observations required (`activity_kind` + optional `event_count`). Formula per ADR-013: rate/sum of non-idle closed-set events in window — sum `event_count` (default 1) for `activity_kind ∈ {commit, checkout, sync, other}` → scalar ≥ 0 (events per window **or** per minute — pick one; document + test). Empty / only-`idle` / only-`unknown` thin windows → **omit** Feature **or** emit with clearly lower confidence (pick one policy; document + test).
3. Pipeline: `git_activity` remains a known normalized type; keep stripping forbidden content keys (paths / remotes / branch / sha / message / diff / author / etc.) if present — never persist them via normalize path. Prefer reuse E2 normalize coverage; add tests only if gaps.
4. Register in `feature_engine::register_catalog_v1` (or helper wired into it); Feature appears on existing snapshot / Feature Worker path when inputs present — **no** mandatory new Dashboard UI. **Do not** merge into or redefine `DistractionScore` math.
5. Unit tests: rich commit/sync/other activity → emit; empty / idle-unknown-only → omit or low confidence per policy; confidence per ADR-007; optional `event_count` batches respected.
6. Optional: `ExplanationFactor`s if weighted components are clear (P7-E2 / DistractionScore / AmbientMediaShare shape).
7. Handoff: `docs/handoffs/P13-E3-T1-dev-to-qa.md`.

## Out of scope
- Live OS git path watching / path-allowlist table (production probe may stay soft-fail idle; Feature must work on scripted / persisted / HTTP-ingest `git_activity`)
- IDE collector; NotificationPressure; weather/light; App Store packaging
- New Insights / Recommendations rules for GitActivityRate
- Dashboard redesign / dedicated chart (snapshot path enough)
- New SQLite schema / watched-roots allowlist
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-013 + E2 contract in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Modules: `crates/feature-engine`, `crates/pipeline` (normalize if needed), `docs/06-feature-catalog.md`
- Branch: `phase/13-plugin-wave-2`
- Prefer extend existing catalog registration — no parallel Feature registry
- LLM remains L5 interpret-only — must not invent score or activity kinds
- Personal self-tracking only — not workplace monitoring

## After QA Pass
PM → mark P13-E3-T1 Done; close Epic **P13-E3** and **Phase 13** Kanban if no further P13 tasks; next horizon via separate PM gate — **without** opening a PR during freeze.
