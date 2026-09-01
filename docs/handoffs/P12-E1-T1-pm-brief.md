# PM Brief → Dev: P12-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P11-E3-T1 (QA Pass with notes — provider UX + pack-aware Report); Phase 11 complete  
**Evidence:** `docs/handoffs/P11-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 12 Ambient + commercial packaging (Sprint 23–24) — `docs/SPRINT_ROADMAP.md`

## Task
**P12-E1-T1 — ADR-012: Phase 12 scope (ambient sources + commercial packaging)**

## Why
Vision ladder Phase **12+** is *Ambient sources + commercial packaging* (`docs/00-vision.md` §4 / §7 · `docs/14-roadmap.md`). Source priority after plugins is **ambient** (music / weather / light). Commercial split is **packaging** (signed builds, updates, optional user-opt-in sync, support) — algorithms stay open-source; not secret Core math. Phase 11 closed L5 coaching polish. Before any ambient collector, new `data_type`, Feature, or packaging pipeline, Phase 12 needs an **ADR** that picks the v1 slice and sequencing so dogfood stays Local-First, opt-in, and Capability-Model aligned.

## Acceptance Criteria
1. Record **ADR-012** in `docs/decision-log.md`: decide Phase 12 **v1 primary track** and sequencing between (a) **ambient Observation source** (pick **one** of music / weather / light for wave-1) and (b) **commercial packaging** (signed builds / update channel / optional sync stance). Rationale: vision source ladder, dogfood feasibility on macOS, privacy, idle footprint.
2. If ambient is in v1: Observation contract sketch — `data_type`(s), privacy-safe payload (coarse labels / aggregates only; no song lyrics, full playlists, precise home geolocation dumps, or always-on mic), `provider_id`, opt-in default **off**, poll/event + idle posture. Prefer existing `observations` store (ADR-006/010 stance).
3. Commercial packaging boundaries: what is in Phase 12 v1 vs deferred (e.g. notarization/signing runbook vs full App Store; **optional** sync must stay opt-in and off by default; no mandatory cloud telemetry). Reaffirm AGPLv3 Core stays open; commercial ≠ closed metric math.
4. Rejected alternatives documented (shipping both music+weather+light in one wave; always-on ambient capture; cloud sync by default; secret/proprietary Feature formulas; ambient Features without Observation inputs; packaging that forces UI→DB or cloud LLM; opening a PR during freeze).
5. If schema / new persistence / sync store is proposed: sketch only — **do not apply migration** until user approve.
6. Short sketch: E2 first implementation slice → E3 dogfood Feature **or** packaging UX/runbook (names locked by this ADR).
7. Docs touch: `08-plugin-sdk` / `07-contracts` / `10-security` / `12-development` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2/E3 (must be stated).
8. Handoff: `docs/handoffs/P12-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing ambient collector / Feature code (→ **P12-E2** / **P12-E3** per ADR)
- Shipping signed installers or sync product (→ later epic per ADR; E1 = decision + boundaries only)
- IDE/Git plugin wave (deferred from ADR-010 — not Phase 12 unless ADR explicitly re-prioritizes with rationale)
- Cloud LLM marketplace; chat-history SQLite; clinical ambient framing
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer Capability Plugin Model + existing ingest/observations — no parallel ambient marketplace crate without ADR justification
- Branch: `phase/12-ambient-packaging`
- Personal self-tracking only — not workplace / environmental surveillance framing
- LLM remains L5 interpret-only — must not invent ambient Features or packaging policy
- **PM lean (non-binding for ADR):** ambient wave-1 first (one source), packaging as signed-build/update **runbook + process** in same phase without default cloud sync

## After QA Pass
PM → mark P12-E1-T1 Done; if ADR requires schema/sync approve, wait for user; else Ready **P12-E2-T1** shaped by ADR-012.
