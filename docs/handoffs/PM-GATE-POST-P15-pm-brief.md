# PM Brief → PM: PM-GATE-POST-P15

**From:** PM (close of Phase 15 companion)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **resume ambient light** (ADR-015) → Phase 16 opened; Ready **P16-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 15 companion HRV + autonomy (E1–E3); ADR-016  
**Evidence:** `docs/handoffs/P15-E3-T1-qa-to-pm.md` · `docs/handoffs/P15-E*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather-as-primary, App Store packaging product, NotificationPressure. Next brief: `docs/handoffs/P16-E1-T1-pm-brief.md`.  
**Branch note:** Phase 15 cluster stays on `phase/15-companion-hrv-autonomy` (PR after freeze). Phase 16 on `phase/16-ambient-light`.

## Task
**PM-GATE-POST-P15 — Choose next Phase 16+ slice**

## Why
Phase 0–15 ladder is complete through autonomous companion HR+HRV (ADR-016). Vision says open later slices via PM gate. ADR-015 ambient light contract was parked during Priority A companion wave — highest-readiness unlock is to **resume** that Local-First collector + Feature.

## Decision (locked)
**Primary Phase 16 track = resume ambient light (ADR-015 SoT)** — implement collector + `AmbientLightShare`. No new Observation contract ADR unless Dev finds a justified change (then ADR-017 + approve). Weather remains deferred.

| Candidate | Verdict |
| :--- | :--- |
| **Resume ambient light (ADR-015)** | **Chosen** — contract locked; Local-First; continues ambient ladder after Now Playing |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` without a new contract |
| Weather ambient | Deferred — cloud / geo implications vs Local-First bar |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| NotificationPressure | Deferred — needs notification Observation family first |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 16 v1 track — **done** (resume ambient light).
2. Open Phase 16 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P16-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 16 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 15 on `phase/15-companion-hrv-autonomy`; Phase 16 on `phase/16-ambient-light`.

## Out of scope
- Implementing collectors / Features in this gate chat
- Opening a PR
- Shipping multiple unrelated tracks in one phase without ADR justification

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Local-First · Capability Plugin Model · calm non-clinical copy
- Features only with real Observation inputs
- New SQLite schema only via ADR + user approve

## After gate
Paste **build-qa** command for **P16-E1-T1**.
