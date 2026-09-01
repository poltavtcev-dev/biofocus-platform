# PM Brief → PM: PM-GATE-POST-P14

**From:** PM (close of Phase 14)  
**To:** PM (this gate)  
**Status:** Done (2026-08-10) — chose **ambient light** (weather/light track) → Phase 15 opened; Ready **P15-E1-T1**  
**Date:** 2026-08-10  
**Closed previous:** P14-E3-T1 (QA Pass — dogfood + Git folders Settings/IPC); Epic **P14-E3** ✅; **Phase 14** ✅  
**Evidence:** `docs/handoffs/P14-E3-T1-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather-as-primary, App Store packaging product, NotificationPressure. Next brief: `docs/handoffs/P15-E1-T1-pm-brief.md`.  
**Branch note:** Phase 14 cluster stays on `phase/14-git-allowlist` (PR after freeze). Phase 15 on `phase/15-ambient-light`.

## Task
**PM-GATE-POST-P14 — Choose next Phase 15+ slice**

## Why
Phase 0–14 ladder is complete through live Git allowlist (`git-watched-roots.toml` + `SystemGitActivityProbe` + Menubar **Git folders** / dogfood). Vision (`docs/00-vision.md`) says open later slices via PM gate, not all at once.

## Decision (locked)
**Primary Phase 15 track = weather/light ambient → v1 primary = ambient light (ADR-015)** — continue the ambient ladder after Now Playing with a Local-First, on-device source (no cloud weather API / precise geo in this phase). Weather remains deferred as a later ambient slice.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred again — still no additive privacy-safe signal beyond `context_window` without a new contract |
| **Weather / light ambient** | **Chosen track** — v1 locks **ambient light** (Local-First); weather deferred within phase |
| App Store packaging product | Deferred — commercial track beyond P12 runbook; not dogfood Observation unlock |
| NotificationPressure | Deferred — needs notification Observation family first |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 15 v1 track — **done** (ambient light within weather/light).
2. Open Phase 15 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P15-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 15 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 14 cluster stays on `phase/14-git-allowlist`; Phase 15 on `phase/15-ambient-light`.

## Out of scope
- Implementing collectors / Features / installers / App Store submissions in this gate chat
- Opening a PR
- Shipping multiple unrelated tracks in one phase without ADR justification
- Widening `git_activity` Observation payloads or adding SQLite allowlist without a new ADR + approve

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Local-First · Capability Plugin Model · calm non-clinical copy
- Features only with real Observation inputs
- Commercial packaging ≠ closed Core Feature math
- New SQLite schema only via ADR + user approve

## After gate
Paste **build-qa** command for **P15-E1-T1**.
