# PM Brief → Dev: P12-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P12-E1-T1 (QA Pass — ADR-012); Epic **P12-E1** ✅  
**Evidence:** `docs/handoffs/P12-E1-T1-qa-to-pm.md`

## Task
**P12-E2-T1 — Implement Now Playing ambient plugin (ADR-012)**

## Why
ADR-012 locked Phase 12 primary track to **Now Playing / music**: `data_type: "now_playing"`, provider `com.biofocus.macos.now_playing`, opt-in `BIOFOCUS_NOW_PLAYING=1`, coarse `media_kind` + `is_playing` only. Next: ship the `BioFocusPlugin` into the existing ingest channel so E3 can compute **`AmbientMediaShare`** from real Observations — Feature DAG + packaging runbook stay **P12-E3**.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.now_playing` with Capability covering `now_playing`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` / ADR-012.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_NOW_PLAYING=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-012 / `docs/07-contracts.md`: required `media_kind` (`music` \| `podcast` \| `other` \| `none` \| `unknown` — refine labels only if still closed-set + coarse) and `is_playing` (bool). **Never** persist or log song/episode titles, artists, albums, lyrics, playlist IDs, mic/waveform, or precise geo.
4. Injectable probe for tests; production probe may soft-fail / emit nothing when OS mapping unavailable — must compile, idle safely (emit on play-state / media-kind change or rare ≥5s poll; **no** busy-loop); `stop_stream` joins background work.
5. Docs finalized (shipped, not “planned only”): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development` as needed for the live contract.
6. Unit and/or integration tests with mock probe: emit → channel → persist path covered where practical; after `stop_stream`, probe/emission counts freeze.
7. Handoff: `docs/handoffs/P12-E2-T1-dev-to-qa.md`.

## Out of scope
- `AmbientMediaShare` / feature-engine DAG (→ **P12-E3-T1**)
- Packaging installer binary / notarization automation (→ E3 runbook companion)
- Weather / light collectors; IDE/Git; NotificationPressure
- New SQLite schema / ambient allowlist / sync store (needs future ADR + approve)
- Plugin marketplace crate; always-on capture; cloud sync product
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-012 + contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel ambient marketplace crate
- Branch: `phase/12-ambient-packaging`
- Personal self-tracking only — not workplace / environmental surveillance
- LLM remains L5 interpret-only — must not invent media kinds or Features

## After QA Pass
PM → mark P12-E2-T1 Done; Ready **P12-E3-T1** (`AmbientMediaShare` + packaging runbook).
