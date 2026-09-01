# PM Brief → Dev: P11-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P10-E3-T1 (QA Pass — `DistractionScore`); Phase 10 complete  
**Evidence:** `docs/handoffs/P10-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 11 AI coaching polish (Sprint 21–22) — `docs/SPRINT_ROADMAP.md`

## Task
**P11-E1-T1 — ADR-011: AI coaching polish (prompt packs + provider UX)**

## Why
Vision L5 is *Coaching (AI) — NL explanation only* (`docs/00-vision.md` §3 / §7). Phase 4 already shipped `report-engine` + opt-in local LLM + Dashboard Report UX (env-only enable). Phase 10 closed plugin wave-1. Horizon Phase 11 is **prompt packs / provider UX** — still interpret-only — so power users get editable/named packs and calmer provider status without turning LLM into a Feature/Recommendation engine.

## Acceptance Criteria
1. Record **ADR-011** in `docs/decision-log.md`: choose v1 shape for **prompt packs** (named/versioned templates over already-computed Features / Insights / Recommendations) and **provider UX** (calm Dashboard surface for opt-in local LLM status/config); reaffirm L5 interpret-only boundary (LLM must not invent scores, Evidence, or Recommendations); local-first / idle / no auto-invoke on open.
2. Rejected alternatives documented (cloud LLM by default; LLM as SoT for Features/Recommendations; auto-invoke on Dashboard open; parallel “Coach Engine” crate that bypasses Evidence; clinical/prescription coaching tone; persist chat history SQLite without need; shipping packs without ADR).
3. If schema / new persistence is proposed: sketch only — **do not apply migration** until user approve. Prefer in-process packs + existing env/IPC (Phase 4 stance).
4. Short sketch: E2 packs API in `report-engine` → E3 Dashboard provider UX on existing `generate_report` / interpret path (explicit user action).
5. Docs touch: `09-api` / `10-security` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2/E3 (must be stated).
6. Handoff: `docs/handoffs/P11-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing prompt pack code (→ **P11-E2**)
- Dashboard provider UI / pack picker UX (→ **P11-E3**)
- Ambient music/weather/light or commercial packaging (Phase 12+)
- Cloud LLM marketplace; chat history product
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer evolve `report-engine` + existing Report IPC — no parallel Coach Engine crate without ADR justification
- Branch: `phase/11-ai-coaching-polish`
- Calm non-clinical copy only
- LLM remains L5 interpret-only — must not compute Features / Recommendations / Evidence

## After QA Pass
PM → mark P11-E1-T1 Done; if ADR requires schema approve, wait for user; else Ready **P11-E2-T1**.
