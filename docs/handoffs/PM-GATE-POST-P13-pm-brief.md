# PM Brief → PM: PM-GATE-POST-P13

**From:** PM (close of Phase 13)  
**To:** PM (this gate)  
**Status:** Done (2026-08-10) — chose **Git path-allowlist** → Phase 14 opened; Ready **P14-E1-T1**  
**Date:** 2026-08-10  
**Closed previous:** P13-E3-T1 (QA Pass — `GitActivityRate`); Epic **P13-E3** ✅; **Phase 13** ✅  
**Evidence:** `docs/handoffs/P13-E3-T1-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather/light, App Store packaging product, NotificationPressure. Next brief: `docs/handoffs/P14-E1-T1-pm-brief.md`.

## Task
**PM-GATE-POST-P13 — Choose next Phase 14+ slice**

## Why
Phase 0–13 ladder is complete through Browser `DistractionScore`, ambient Now Playing / packaging, and Git `GitActivityRate`. Vision (`docs/00-vision.md`) says open later slices via PM gate, not all at once. Do **not** invent a full Phase 14 backlog without an explicit scope ADR / Ready brief.

## Decision (locked)
**Primary Phase 14 track = Git path-allowlist / watched-roots ADR (ADR-014)** — unlock live `SystemGitActivityProbe` for already-shipped `git_activity` / `GitActivityRate` without widening privacy surface of Observation payloads.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred again — ADR-013 still finds no additive privacy-safe signal beyond `context_window` |
| Weather / light ambient | Deferred — valuable ambient ladder, but does not unblock shipped Git Feature |
| App Store packaging product | Deferred — commercial track beyond P12 runbook; not dogfood Feature unlock |
| **Git path-allowlist ADR** | **Chosen** — closes intentional soft-fail; schema/config sensitive → E1 ADR |
| NotificationPressure | Deferred — needs notification Observation family first |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 14 v1 track — **done** (Git path-allowlist).
2. Open Phase 14 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P14-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 14 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 13 cluster stays on `phase/13-plugin-wave-2`; Phase 14 on `phase/14-git-allowlist`.

## Out of scope
- Implementing collectors / Features / installers / allowlist migrations in this gate chat
- Opening a PR
- Shipping multiple unrelated tracks in one phase without ADR justification

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Local-First · Capability Plugin Model · calm non-clinical copy
- Features only with real Observation inputs
- Commercial packaging ≠ closed Core Feature math
- New SQLite schema only via ADR + user approve

## After gate
Paste **build-qa** command for **P14-E1-T1**.
