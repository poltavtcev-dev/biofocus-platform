# PM Brief → UX + Dev: P11-E3-T1

**From:** PM  
**To:** UX + Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P11-E2-T1 (QA Pass — versioned prompt packs); Epic **P11-E2** ✅  
**Evidence:** `docs/handoffs/P11-E2-T1-qa-to-pm.md`

## Task
**P11-E3-T1 — Local LLM provider UX + pack-aware Report flow**

## Why
ADR-011 + P11-E2 shipped `build_report_with_pack` and default pack `biofocus.default` @ `1`. Phase 4 Dashboard still uses env-only LLM and Phase 4 `build_report`. Close Phase 11 by surfacing calm provider status and wiring Generate report to the pack API — still interpret-only, explicit user action, no auto-invoke.

## Acceptance Criteria
1. Dashboard shows calm **provider status** for local LLM (`disabled` / `ready` / `error` or equivalent) via IPC — UI ↛ SQLite; no secrets/tokens in UI logs; status reflects host env/config (`BIOFOCUS_LOCAL_LLM` / Phase 4 knobs — may remain env-backed; in-app “toggle that writes secrets” not required if status is honest).
2. Report / coaching flow uses **`build_report_with_pack`** with default pack (or a documented pack id/version) behind **explicit** user action (existing Generate report or calm equivalent); never on Dashboard open, soft poll, or background timer. Pass Features + Insights + Recommendations when available.
3. When LLM off: offline markdown + `llm_prompt` still returned/shown; calm copy that local AI is optional.
4. When LLM on: optional `interpret_report` soft-fails without losing markdown (`llmStatus` pattern from Phase 4).
5. Copy non-clinical; no diagnosis / employee-surveillance framing; no cloud marketplace UI.
6. Docs: `09-api` / `12-development` / desktop README as needed; mock/dev path for browser QA if useful.
7. Handoff: `docs/handoffs/P11-E3-T1-dev-to-qa.md` with smoke steps (status visible; Generate → markdown; LLM off/on soft-fail).

## Out of scope
- Cloud LLM providers / marketplace; auto-invoke on open
- Chat history / coaching transcript SQLite
- On-disk user-editable pack overrides (future ADR)
- Ambient sources / commercial packaging (Phase 12+)
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD + Coaching DoD from `docs/SPRINT_ROADMAP.md`
- ADR-011 + pack API in `docs/09-api.md` (`build_report_with_pack`, `biofocus.default`)
- Modules: `apps/desktop` (+ `src-tauri` IPC); evolve existing Report path — **no** Coach Engine crate
- Branch: `phase/11-ai-coaching-polish`
- Roles: **UX** for Dashboard surface/copy; **Dev** for IPC/host wire
- LLM remains L5 interpret-only

## After QA Pass
PM → mark P11-E3-T1 Done; close Epic **P11-E3** and **Phase 11** Kanban if no further P11 tasks; next horizon **Phase 12+** via separate PM gate — **without** opening a PR during freeze.
