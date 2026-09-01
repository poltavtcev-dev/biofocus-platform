# QA → PM: P12-E1-T1

**From:** QA  
**To:** PM  
**Date:** 2026-08-10  
**Dev/UX handoff:** `docs/handoffs/P12-E1-T1-dev-to-qa.md`  
**Verdict:** Pass

## What was verified

### Commands / evidence
```text
rg ADR-012 across decision-log + 08 / 07 / 10 / 12 / 16 / 06 / 04 → present
rg now_playing | AmbientMediaShare | BIOFOCUS_NOW_PLAYING → contract + glossary + catalog + ADR detail
rg rejected (all three ambient; always-on; cloud sync by default; PR during freeze; …) → present in ADR-012 detail
rg CREATE TABLE.*(now_playing|ambient|sync) in decision-log → none
git diff --name-only crates/ apps/ → empty (docs-only ADR)
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 ADR-012 primary track + sequencing + rationale | **Pass** — primary Now Playing ambient; secondary packaging runbook; vision / macOS / privacy / idle |
| AC2 Ambient Observation contract sketch | **Pass** — `now_playing`, `com.biofocus.macos.now_playing`, coarse payload, opt-in off, idle poll, existing `observations` |
| AC3 Packaging boundaries + AGPLv3 open | **Pass** — runbook in v1; App Store / sync product deferred; commercial ≠ closed math |
| AC4 Rejected alternatives | **Pass** — includes all-three ambient, always-on, sync-by-default, secret formulas, Features w/o Observations, UI→DB/cloud LLM packaging, IDE/Git into P12, PR during freeze, mic/lyrics |
| AC5 Schema sketch only — no migration | **Pass** — explicit “Do not apply”; `04-storage` note |
| AC6 E2 → E3 names locked | **Pass** — E2 Now Playing plugin; E3 `AmbientMediaShare` + packaging runbook |
| AC7 Docs notes (not deferred) | **Pass** — 08 / 07 / 10 / 12 / 16 / 06 (+ 04-storage) |
| AC8 Handoff | **Pass** — `docs/handoffs/P12-E1-T1-dev-to-qa.md` |
| Global DoD (docs) | **Pass** — Local-First / opt-in / Capability Model / UI↛DB unchanged / freeze noted |

### Extra checks
- No crate/app code in this task (correct OOS).
- Personal self-tracking framing; no workplace surveillance.
- LLM must not invent ambient Features / packaging policy (stated in ADR).

## Defects
None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — mark **P12-E1-T1** Done; Ready **P12-E2-T1** shaped by ADR-012 (Now Playing plugin)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Optionally mirror ADR lock in status docs if not already: E2 = `now_playing` plugin; E3 = `AmbientMediaShare` + packaging runbook
- [x] **No schema/sync approve needed** for v1 (sketch only; no migration)
- [x] **Do not open a PR** (PR freeze until 2026-09-01)

## Suggested next Ready task
- **P12-E2-T1** — Now Playing ambient plugin (`com.biofocus.macos.now_playing` / `data_type: "now_playing"`; opt-in `BIOFOCUS_NOW_PLAYING`; contracts + idle-safe tests). Branch: `phase/12-ambient-packaging`.

## Notes for PM
- Public names locked: `now_playing`, `AmbientMediaShare`, packaging companion runbook in E3.
- Weather / light / IDE/Git / cloud sync product remain deferred.
- Branch: `phase/12-ambient-packaging`.
