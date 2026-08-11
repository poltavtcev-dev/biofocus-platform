# PM Brief → Dev|UX: P23-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** P23-E2-T1 (`DeskAwayPresence` + health→prompt shipped; QA Pass)  
**Evidence:** `docs/handoffs/P23-E2-T1-qa-to-pm.md` · `docs/handoffs/P23-E2-T1-dev-to-qa.md`  
**Phase:** Phase 23 Personal Context Layer — Epic P23-E3  
**Branch:** `phase/23-personal-context`

## Task
**P23-E3-T1 — Dogfood notes + optional calm UI surface for Personal Context Layer**

## Why
E2 shipped **`DeskAwayPresence`** (`register_desk_away_v1` / catalog) and secondary **health-context.toml → L5 prompt/report** injection. Phase 23 closes with operator dogfood notes and optional calm presentation — **not** new math, **not** GPS, **not** diagnosis.

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §):  
   - How to verify `DeskAwayPresence` (emit with walk/steps evidence; omit quiet-alone / insufficient evidence; no GPS).  
   - How to verify health→prompt: create/edit `~/.biofocus/health-context.toml`; pack/`build_report_with_pack` path injects declared context; empty/missing = none; Feature values unchanged by health labels.  
   - Note: Phase 4 `build_report` (non-pack) does **not** auto-inject health — pack path does.
2. Calm framing only: “away from desk in this window” — **not** workplace presence / GPS tracking / clinical claims. Health copy: user-declared only — **not** “you have X from HRV”.
3. **Optional** calm Dashboard / UI surface:  
   - If existing Feature chart path shows catalog Features, ensure `DeskAwayPresence` is visible with calm label when present; omit stays quiet. Prefer reuse.  
   - Optional thin health-declare surface (path to edit/show declared conditions) — docs-only is enough if config file path is clear and Feature already surfaces via IPC. State which path in handoff.
4. Smoke notes in handoff: commands / steps; confirm UI ↛ SQLite.
5. **Do not** rewrite `DeskAwayPresence` / leaf formulas; **no** migration; **no** GPS; **no** large literature library; **no** new Insights/Recommendations required.
6. Handoff: `docs/handoffs/P23-E3-T1-dev-to-qa.md`.

## Out of scope
- Formula / DAG / ADR-024 math changes
- CircadianOffset; IDE; weather; App Store packaging
- Workplace / manager presence dashboards; clinical diagnosis UI
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. personal-context DoD)
- Modules: docs (+ optional desktop Feature presentation / health declare UX)
- Branch: `phase/23-personal-context`
- Personal self-tracking only
- LLM remains L5 interpret-only
- Prefer thin E3 — Phase 23 math + health→prompt are Done

## After QA Pass
PM → mark P23-E3-T1 Done; close Epic **P23-E3** and **Phase 23** if no further P23 tasks; next = **PM-GATE-POST-P23** (CircadianOffset · IDE · weather · App Store · Companion polish · TypingRhythm · other) **without** opening a PR during freeze.
