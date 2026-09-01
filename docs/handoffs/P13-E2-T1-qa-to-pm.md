# QA → PM: P13-E2-T1

## Meta
- **Task ID:** P13-E2-T1
- **Title:** Implement Git activity plugin (ADR-013)
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P13-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands:
  - `cargo test -p bio-spec -p macos-collector -p pipeline -p ingest --tests` → all green (incl. `git_activity_*` unit/integration + normalize strip/reject)
  - `cargo check -p desktop` → ok
  - `rg` for `BIOFOCUS_GIT_ACTIVITY` / `com.biofocus.macos.git` / `validate_git_activity_payload` / `GitActivityPlugin` across crates + docs → present
  - No UI→SQLite under `apps/desktop/src`; `unwrap`/`expect` only in `#[cfg(test)]` for new git modules

| AC | Result |
| :--- | :--- |
| AC1 Plugin id + Capability `git_activity` in macos-collector | **Pass** |
| AC2 ingest_host only when `BIOFOCUS_GIT_ACTIVITY=1`; same channel → persist; UI↛SQLite | **Pass** |
| AC3 Closed-set `activity_kind` + optional `event_count` ≥ 1; no forbidden fields | **Pass** |
| AC4 Scripted probe; system soft-fail idle; stop freezes polls; no path-allowlist table | **Pass** |
| AC5 Docs shipped + `validate_git_activity_payload` | **Pass** |
| AC6 Emit → channel → persist; stop freezes; normalize strips forbidden keys | **Pass** |
| AC7 Dev handoff | **Pass** |
| Global DoD | **Pass** |

## Defects (if any)
- None blocking.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P13-E2-T1** Done; Ready **P13-E3-T1** (`GitActivityRate`)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Optionally mirror: Epic P13-E2 ✅; status/canvas notes that collector shipped (soft-fail OS probe until future allowlist ADR)

## Suggested next Ready task
- **P13-E3-T1** — Catalog Feature `GitActivityRate` from `git_activity` Observations + pipeline normalize (already strips extras). Branch: `phase/13-plugin-wave-2`.

## Notes for PM
- Production `SystemGitActivityProbe` intentionally emits nothing without a path-allowlist ADR — Feature E3 can still dogfood via scripted Observations / HTTP ingest fixtures.
- Locked names: `git_activity`, `com.biofocus.macos.git`, `BIOFOCUS_GIT_ACTIVITY`, `GitActivityRate`.
- PR freeze until 2026-09-01 — no PR from this handoff.
