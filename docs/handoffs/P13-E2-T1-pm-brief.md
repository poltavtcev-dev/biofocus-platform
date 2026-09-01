# PM Brief → Dev: P13-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P13-E1-T1 (QA Pass — ADR-013); Epic **P13-E1** ✅  
**Evidence:** `docs/handoffs/P13-E1-T1-qa-to-pm.md`

## Task
**P13-E2-T1 — Implement Git activity plugin (ADR-013)**

## Why
ADR-013 locked Phase 13 wave-2 to **Git activity aggregates** (not IDE): `data_type: "git_activity"`, provider `com.biofocus.macos.git`, opt-in `BIOFOCUS_GIT_ACTIVITY=1`, coarse `activity_kind` + optional `event_count` only. Next: ship the `BioFocusPlugin` into the existing ingest channel so E3 can compute **`GitActivityRate`** from real Observations — Feature DAG stays **P13-E3**.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.git` with Capability covering `git_activity`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` / ADR-013.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_GIT_ACTIVITY=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-013 / `docs/07-contracts.md`: required `activity_kind` (`commit` \| `checkout` \| `sync` \| `other` \| `idle` \| `unknown` — refine only if still closed-set + coarse) and optional `event_count` (≥ 1 aggregate). **Never** persist or log repo paths, remotes/URLs, branch names, SHAs, commit messages, diffs, authors, or file-change lists.
4. Injectable / scripted probe for tests; production probe may soft-fail / emit nothing when mapping unavailable — must compile, idle safely (emit on activity change or rare ≥5s poll; **no** busy-loop); `stop_stream` joins background work. **No** persisted path-allowlist table (deferred — soft-fail without it).
5. Docs finalized (shipped, not “planned only”): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development` as needed for the live contract; validation helper if peer plugins have one (e.g. `validate_git_activity_payload`).
6. Unit and/or integration tests with mock probe: emit → channel → persist path covered where practical; after `stop_stream`, probe/emission counts freeze; forbidden keys rejected or stripped on validate/normalize path as appropriate.
7. Handoff: `docs/handoffs/P13-E2-T1-dev-to-qa.md`.

## Out of scope
- `GitActivityRate` / feature-engine DAG (→ **P13-E3-T1**)
- IDE collector / session kinds; NotificationPressure
- Weather / light; App Store packaging product; sync product
- New SQLite schema / watched-roots allowlist / plugin registry table (needs future ADR + approve)
- Plugin marketplace crate; always-on capture; cloud git history sync
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-013 + contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel marketplace crate
- Branch: `phase/13-plugin-wave-2`
- Personal self-tracking only — not workplace / manager dashboards
- LLM remains L5 interpret-only — must not invent activity kinds or Features

## After QA Pass
PM → mark P13-E2-T1 Done; Ready **P13-E3-T1** (`GitActivityRate`).
