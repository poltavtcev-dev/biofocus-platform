# PM Brief → Dev: P4-E2-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P4-E2-T1 (QA Pass — `KnowledgeEngine` + `bio-spec` Insight/Evidence)

## Task
**P4-E2-T2 — Rule Insights v1**

## Why
Skeleton evaluates pluggable rules but ships none. Product needs ≥2 deterministic Insights from Features/Signals so Dashboard (T3) and reports (E3) have real content — still no LLM, no SQLite.

## Acceptance Criteria
1. ≥2 deterministic product rules registered (examples: `High_Stress` Signal → Insight; elevated Focus / ContextSwitch pattern).
2. Each Insight has calm, non-evaluative / non-clinical copy.
3. `evidence_list` references Feature and/or Signal ids (`EvidenceRef`).
4. Unit tests: trigger path + no-trigger / empty → no spurious Insights.
5. No persistence ADR, no UI/IPC yet, no LLM rewrite of Insights.
6. Handoff: `docs/handoffs/P4-E2-T2-dev-to-qa.md` with `cargo test -p knowledge-engine`.

## Out of scope
- IPC `get_insights` / Dashboard list → **P4-E2-T3**
- report-engine / LLM → **P4-E3**
- New SQLite tables

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Reuse T1 API: `KnowledgeEngine::evaluate` + `InsightRule`; prefer a `register_*_v1` (or equivalent) for product rules — not test-only scaffolds
- Default empty / unregistered engine may still return `[]`; document how host should register rules
- Branch: `phase/4-dashboard-ai`

## Skeleton reference (T1)
- Types: `bio-spec::{Insight, EvidenceRef, Feature, Signal}`
- Engine: `crates/knowledge-engine` — `InsightRule`, `KnowledgeEngine::evaluate`
- Errors: `KnowledgeEngineError` (`thiserror`)

## After QA Pass
PM → Ready **P4-E2-T3** (Insights IPC + Dashboard list — Dev + UX).

## Next chat (скопируй в новый чат)
```text
как агент: режим build-qa для P4-E2-T2.
Brief: docs/handoffs/P4-E2-T2-pm-brief.md
1) Как Dev — собери по AC, создай docs/handoffs/P4-E2-T2-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P4-E2-T2-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
