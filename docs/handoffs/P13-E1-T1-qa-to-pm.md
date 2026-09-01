# QA → PM: P13-E1-T1

## Meta
- **Task ID:** P13-E1-T1
- **Title:** ADR-013: Plugin wave-2 scope (IDE or Git) + Observation contract
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P13-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```text
rg ADR-013 across decision-log + 08 / 07 / 10 / 12 / 16 / 06 / 04 → present
rg git_activity | BIOFOCUS_GIT_ACTIVITY | GitActivityRate | com.biofocus.macos.git → contract + glossary + catalog + ADR
rg rejected (IDE+Git; always-on; cloud git; marketplace; parallel SQLite; weather/App Store primary; NotificationPressure; DistractionScore redefine; PR freeze; paths/remotes) → present in ADR-013 detail
rg CREATE TABLE.*(git|plugin|repo|sync) → none
git diff --name-only crates/ apps/ → empty (docs-only)
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 ADR-013 chooses **exactly one** source (**Git**); rationale vs ADR-010; Capability Model | **Pass** |
| AC2 Observation contract sketch (privacy-safe; opt-in off; idle; forbids paths/remotes/diffs/…) | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema — none to apply; existing `observations`; no migration | **Pass** |
| AC5 E2 plugin → channel → persist; E3 `GitActivityRate` locked; calm framing; ADR-007; not DistractionScore | **Pass** |
| AC6 Docs notes in 08 / 07 / 10 / 12 / 16 / 06 / 04 — not deferred | **Pass** |
| AC7 Handoff | **Pass** |
| Global DoD / out of scope / no PR | **Pass** |

### Extra checks
- QA fixed a typo in ADR-013 E3 sketch (`Distinc` → `Distinct`) — non-blocking polish.
- Out of scope respected: no collector code, no Feature math, no weather/App Store product, no PR.

## Defects (if any)
None blocking.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P13-E1-T1** Done; Ready **P13-E2-T1** shaped by ADR-013 (Git activity plugin / `BIOFOCUS_GIT_ACTIVITY` / `git_activity`)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs as needed: `docs/00-vision.md` / `docs/14-roadmap.md` / `docs/ARCHITECTURE_STATUS.md` / `docs/PROJECT_CANVAS.md` — lock wave-2 = Git (not IDE)
- [ ] Optionally mirror ADR lock: E2 = `git_activity` plugin; E3 = `GitActivityRate`

## Suggested next Ready task
- **P13-E2-T1** — Implement Git activity ambient/plugin collector (ADR-013): `com.biofocus.macos.git` / `data_type: "git_activity"`; opt-in `BIOFOCUS_GIT_ACTIVITY`; contracts + idle-safe tests. Branch: `phase/13-plugin-wave-2`.

## Notes for PM
- **No schema approve required** — no migration proposed.
- Public names locked: `git_activity`, `GitActivityRate`, packaging/weather/IDE remain deferred.
- Do **not** open a PR during freeze (until 2026-09-01).
