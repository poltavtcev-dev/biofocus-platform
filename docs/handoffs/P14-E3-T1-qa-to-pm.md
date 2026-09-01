# QA → PM: P14-E3-T1

## Meta
- **Task ID:** P14-E3-T1
- **Title:** Dogfood gate + calm Git allowlist Settings/IPC
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P14-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p macos-collector git_watched_roots` → 5/5 pass (incl. `BIOFOCUS_HOME` write round-trip, empty soft-fail, relative reject)
  - `cargo test -p desktop git_watched_roots_ipc` → 2/2 pass (get/set under `BIOFOCUS_HOME`, relative reject)
  - `pnpm exec tsc --noEmit` (apps/desktop) → exit 0 (earlier in Dev pass)
  - `rg` for IPC names / dogfood / `mockGitRoots` across desktop + docs → present; companion marked shipped
  - `git diff --name-only -- crates/feature-engine/` → empty
  - No migration SQL under collector/desktop host
  - Branch: `phase/14-git-allowlist`; handoff present
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Dogfood runbook + cross-links; enable + allowlist; soft-fail; privacy | **Pass** |
| AC2 IPC get/set file only; absolute roots; empty soft-fail; UI↛SQLite; no Observation path writes | **Pass** |
| AC3 Calm Menubar Git folders; personal framing; no Dashboard chart | **Pass** |
| AC4 BIOFOCUS_HOME tests + `?mockGitRoots=` | **Pass** |
| AC5 No Feature rewrite; no SQLite migration; ADR-013 payload unchanged | **Pass** |
| AC6 Docs mark companion shipped | **Pass** |
| AC7 Dev|UX handoff | **Pass** |
| Global DoD | **Pass** |

- Extra checks: Menubar copy is personal self-tracking (no workplace/surveillance); Settings IPC may return paths (documented exception vs Observation privacy).

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — mark **P14-E3-T1** Done; close Epic **P14-E3** and **Phase 14** if no further P14 tasks
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Optionally mirror: Phase 14 complete in `ARCHITECTURE_STATUS` / vision; next horizon via separate PM gate
- [x] **Do not open a PR** (freeze until 2026-09-01)

## Suggested next Ready task
- Separate **PM gate** for post–Phase 14 horizon (IDE / weather-light / App Store / NotificationPressure / other) — not a P14 task.

## Notes for PM
- Locked: Menubar **Git folders**; IPC `get_git_watched_roots` / `set_git_watched_roots`; mock `?mockGitRoots=`; dogfood § in `12-development`.
- Dogfood still needs `BIOFOCUS_GIT_ACTIVITY=1` + non-empty allowlist (file or CI env when file absent).
- Cluster code for Phase 14 (E1–E3) is on `phase/14-git-allowlist` — local commit OK when asked; no PR during freeze.
