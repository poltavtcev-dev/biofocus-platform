# QA → PM: P9-E1-T1

## Meta
- **Task ID:** P9-E1-T1
- **Title:** ADR-009: Recommendations domain / engine shape
- **Date:** 2026-08-08
- **Dev/UX handoff:** `docs/handoffs/P9-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg -n "ADR-009"` across decision-log, domain, storage, pipeline, api, glossary, vision, 12-development — all present.
  - `rg -n "focus_dip_pace_hint_v1"` in decision-log + `09-api.md` — present.
  - No `CREATE TABLE … recommendation` in ADR/storage — OK (no schema to apply).
  - `git diff --name-only -- crates/ apps/` — empty (docs-only; no migration / code).
  - Glossary `Recommendation` term + domain §1.5 + Dev handoff file — present.
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-009: first-class `Recommendation` + `RecommendationRule` in `knowledge-engine`; Features/Insights/Evidence; evaluate-on-read; idle/privacy local-only / no busy-loop | **Pass** |
| AC2 Rejected alternatives (LLM SoT; clinical framing; Coach Engine; cloud sync; persist history; thin-text-only; replace Insights) | **Pass** |
| AC3 Schema sketched as none; no migration; deferred dismiss history needs future ADR + approve | **Pass** — **no user schema approve required** for v1 |
| AC4 E2 sketch `focus_dip_pace_hint_v1` (Feature + Insight Evidence; calm non-clinical copy) | **Pass** |
| AC5 Glossary + domain note for `Recommendation` (not deferred thin-text-only) | **Pass** |
| AC6 `docs/handoffs/P9-E1-T1-dev-to-qa.md` | **Pass** |
| Global DoD: UL terms; calm framing; UI↛DB; no parallel crate; LLM not computing Recommendations | **Pass** |

- Extra checks: Out of scope respected (no RecommendationRule implementation, no IPC surface, no CircadianOffset/SleepDebt, no PR). Calm copy in ADR/API sketch is optional personal hint language.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move P9-E1-T1 to Done; Ready **P9-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: confirm `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` reflect ADR-009 recorded (Dev already touched vision/canvas L4 row lightly)
- [x] Brief for next: `docs/handoffs/P9-E2-T1-pm-brief.md`

## Suggested next Ready task
- **P9-E2-T1** — Recommendations v1 engine path (`bio-spec::Recommendation`, `EvidenceRef::Insight`, `RecommendationRule` + `focus_dip_pace_hint_v1` in `knowledge-engine`, unit tests). No schema approve gate.

## Notes for PM
- Branch: `phase/9-recommendations` (from Phase 8 tip).
- Evidence: this file + `docs/handoffs/P9-E1-T1-dev-to-qa.md` + ADR-009 in `docs/decision-log.md`.
- PR freeze still active — no PR.
