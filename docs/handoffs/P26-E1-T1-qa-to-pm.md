# QA → PM: P26-E1-T1

## Meta
- **Task ID:** P26-E1-T1
- **Title:** ADR-027: lock OSS public-launch scope (gates, non-goals, relationship to packaging runbook)
- **Date:** 2026-08-12
- **Dev/UX handoff:** `docs/handoffs/P26-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
| Check | Result |
| :--- | :--- |
| `rg ADR-027` in decision-log / 12-development / 00-vision / 16-glossary | **Pass** |
| SoT `19-oss…` + packaging `18-packaging…` + layers + freeze | **Pass** in ADR-027 detail + Phase 26 note |
| Rejected alts (IDE / weather / App Store / Companion / TypingRhythm / DeepFocusLikelihood / GPS / Feature math / Pattern Discovery / PR freeze / cloud-telemetry / secret formulas / public Release during freeze) | **Pass** — items 1–15 in ADR-027 rejected list |
| `test ! -f docs/19-oss-public-launch.md` | **Pass** — body correctly deferred to E2 |
| `docs/18-packaging-runbook.md` exists | **Pass** |
| `git diff --name-only -- crates/ apps/` | **Pass** — empty (docs-only) |
| Handoff `P26-E1-T1-dev-to-qa.md` | **Pass** |

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 ADR-027 primary = OSS Public Launch Hygiene; SoT `19-oss…`; vs `18-packaging`; personal framing | **Pass** | Summary row + detail |
| AC2 Three checklist layers; freeze vs after-freeze | **Pass** | Locked sections in ADR-027 |
| AC3 Rejected alternatives | **Pass** | Full list present |
| AC4 Schema none | **Pass** | No migration / Observation / Feature math |
| AC5 E2 author launch doc; E3 dry-run vs packaging | **Pass** | E2/E3 sketch |
| AC6 Docs planned/ADR notes (`12` / `00-vision` / `16`) | **Pass** | Not deferred-only — notes landed |
| AC7 Handoff | **Pass** | |
| Global DoD | **Pass** | AGPLv3 open; packaging ≠ secret math; PR freeze; no Release/body in E1 |

### Extra checks
- Vision pointer names ADR-027 and keeps App Store deferred.
- Absence of `docs/19-oss-public-launch.md` is intentional (E2), not a defect.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P26-E1-T1** to Done; Ready **P26-E2-T1** (author `docs/19-oss-public-launch.md`)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: status notes that ADR-027 is locked (vision/roadmap already mention Phase 26)
- [x] Do **not** open a PR (freeze until 2026-09-01)
- [x] Brief: `docs/handoffs/P26-E2-T1-pm-brief.md`

## Suggested next Ready task
- **P26-E2-T1** — Author `docs/19-oss-public-launch.md` + hygiene touchpoints per ADR-027 (still no public Release flip; still no Feature math).

## Notes for PM
- Branch: `phase/26-oss-public-launch`.
- E2 must create the missing SoT file vision already links; must keep packaging ops in `18-packaging-runbook.md` without copying secrets.
- Unrelated dirty PM-gate docs on the branch are out of this task’s AC.
