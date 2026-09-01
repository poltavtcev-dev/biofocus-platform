# PM Brief → Dev: P10-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P9-E3-T1 (QA Pass — Recommendations IPC / UX); Phase 9 complete  
**Evidence:** `docs/handoffs/P9-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 10 Plugin wave-1 (Sprint 19–20) — `docs/SPRINT_ROADMAP.md`

## Task
**P10-E1-T1 — ADR-010: Plugin wave-1 source + Observation contract**

## Why
Vision source priority after Calendar is **IDE/Git / Browser** (`docs/00-vision.md` §4 / §7). Phase 9 closed L4 Recommendations; wave-1 needs an **ADR** before a new collector, `data_type`, or Feature — so dogfood stays privacy-first, opt-in, and Capability-Model aligned (`docs/08-plugin-sdk.md`). Catalog already anticipates `DistractionScore` (browser categories) and related P10 Features (`docs/06-feature-catalog.md` § Planned).

## Acceptance Criteria
1. Record **ADR-010** in `docs/decision-log.md`: choose **one** wave-1 source for v1 — **IDE/Git** *or* **Browser categories**; rationale (dogfood, reuse of `context_window` / CSR, catalog readiness); how it fits Capability Plugin Model.
2. Observation contract sketch: `data_type`(s), privacy-safe payload (coarse categories / labels only), `provider_id`, opt-in env (default off), poll/event + idle posture (no busy-loop). Explicitly forbid (unless carefully scoped and justified) full URLs, keystroke/content capture, and employee-surveillance framing.
3. Rejected alternatives documented (both IDE+Browser in same wave; always-on capture; cloud history sync; plugin marketplace; parallel plugin SQLite registry without need; ambient music/weather in P10).
4. If schema is proposed: sketch only — **do not apply migration** until user approve. Prefer existing `observations` store (ADR-006 stance).
5. Short sketch: E2 plugin → shared Observation channel → persist; E3 Feature path (`DistractionScore` if Browser; IDE-aligned catalog name if IDE/Git) with calm non-clinical framing + ADR-007 confidence.
6. Docs touch: `08-plugin-sdk` / `07-contracts` / `16-glossary` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P10-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing the collector plugin (→ **P10-E2**)
- Catalog Feature / pipeline normalize beyond ADR sketch (→ **P10-E3**)
- NotificationPressure (unless ADR notes a deferral)
- AI coaching polish (Phase 11); ambient sources (Phase 12+)
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `plugin-sdk` + `macos-collector` (or thin adapter) — no parallel plugin marketplace crate without ADR justification
- Branch: `phase/10-plugin-wave-1`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only — must not define plugin payloads or Features

## After QA Pass
PM → mark P10-E1-T1 Done; if ADR requires schema approve, wait for user; else Ready **P10-E2-T1**.
