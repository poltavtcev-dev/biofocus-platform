# PM Brief → Dev: P10-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P10-E2-T1 (QA Pass — Browser categories collector); Epic **P10-E2** ✅  
**Evidence:** `docs/handoffs/P10-E2-T1-qa-to-pm.md`

## Task
**P10-E3-T1 — Catalog Feature `DistractionScore`**

## Why
ADR-010 + P10-E2 shipped opt-in `browser_category` Observations. Vision rule: Features only with real inputs. Catalog already names **`DistractionScore`** (context fragmentation from browser categories ± CSR). Close Phase 10 by registering a calm, confidence-aware catalog node — no new Dashboard chrome required.

## Acceptance Criteria
1. Move **`DistractionScore`** from `docs/06-feature-catalog.md` § Planned → §1 with: goal, window/step, units, inputs, formula strategy (v1), provenance, ADR-007 confidence, DAG registration note. Calm framing only (fragmentation / category mix — **not** clinical ADHD / “you are distracted” diagnosis).
2. Inputs: `browser_category` Observations required; optional combine with upstream `ContextSwitchRate` if documented as improving the signal (either choice OK if justified in catalog).
3. Pipeline: ensure `browser_category` is a known normalized type (canonical category set; strip forbidden keys `url` / `title` / `href` / content extras if present).
4. Register in `feature_engine::register_catalog_v1` (or helper wired into it); Feature appears on existing snapshot / Feature Worker path when inputs present — **no** mandatory new Dashboard UI.
5. Empty window, missing browser inputs, or only-`unknown` thin windows → **omit** Feature **or** emit with clearly lower confidence (pick one policy; document + test). No busy-loop; no new SQLite schema.
6. Unit tests: rich closed-set categories → emit; empty / unknown-only → omit or low confidence per policy; confidence behaves per ADR-007.
7. Optional: `ExplanationFactor`s if weighted components are clear (P7-E2 shape).
8. Handoff: `docs/handoffs/P10-E3-T1-dev-to-qa.md`.

## Out of scope
- Richer OS URL→category mapping (may stay deferred; Feature must work on scripted / closed-set categories)
- IDE/Git plugins; NotificationPressure; ambient sources
- New Insights / Recommendations rules for DistractionScore
- Dashboard redesign / dedicated chart (snapshot path enough)
- New SQLite schema / allowlist table
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-010 + collector contract in `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Modules: `crates/feature-engine`, `crates/pipeline` (normalize), `docs/06-feature-catalog.md` (+ API/dev notes if needed)
- Branch: `phase/10-plugin-wave-1`
- Prefer extend existing catalog registration — no parallel Feature registry
- LLM remains L5 interpret-only — must not compute Features

## After QA Pass
PM → mark P10-E3-T1 Done; close Epic **P10-E3** and **Phase 10** Kanban if no further P10 tasks; next horizon **Phase 11** via separate PM gate — **without** opening a PR during freeze.
