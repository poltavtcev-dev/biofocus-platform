# PM Brief → Dev: P10-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P10-E1-T1 (QA Pass — ADR-010 Browser categories); Epic **P10-E1** ✅  
**Evidence:** `docs/handoffs/P10-E1-T1-qa-to-pm.md`

## Task
**P10-E2-T1 — Implement Browser categories collector plugin (ADR-010)**

## Why
ADR-010 locked wave-1 to **Browser categories**: `data_type: "browser_category"`, provider `com.biofocus.macos.browser`, opt-in `BIOFOCUS_BROWSER_CATEGORIES=1`, coarse payload only. Next: ship the `BioFocusPlugin` into the existing ingest channel so E3 can compute **`DistractionScore`** from real Observations — Feature DAG stays **P10-E3**.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.browser` with Capability covering `browser_category`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` §5 / ADR-010.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_BROWSER_CATEGORIES=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-010 / `docs/07-contracts.md`: required coarse `category` (`work` \| `communication` \| `entertainment` \| `reference` \| `shopping` \| `unknown` — refine labels only if still closed-set + coarse); optional `browser_bundle_id`. **Never** persist or log full URLs, page titles, keystroke/content.
4. Injectable probe for tests; production probe may soft-fail / emit nothing when OS mapping unavailable — must compile, idle safely (emit on change or rare ≥5s poll; **no** busy-loop); `stop_stream` joins background work.
5. Docs finalized (shipped, not “planned only”): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`.
6. Unit and/or integration tests with mock probe: emit → channel → persist path covered where practical; after `stop_stream`, probe/emission counts freeze.
7. Handoff: `docs/handoffs/P10-E2-T1-dev-to-qa.md`.

## Out of scope
- `DistractionScore` / feature-engine DAG (→ **P10-E3-T1**)
- IDE/Git collectors; NotificationPressure; ambient sources
- New SQLite schema / allowlist config table (needs future ADR + approve)
- Plugin marketplace crate; always-on capture; cloud history sync
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-010 + contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel marketplace crate
- Branch: `phase/10-plugin-wave-1`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only — must not define categories or Features

## After QA Pass
PM → mark P10-E2-T1 Done; Ready **P10-E3-T1** (`DistractionScore`).
