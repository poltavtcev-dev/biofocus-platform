# PM Brief → Dev: P14-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P14-E1-T1 (QA Pass — ADR-014); Epic **P14-E1** ✅  
**Evidence:** `docs/handoffs/P14-E1-T1-qa-to-pm.md`

## Task
**P14-E2-T1 — Implement Git watched-roots allowlist + live probe (ADR-014)**

## Why
ADR-014 locked Phase 14 v1 allowlist to local config `~/.biofocus/git-watched-roots.toml` (absolute `roots` only; **no** SQLite / **no** migration). Next: wire load/validate + live `SystemGitActivityProbe` so opt-in `BIOFOCUS_GIT_ACTIVITY` can emit real ADR-013 `git_activity` Observations under user-chosen roots — existing **`GitActivityRate`** already consumes them.

## Acceptance Criteria
1. Load/validate `~/.biofocus/git-watched-roots.toml` (`version` + absolute `roots`); missing / empty / unreadable → soft-fail idle (**no** emit, **no** whole-disk scan). Prefer expand home at runtime.
2. Live `SystemGitActivityProbe` (or equivalent behind `GitActivityPlugin`) considers activity **only under allowlisted roots** (or nested repos discovered under a listed root); emit ADR-013 Observations (`activity_kind` + optional `event_count`) — **never** put root/repo paths, remotes, branch names, SHAs, messages, diffs, or authors into Observation payloads or default logs.
3. Desktop `ingest_host` still starts the plugin **only** when `BIOFOCUS_GIT_ACTIVITY=1`; same bounded Observation channel → persist; idle-safe poll (≥5s or on change; **no** busy-loop); `stop_stream` joins.
4. Keep `ScriptedGitActivityProbe` (or equivalent) for tests; unit/integration cover: empty allowlist → no emit; fixture roots + scripted/mock activity → emit → channel → persist where practical; after `stop_stream`, probe/emission freezes; forbidden payload keys still rejected/stripped on validate/normalize path.
5. Docs finalized as shipped (not “planned only”) for the live probe path: `07-contracts` / `08-plugin-sdk` / `10-security` / `12-development` as needed. Optional env override for tests/CI (`BIOFOCUS_GIT_WATCHED_ROOTS`) only if documented with clear precedence (file remains SoT when present).
6. **No** Feature formula rewrite for `GitActivityRate`; **no** SQLite migration / allowlist table.
7. Handoff: `docs/handoffs/P14-E2-T1-dev-to-qa.md`.

## Out of scope
- Settings / IPC UI to edit allowlist (→ **P14-E3-T1**)
- IDE collector; weather/light; App Store packaging; NotificationPressure
- Workplace / manager dashboards; whole-home scans
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-014 + ADR-013 contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `macos-collector` + host — no parallel marketplace crate
- Branch: `phase/14-git-allowlist`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only

## After QA Pass
PM → mark P14-E2-T1 Done; Ready **P14-E3-T1** (dogfood / optional allowlist UX).
