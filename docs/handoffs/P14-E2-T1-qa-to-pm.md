# QA → PM: P14-E2-T1

## Meta
- **Task ID:** P14-E2-T1
- **Title:** Implement Git watched-roots allowlist + live probe (ADR-014)
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P14-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p macos-collector --tests` → all green (unit + integration: empty allowlist no emit; fixture commit → persist; stop freezes; scripted path)
  - `cargo test -p pipeline normalize::tests::git_activity` → strip + reject still green
  - `cargo check -p desktop` → ok (earlier in Dev pass)
  - `rg` for `git-watched-roots` / `BIOFOCUS_GIT_WATCHED_ROOTS` / `SystemGitActivityProbe` / `resolve_watched_roots` across crate + docs → present; docs marked **shipped**
  - `git diff --name-only -- crates/feature-engine/` → empty (no Feature rewrite)
  - No `CREATE TABLE` / `ALTER TABLE` under `crates/macos-collector`
  - `unwrap`/`expect` only in `#[cfg(test)]` modules for new files
  - Branch: `phase/14-git-allowlist`; handoff present
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Load/validate TOML; empty/missing/unreadable → soft-fail idle; home/`~` expand | **Pass** |
| AC2 Live probe under allowlisted roots only; ADR-013 payload; no path dumps at default logs | **Pass** |
| AC3 ingest_host only when `BIOFOCUS_GIT_ACTIVITY=1`; channel→persist; ≥5s; stop joins | **Pass** |
| AC4 Scripted kept; empty/fixture/stop/forbidden-key coverage | **Pass** |
| AC5 Docs shipped + env override documented (file SoT when present) | **Pass** |
| AC6 No Feature rewrite; no SQLite migration / allowlist table | **Pass** |
| AC7 Dev handoff exists | **Pass** |
| Global DoD | **Pass** |

- Extra checks: privacy (no repo_path/remote/branch/sha/message/diff/author on persisted fixture Observation); personal self-tracking framing in docs.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P14-E2-T1** Done; Ready **P14-E3-T1** (dogfood / optional allowlist UX)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Optionally mirror: Epic P14-E2 ✅; `ARCHITECTURE_STATUS` / vision note that live allowlist probe shipped
- [ ] Docs already finalized by Dev for shipped probe path

## Suggested next Ready task
- **P14-E3-T1** — Dogfood gate and/or calm Settings/IPC to edit `~/.biofocus/git-watched-roots.toml` (no Feature math rewrite). Branch: `phase/14-git-allowlist`.

## Notes for PM
- Locked: file `~/.biofocus/git-watched-roots.toml`; env `BIOFOCUS_GIT_WATCHED_ROOTS` only if file absent; opt-in still `BIOFOCUS_GIT_ACTIVITY`.
- Dogfood needs both env enable + non-empty allowlist file (or CI env roots without a file).
- PR freeze until 2026-09-01 — do not open PR.
