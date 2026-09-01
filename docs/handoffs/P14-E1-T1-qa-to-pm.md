# QA → PM: P14-E1-T1

## Meta
- **Task ID:** P14-E1-T1
- **Title:** ADR-014: Git watched-roots / path-allowlist + live probe boundaries
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P14-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg -n "ADR-014" …` → present in `decision-log`, `04-storage`, `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`
  - Config path `~/.biofocus/git-watched-roots.toml` + watched-roots language present across ADR + contracts/security/storage/glossary/plugin-sdk
  - Rejected alternatives present in ADR-014 detail (whole-disk, SQLite v1, workplace, cloud git, IDE/weather/App Store primary, NotificationPressure, PR freeze, migration without approve)
  - `CREATE TABLE` / `git_watched_roots` → only existing `observations` DDL + **deferred sketch** (explicitly not Phase 14 v1 / needs user approve) — **no migration applied**
  - `git diff --name-only -- crates/ apps/` → empty (docs-only)
  - Branch: `phase/14-git-allowlist`; `P14-E1-T1-dev-to-qa.md` present
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 One storage stance (config file) + rationale vs ADR-013 + Capability + personal self-tracking | **Pass** |
| AC2 Privacy — roots in config only; no paths/remotes/… in Observation payloads; logs prefer id/counts | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema — none to apply; non-SQLite; no migration | **Pass** |
| AC5 E2 allowlist + live probe → channel → persist → existing `GitActivityRate`; optional E3 | **Pass** |
| AC6 Docs touch `08`/`07`/`10`/`12`/`04`/`16` (not deferred) | **Pass** |
| AC7 Dev handoff exists | **Pass** |
| Global DoD (self-tracking; UI↛DB; LLM interpret-only; no PR; ADR-013 payload unchanged) | **Pass** |

- Extra checks: no `crates/` / `apps/` changes; ADR-013 deferral cross-linked to ADR-014; Feature formula rewrite explicitly not required.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P14-E1-T1** Done; Ready **P14-E2-T1** (no schema approve wait — ADR chose config file)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Optionally mirror: Epic P14-E1 ✅; `ARCHITECTURE_STATUS` / vision notes that ADR-014 locked config-file allowlist
- [ ] Other docs already touched by Dev for ADR notes — no extra schema approve needed

## Suggested next Ready task
- **P14-E2-T1** — Implement allowlist load (`~/.biofocus/git-watched-roots.toml`) + live `SystemGitActivityProbe` under roots → existing Observation channel → persist (payload unchanged). Branch: `phase/14-git-allowlist`.

## Notes for PM
- Locked names: `~/.biofocus/git-watched-roots.toml`, ADR-014, live probe in E2, optional Settings/IPC in E3.
- **No user schema approve** required before Ready E2 (non-SQLite v1).
- PR freeze still active until 2026-09-01 — do not open PR.
