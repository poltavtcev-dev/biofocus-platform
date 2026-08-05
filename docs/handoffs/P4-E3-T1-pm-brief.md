# PM Brief → Dev: P4-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** Epic **P4-E2** (T1–T3) — knowledge-engine · rules v1 · IPC `get_insights` + Dashboard list

## Task
**P4-E3-T1 — report-engine prompt / markdown builder**

## Why
Insights and Features are available in Core/Dashboard. Next: leave stub `report-engine` and produce a deterministic offline markdown / LLM prompt string from Features (+ Insights) so E3-T2/T3 can attach optional local LLM and Report UX — without network in this task.

## Acceptance Criteria
1. `crates/report-engine` is no longer stub-only: public API accepts Features (+ Insights) and returns deterministic markdown and/or LLM prompt string (`Result`, offline).
2. No network calls (no Ollama/OpenAI HTTP) in this crate path.
3. Unit tests: non-empty inputs → stable output; empty / minimal inputs → calm empty or minimal report (document format).
4. Errors via `thiserror` (or project-consistent); no production `unwrap` / `expect`.
5. Document output format briefly in handoff (+ short note in `docs/12-development.md` or `09-api.md` if useful).
6. Handoff: `docs/handoffs/P4-E3-T1-dev-to-qa.md` with `cargo test -p report-engine` (and related).

## Out of scope
- HTTP to Ollama / OpenAI-compatible → **P4-E3-T2**
- Dashboard «Generate report» button / UX → **P4-E3-T3**
- New SQLite / Insight persistence
- Changing Feature math or Insight rules

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- LLM **interprets** only — never compute Features here
- Reuse `bio-spec` / existing Feature & Insight types; calm non-clinical copy in generated text
- Branch: `phase/4-dashboard-ai`

## After QA Pass
PM → Ready **P4-E3-T2** (optional local LLM adapter — Dev) unless sprint re-order.

## Next chat (скопируй в новый чат)
```text
как агент: режим build-qa для P4-E3-T1.
Brief: docs/handoffs/P4-E3-T1-pm-brief.md
1) Как Dev — собери по AC, создай docs/handoffs/P4-E3-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P4-E3-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
