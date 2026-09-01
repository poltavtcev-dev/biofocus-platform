# QA → PM: P12-E3-T1

## Meta
- **Task ID:** P12-E3-T1
- **Title:** Catalog Feature `AmbientMediaShare` + packaging runbook
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P12-E3-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
### Commands run + results
```text
cargo test -p feature-engine ambient_media  → 9 passed
cargo test -p pipeline now_playing          → 2 passed
cargo test -p feature-engine                → 77 passed
cargo test -p pipeline                      → 25 unit + 3 e2e passed
rg AmbientMediaShare / register_ambient_v1 / 18-packaging-runbook → present in catalog §1.9, 12-dev, 08, 16, runbook
rg title|artist|lyrics|playlist in normalize_now_playing → strip + tests
rg AmbientMediaShare in apps/desktop → none (no mandatory Dashboard UI)
rg CREATE TABLE.*(now_playing|ambient|sync) → none
unwrap/expect in ambient_media_share.rs → tests only
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Catalog §1.9 + remove from Planned; calm framing; 15m/1m; formula; provenance; ADR-007; DAG | **Pass** |
| AC2 Inputs `now_playing`; share 0–100; **omit** empty/none/unknown-only (documented + tested) | **Pass** |
| AC3 Known normalize type; forbidden content keys stripped | **Pass** |
| AC4 `register_ambient_v1` in `register_catalog_v1`; no new Dashboard UI | **Pass** |
| AC5 Packaging runbook + `12-development` link; sync off; AGPLv3 Core open | **Pass** |
| AC6 Unit tests (rich emit / omit thin / confidence) | **Pass** |
| AC7 Optional ExplanationFactors (playing kinds) | **Pass** |
| Global DoD / out of scope / no PR | **Pass** |

### Extra checks
- Policy choice matches DistractionScore shape: **omit** thin windows (not low-confidence emit).
- Paused closed-set → emit 0 (documented).
- Out of scope respected: no MediaRemote content mapping, no Insights/Recommendations, no schema, no Dashboard chart, no PR.

## Defects (if any)
None blocking.

## Notes (non-blocking)
- `docs/12-development.md` Now Playing collector bullet still ends with ``AmbientMediaShare` → P12-E3`` while the next bullet already documents P12-E3 shipped — PM may tighten wording on close.
- Notarization/codesign not executed (runbook is docs/process only — expected).

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P12-E3-T1** Done; close Epic **P12-E3** and **Phase 12** Kanban if no further P12 tasks
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs as needed: `docs/00-vision.md` / `docs/14-roadmap.md` / `docs/ARCHITECTURE_STATUS.md` / `docs/PROJECT_CANVAS.md` — Phase 12 complete; next horizon via separate PM gate
- [ ] Optional: tighten stale “→ P12-E3” wording in `docs/12-development.md` collector bullet

## Suggested next Ready task
- Separate PM gate for post–Phase 12 horizon (not a P12 micro-task). Do **not** open a PR during freeze (until 2026-09-01).

## Notes for PM
- Branch: `phase/12-ambient-packaging` (cluster PR after freeze).
- Evidence: this file + `docs/handoffs/P12-E3-T1-dev-to-qa.md`.
- Packaging companion: `docs/18-packaging-runbook.md`.
