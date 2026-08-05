# PM Brief → Dev: P4-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass, 2026-08-05)  
**Date:** 2026-08-05  
**Closed previous:** Epic **P4-E1** (T1–T3) — snapshot IPC · Dashboard shell · Recharts

## Task
**P4-E2-T1 — knowledge-engine skeleton + Insight types**

## Why
Dashboard already shows Features. Next epic: deterministic Knowledge Insights. First step — crate leaves stub-only: `Insight` / Evidence types + API that can return empty (real product rules → T2).

## Acceptance Criteria
1. `crates/knowledge-engine` is no longer stub-only: types for `Insight` and Evidence (references Feature and/or Signal ids).
2. Public API: `Features + Signals → Result<Vec<Insight>>` (empty `Ok` is valid).
3. Errors via `thiserror` (or project-consistent error type); no production `unwrap` / `expect`.
4. Unit tests: happy path (at least one Insight) + empty input / no-match → empty vec.
5. No SQLite, no UI, no LLM, no new persistence schema.
6. Handoff: `docs/handoffs/P4-E2-T1-dev-to-qa.md` with commands (`cargo test -p knowledge-engine`).

## Out of scope
- Real product rules (≥2 rules) → **P4-E2-T2**
- IPC `get_insights` / Dashboard list → **P4-E2-T3**
- report-engine / LLM → **P4-E3**
- New SQLite tables (needs ADR)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Ubiquitous Language: `Insight` (not Feature math in this crate beyond reading inputs)
- Prefer reusing Feature/Signal types from `feature-engine` / `bio-spec` where they already exist — don’t invent a parallel metric model
- Branch: `phase/4-dashboard-ai`

## After QA Pass
PM → Ready **P4-E2-T2** (Rule Insights v1 — Dev).

## Next chat (скопируй в новый чат)
```text
как агент: режим build-qa для P4-E2-T1.
Brief: docs/handoffs/P4-E2-T1-pm-brief.md
1) Как Dev — собери по AC, создай docs/handoffs/P4-E2-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P4-E2-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
