# PM Brief → Dev: P14-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** PM-GATE-POST-P13 (chose **Git path-allowlist**); Phase 13 complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P13-pm-brief.md` · `docs/handoffs/P13-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 14 Git path-allowlist / live probe (Sprint 27–28) — `docs/SPRINT_ROADMAP.md`

## Task
**P14-E1-T1 — ADR-014: Git watched-roots / path-allowlist + live probe boundaries**

## Why
Phase 13 shipped opt-in `git_activity` Observations + catalog Feature **`GitActivityRate`**, but production `SystemGitActivityProbe` intentionally **soft-fails idle** — ADR-013 deferred any persisted watched-roots allowlist. Dogfood cannot see real VCS cadence without a privacy-scoped allowlist. PM gate post–P13 chose this slice over IDE, weather/light, App Store packaging product, and NotificationPressure: unlock the already-shipped Feature with the smallest architecture-sensitive step.

## Acceptance Criteria
1. Record **ADR-014** in `docs/decision-log.md`: how personal watched roots are declared for Git activity probing (choose **one** v1 storage stance — e.g. env list, local config file under `~/.biofocus/`, **or** SQLite table). Rationale vs ADR-013 soft-fail; Capability Plugin Model fit; personal self-tracking only.
2. Privacy contract: allowlist stores **user-chosen root paths only** as configuration — still **never** persist repo paths / remotes / branch names / SHAs / messages / diffs / authors into `git_activity` Observation payloads. Clarify what may appear in logs (prefer Observation `id` / counts only).
3. Rejected alternatives documented (always-on whole-disk scan; workplace / manager dashboards; cloud git history sync; IDE wave as Phase 14 primary when allowlist unfinished; weather/light or App Store as this phase primary; NotificationPressure without notification Observations; PR during freeze; applying migration without user approve).
4. If SQLite schema is proposed: sketch columns / migration **only** — **do not apply** until user approve. Prefer non-SQLite config if ADR finds it sufficient for v1.
5. Short sketch: E2 implements allowlist + live `SystemGitActivityProbe` → existing Observation channel → persist → existing `GitActivityRate` (no Feature formula rewrite unless ADR finds a justified confidence tweak). Optional E3: Settings / IPC to edit allowlist, or dogfood gate notes.
6. Docs touch: `08-plugin-sdk` / `07-contracts` / `10-security` / `12-development` / `04-storage` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P14-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing the allowlist store or live probe (→ **P14-E2**)
- Rewriting `GitActivityRate` math (Feature stays; only unlock inputs)
- IDE collector; weather/light ambient; App Store packaging product; NotificationPressure
- Workplace / manager dashboards; full-text indexing of repos
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-013 contract stays: coarse `activity_kind` + optional `event_count` only
- Prefer extend `macos-collector` + host — no parallel marketplace crate
- Branch: `phase/14-git-allowlist`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only
- **PM lean (non-binding for ADR):** prefer **config file or env allowlist** for v1 if durable enough for dogfood; choose SQLite only if ADR needs CRUD + Settings later. Either way, **no migration applied in E1**.

## After QA Pass
PM → mark P14-E1-T1 Done; if ADR requires schema approve, wait for user before Ready **P14-E2-T1**; else Ready **P14-E2-T1** shaped by ADR-014.
