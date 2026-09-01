# QA → PM: P26-E2-T1

## Meta
- **Task ID:** P26-E2-T1
- **Title:** Author `docs/19-oss-public-launch.md` + hygiene touchpoints per ADR-027
- **Date:** 2026-08-12
- **Dev/UX handoff:** `docs/handoffs/P26-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
| Check | Result |
| :--- | :--- |
| `test -f docs/19-oss-public-launch.md` | **Pass** — SoT exists |
| Content: purpose / vs 18 / personal framing / AGPLv3 / App Store deferred / three layers / freeze table | **Pass** |
| Touchpoints link to `19-oss…` (vision, README, CONTRIBUTING, 12-dev, glossary, packaging related) | **Pass** |
| “public launch not Done” + dry-run → P26-E3 | **Pass** |
| No packaging secret recipes in 19 (`\.p12` / `notarytool submit` / passwords) | **Pass** |
| `git diff --name-only -- crates/ apps/` | **Pass** — empty |
| Handoff present | **Pass** |

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 SoT body per ADR-027 | **Pass** | |
| AC2 Three layers + freeze vs after-freeze | **Pass** | |
| AC3 Hygiene touchpoints; no “public launch Done” | **Pass** | |
| AC4 Schema/product none | **Pass** | docs-only |
| AC5 No public Release / visibility flip; dry-run → E3 | **Pass** | |
| AC6 12-development / glossary E2 SoT authored | **Pass** | |
| AC7 Handoff | **Pass** | |
| Global DoD | **Pass** | ADR-027; PR freeze; no secrets |

### Extra checks
- Cross-link from `18-packaging-runbook.md` → 19 keeps ops split honest.
- Vision broken-link target for `19-oss…` now resolves.

## Defects (if any)
- None. (Initial QA `rg` on exclusion-table wording was a false positive; wording tightened — no credential recipes in 19.)

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P26-E2-T1** to Done; Ready **P26-E3-T1** (dry-run checklist vs packaging runbook)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: note SoT authored; still **not** claim public launch Done
- [x] Do **not** open a PR (freeze until 2026-09-01)
- [x] Brief: `docs/handoffs/P26-E3-T1-pm-brief.md`

## Suggested next Ready task
- **P26-E3-T1** — Dry-run checklist vs `docs/18-packaging-runbook.md` (unsigned/local notes OK in freeze; notarized public Release only after freeze + layer 2).

## Notes for PM
- Branch: `phase/26-oss-public-launch`.
- Layer (1) SoT checkbox marked done; remaining honesty items stay maintainer checklist.
- Do not flip visibility or cut public Release when closing E2.
