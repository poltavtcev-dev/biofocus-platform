# PM Brief → Dev: P16-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** PM-GATE-POST-P15 (chose **resume ambient light** within ADR-015); Phase 15 companion complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P15-pm-brief.md` · `docs/handoffs/P15-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 16 Ambient light (Sprint 31–32) — `docs/SPRINT_ROADMAP.md`

## Task
**P16-E1-T1 — Implement ambient light plugin (ADR-015)**

## Why
ADR-015 already locked Phase ambient-light contract (`data_type: "ambient_light"`, provider `com.biofocus.macos.ambient_light`, opt-in `BIOFOCUS_AMBIENT_LIGHT=1`, coarse `light_kind` + optional bounded `level`). Phase 15 parked the collector for companion HRV (ADR-016). PM-GATE-POST-P15 resumes that track — ship the `BioFocusPlugin` into the existing ingest channel so E2 can compute **`AmbientLightShare`** from real Observations.

## Acceptance Criteria
1. `BioFocusPlugin` id `com.biofocus.macos.ambient_light` with Capability covering `ambient_light`; implement in `crates/macos-collector` (or thin adapter behind `plugin-sdk`) per `docs/08-plugin-sdk.md` / ADR-015.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_AMBIENT_LIGHT=1` (default **off**); same bounded Observation channel → persist worker (no UI→SQLite).
3. Emitted Observations match ADR-015 / `docs/07-contracts.md`: required `light_kind` (`dark` \| `dim` \| `moderate` \| `bright` \| `unknown` — refine labels only if still closed-set + coarse) and optional `level` integer **0–100**. **Never** persist or log camera frames, screen contents/screenshots, precise geo, mic/waveform, or cloud light telemetry.
4. Soft-fail OS probe when mapping unavailable (idle; **no** emit / **no** busy-loop); injectable **scripted probe** for tests; emit on light-band change or rare poll ≥5s; `stop_stream` joins background work.
5. Docs finalized as **shipped** (not parked-only): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary` as needed for the live contract / probe tables. Mark Phase 16 collector active; Feature still → P16-E2.
6. Unit and/or integration tests with scripted probe: emit → channel → persist path covered where practical; after `stop_stream`, probe/emission counts freeze; soft-fail / unavailable mapping → no busy-loop.
7. **No** Feature formula / `AmbientLightShare` wiring; **no** SQLite migration; **no** new ADR unless contract must change (then stop and propose a **new** ADR + user approve — do **not** reuse ADR-017, which parks Phase 17 sequencing).
8. Handoff: `docs/handoffs/P16-E1-T1-dev-to-qa.md`.

## Out of scope
- `AmbientLightShare` / feature-engine DAG (→ **P16-E2-T1**)
- Phase 17 wearable depth / chart ranges (ADR-017 parked — after P16)
- Weather ambient collector; IDE; App Store packaging; NotificationPressure
- Camera-based scene understanding; workplace / surveillance framing
- New SQLite schema / ambient registry (needs future ADR + approve)
- Plugin marketplace crate; always-on capture; cloud sync product
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-015 + contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `plugin-sdk` + `macos-collector` — no parallel ambient marketplace crate
- Branch: `phase/16-ambient-light`
- Personal self-tracking only — not workplace / environmental surveillance
- LLM remains L5 interpret-only — must not invent light kinds or Features

## After QA Pass
PM → mark P16-E1-T1 Done; Ready **P16-E2-T1** (`AmbientLightShare`).
