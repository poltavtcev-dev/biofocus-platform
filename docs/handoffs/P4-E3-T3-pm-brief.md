# PM Brief → UX + Dev: P4-E3-T3

**From:** PM  
**To:** UX + Dev  
**Status:** Closed  
**Date:** 2026-08-05  
**Closed previous:** P4-E3-T2 (QA Pass with notes — `LocalLlmConfig` + `interpret_report` in `report-engine`, opt-in default OFF)  
**Closed this:** QA Pass with notes → `docs/handoffs/P4-E3-T3-qa-to-pm.md`; Epic **P4-E3** ✅ · Phase 4 ✅

## Task
**P4-E3-T3 — Report UX in Dashboard**

## Why
Offline `build_report` and optional local LLM interpret exist in Core. Next: Dashboard flow so the user can **explicitly** generate a report (deterministic markdown/prompt first; optional local LLM only when enabled) — never auto-send on startup.

## Acceptance Criteria
1. Dashboard has a clear «Generate report» (or equivalent) control / flow opened by the user.
2. Flow shows deterministic markdown and/or prompt from `build_report` (via IPC — UI ↛ SQLite / Core crates directly).
3. If local LLM is enabled (env/config from T2), optional interpreted output is available behind the same explicit user action; if disabled, calm offline-only path (no error panic; no network attempt from UI when off — or typed disabled state).
4. Copy states that AI is **local / optional**; calm, non-evaluative, no clinical claims.
5. Never auto-invoke LLM (or report build that hits network) on app / Dashboard open.
6. Idle-safe: no busy-loop; refresh only on user action (or documented rare timer if needed for display only).
7. Handoff: `docs/handoffs/P4-E3-T3-dev-to-qa.md` with smoke steps (open Dashboard → generate → offline path; optional note how to enable LLM for manual smoke).

## Out of scope
- Insight persistence / new SQLite schema
- LAN ingest, cloud account UX, mandatory AI
- Changing Feature math, Insight rules, or T1/T2 crate APIs beyond thin IPC glue
- Menubar redesign

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `apps/desktop` (+ `src-tauri` IPC glue as needed); reuse `report_engine::build_report` and optionally `interpret_report` / `LocalLlmConfig::from_env()`
- LLM interprets only — Features stay in `feature-engine`
- Branch: `phase/4-dashboard-ai`

## After QA Pass
Done — Epic **P4-E3** / Phase 4 closed 2026-08-05. Next ops: cluster PR on `phase/4-dashboard-ai`.

