# PM Brief → Dev: P4-E3-T2

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes, 2026-08-05)  
**Date:** 2026-08-05  
**Closed previous:** P4-E3-T1 (QA Pass — `build_report` → offline `ReportDocument { markdown, llm_prompt }`)  
**Closed this:** QA Pass with notes → `docs/handoffs/P4-E3-T2-qa-to-pm.md`; Ready **P4-E3-T3**

## Task
**P4-E3-T2 — Optional local LLM adapter**

## Why
Deterministic report/prompt exists offline. Next: optional opt-in path to call a **local** LLM (Ollama / OpenAI-compatible) that **interprets** `ReportDocument::llm_prompt` only — default OFF, no auto-send, privacy-documented.

## Acceptance Criteria
1. Opt-in via env / flag / config; default **OFF** (no network when disabled).
2. Prefer localhost Ollama or OpenAI-compatible base URL; configurable base URL + model as needed.
3. Consumes `ReportDocument::llm_prompt` (or equivalent from T1); returns interpreted text via `Result`; **no Feature math** in the LLM path.
4. Timeout on HTTP; never auto-send on app startup (explicit call / user action path only — document).
5. Errors via `thiserror` (or project-consistent); no production `unwrap` / `expect`.
6. Privacy note in `docs/12-development.md` (local-only / opt-in / what leaves the machine when enabled).
7. Handoff: `docs/handoffs/P4-E3-T2-dev-to-qa.md` with tests / smoke (`cargo test` / `cargo check` on touched crates) and how to enable for QA.

## Out of scope
- Dashboard «Generate report» button / UX → **P4-E3-T3**
- Cloud account UX, mandatory AI, telemetry to vendor clouds by default
- Changing Feature math, Insight rules, or T1 markdown format (except thin adapter glue)
- New SQLite schema

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- LLM **interprets** only — Features stay in `feature-engine`
- Modules: `crates/report-engine` (shipped here, not desktop host)
- Reuse T1: `build_report` / `ReportDocument`
- Branch: `phase/4-dashboard-ai`

## Shipped
- Public: `LocalLlmConfig`, `interpret_report` / `interpret_llm_prompt`; env `BIOFOCUS_LOCAL_LLM*` (default OFF)
- Placement: `crates/report-engine` · privacy: `docs/12-development.md` · `docs/10-security.md` · API: `docs/09-api.md`

## After QA Pass
PM → Ready **P4-E3-T3** (Report UX in Dashboard — UX + Dev) unless sprint re-order.
