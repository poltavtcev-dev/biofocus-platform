# PM Brief → Dev: P15-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** P15-E1-T1 (QA Pass — ADR-015); Epic **P15-E1** ✅  
**Evidence:** `docs/handoffs/P15-E1-T1-qa-to-pm.md`

## Task
**P15-E2-T1 — Implement ambient light plugin (ADR-015)**

## Why
ADR-015 locked Phase 15 v1 primary to **ambient light**: `data_type: "ambient_light"`, provider `com.biofocus.macos.ambient_light`, opt-in `BIOFOCUS_AMBIENT_LIGHT=1`, coarse `light_kind` + optional bounded `level` — existing `observations` only (**no** migration). Next: ship the `BioFocusPlugin` into the existing ingest channel so E3 can compute **`AmbientLightShare`** from real Observations — Feature math stays **P15-E3**.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.ambient_light` with Capability covering `ambient_light`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` / ADR-015.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_AMBIENT_LIGHT=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-015 / `docs/07-contracts.md`: required `light_kind` (`dark` \| `dim` \| `moderate` \| `bright` \| `unknown` — refine labels only if still closed-set + coarse) and optional `level` integer **0–100**. **Never** persist or log camera frames, screen contents/screenshots, precise geo, mic/waveform, or cloud light telemetry.
4. Soft-fail OS probe when mapping unavailable (idle; **no** emit / **no** busy-loop); injectable **scripted probe** for tests; emit on light-band change or rare poll ≥5s; `stop_stream` joins background work.
5. Docs finalized (shipped, not “planned only”): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development` as needed for the live contract / probe tables.
6. Unit and/or integration tests with scripted probe: emit → channel → persist path covered where practical; after `stop_stream`, probe/emission counts freeze; soft-fail / unavailable mapping → no busy-loop.
7. **No** Feature formula / `AmbientLightShare` wiring; **no** SQLite migration.
8. Handoff: `docs/handoffs/P15-E2-T1-dev-to-qa.md`.

## Out of scope
- `AmbientLightShare` / feature-engine DAG (→ **P15-E3-T1**)
- Weather ambient collector; IDE; App Store packaging; NotificationPressure
- Camera-based scene understanding; workplace / surveillance framing
- New SQLite schema / ambient registry (needs future ADR + approve)
- Plugin marketplace crate; always-on capture; cloud sync product
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-015 + contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel ambient marketplace crate
- Branch: `phase/15-ambient-light`
- Personal self-tracking only — not workplace / environmental surveillance
- LLM remains L5 interpret-only — must not invent light kinds or Features

## After QA Pass
PM → mark P15-E2-T1 Done; Ready **P15-E3-T1** (`AmbientLightShare`).
