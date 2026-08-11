# PM Brief → Dev: P15-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** PM-GATE-POST-P14 (chose **ambient light** within weather/light track); Phase 14 complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P14-pm-brief.md` · `docs/handoffs/P14-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 15 Ambient light (Sprint 29–30) — `docs/SPRINT_ROADMAP.md`

## Task
**P15-E1-T1 — ADR-015: Ambient light Observation contract + Phase 15 scope**

## Why
Phase 12 shipped Now Playing ambient; ADR-012 deferred weather/light. PM gate post–P14 chose the **weather/light** track and locks v1 primary to **ambient light** (Local-First on-device; no cloud weather API / precise geo in this phase). Need an ADR before a collector: coarse payload, opt-in, Capability Plugin Model, and E3 Feature name — without inventing Features without Observation inputs.

## Acceptance Criteria
1. Record **ADR-015** in `docs/decision-log.md`: Phase 15 v1 primary = **ambient light** (exactly one ambient source this phase — **not** weather in the same wave). Rationale vs ADR-012 deferral + Local-First (prefer on-device light over network weather). Capability Plugin Model fit; personal self-tracking only.
2. Observation contract sketch: `data_type` (e.g. `"ambient_light"`), privacy-safe payload (coarse closed-set label and/or bounded level — **no** camera frames, screen contents, precise geolocation, or always-on mic), `provider_id`, opt-in env default **off**, emit on change / rare poll ≥5s, no busy-loop.
3. Rejected alternatives documented (weather as P15 primary when cloud/geo implied; IDE as P15 primary; App Store packaging product as P15 primary; NotificationPressure without notification Observations; always-on capture; cloud light telemetry; parallel marketplace crate; parallel SQLite ambient registry without need; PR during freeze; applying migration without user approve).
4. Schema: prefer existing `observations` store only — **no** migration unless ADR proves need; if schema proposed, sketch only — **do not apply** until user approve.
5. Short sketch: E2 = ambient light plugin → Observation channel → persist; E3 = catalog Feature name locked (e.g. calm `AmbientLightShare` / equivalent — ADR finalizes) + ADR-007 confidence; calm non-clinical framing; distinct from `AmbientMediaShare` / `GitActivityRate`.
6. Docs touch: `08-plugin-sdk` / `07-contracts` / `10-security` / `12-development` / `16-glossary` / `06-feature-catalog` / `04-storage` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2/E3 (must be stated).
7. Handoff: `docs/handoffs/P15-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing the collector plugin (→ **P15-E2**)
- Catalog Feature math beyond ADR sketch (→ **P15-E3**)
- Weather ambient collector; IDE; App Store packaging product; NotificationPressure
- Workplace / surveillance framing; camera-based scene understanding
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `macos-collector` + host — no parallel marketplace crate
- Branch: `phase/15-ambient-light`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only
- Features only with real Observation inputs
- **PM lean (non-binding for ADR):** coarse `light_kind` / level bands suitable for soft-fail OS probe; scripted probe for tests; soft-fail idle when mapping unavailable.

## After QA Pass
PM → mark P15-E1-T1 Done; if ADR requires schema approve, wait for user before Ready **P15-E2-T1**; else Ready **P15-E2-T1** shaped by ADR-015.
