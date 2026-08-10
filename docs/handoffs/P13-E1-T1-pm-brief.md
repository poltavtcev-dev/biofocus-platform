# PM Brief → Dev: P13-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** PM-GATE-POST-P12 (chose IDE/Git plugin wave-2); Phase 12 complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P12-pm-brief.md` · `docs/handoffs/P12-E3-T1-qa-to-pm.md`  
**Phase opened:** Phase 13 Plugin wave-2 (IDE/Git) (Sprint 25–26) — `docs/SPRINT_ROADMAP.md`

## Task
**P13-E1-T1 — ADR-013: Plugin wave-2 scope (IDE or Git) + Observation contract**

## Why
Vision source priority is Wearables → Life Events/Calendar → **IDE/Git/Browser** → ambient (`docs/00-vision.md` §4). Phase 10 shipped Browser → `DistractionScore` and explicitly deferred IDE/Git (ADR-010). Phase 12 closed ambient music + packaging runbook. PM gate post–P12 chose **plugin wave-2 (IDE/Git)** over weather/light and App Store packaging product — finish the plugin ladder with a privacy-first, opt-in Capability Plugin before more ambient or commercial product work.

## Acceptance Criteria
1. Record **ADR-013** in `docs/decision-log.md`: choose **exactly one** wave-2 v1 primary source — **IDE** *or* **Git** (not both in the same wave). Rationale vs ADR-010 deferral notes (IDE already visible in `context_window` / CSR; Git needs careful path/remote scoping). How it fits Capability Plugin Model.
2. Observation contract sketch: `data_type`(s), privacy-safe payload (coarse labels / aggregates only), `provider_id`, opt-in env (default **off**), poll/event + idle posture (no busy-loop). Explicitly forbid: source file paths / buffer contents / keystroke content / full repo remotes / diffs / employee-surveillance framing — unless a carefully scoped aggregate is justified and named.
3. Rejected alternatives documented (IDE+Git in same wave; always-on capture; cloud history sync; plugin marketplace crate; parallel plugin SQLite registry; weather/light or App Store product as Phase 13 primary; NotificationPressure as this wave’s Feature unless justified; PR during freeze).
4. If schema is proposed: sketch only — **do not apply migration** until user approve. Prefer existing `observations` store (ADR-006 / ADR-010 / ADR-012 stance).
5. Short sketch: E2 plugin → shared Observation channel → persist; E3 catalog Feature name + calm non-clinical framing + ADR-007 confidence (name locked by this ADR; may be new Planned→§1 Feature or a justified extension — **not** redefining Browser `DistractionScore` math unless ADR explicitly argues merge).
6. Docs touch: `08-plugin-sdk` / `07-contracts` / `10-security` / `12-development` / `16-glossary` / `06-feature-catalog` as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2/E3 (must be stated).
7. Handoff: `docs/handoffs/P13-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing the collector plugin (→ **P13-E2**)
- Catalog Feature / pipeline normalize beyond ADR sketch (→ **P13-E3**)
- Weather / light ambient; App Store packaging product; sync product
- Workplace / manager dashboards; full-text indexing of repos
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `plugin-sdk` + `macos-collector` (or thin adapter) — no parallel plugin marketplace crate without ADR justification
- Branch: `phase/13-plugin-wave-2`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only — must not invent payloads or Feature formulas
- **PM lean (non-binding for ADR):** prefer **IDE** as wave-2 v1 **only if** the contract adds signal beyond bare `context_window` bundle_id (e.g. coarse IDE/session kind without file contents). If ADR finds no additive privacy-safe signal, choose **Git activity aggregates** (counts / cadence only — no paths, remotes, diffs) instead. Still **one** source.

## After QA Pass
PM → mark P13-E1-T1 Done; if ADR requires schema approve, wait for user; else Ready **P13-E2-T1** shaped by ADR-013.
