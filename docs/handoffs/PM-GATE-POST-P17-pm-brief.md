# PM Brief → PM: PM-GATE-POST-P17 (closed)

**From:** PM (close of Phase 17 wearable + charts)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **NotificationPressure** → Phase 18 opened; Ready **P18-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 17 E1–E3 (ADR-018 → Companion emit → `get_feature_series` + wearable Features)  
**Evidence:** `docs/handoffs/P17-E3-T1-qa-to-pm.md` · `docs/handoffs/P17-E*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, dogfood-polish-as-primary. Next brief: `docs/handoffs/P18-E1-T1-pm-brief.md`.  
**Branch note:** Phase 17 cluster stays on `phase/17-wearable-charts` (PR after freeze). Phase 18 on `phase/18-notification-pressure`.

## Task
**PM-GATE-POST-P17 — Choose next Phase 18+ slice**

## Decision (locked)
**Primary Phase 18 track = NotificationPressure** — lock notification Observation family (**ADR-019**) → opt-in macOS collector → catalog Feature `NotificationPressure`. Coarse counts/cadence only; **no** body/title/content.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| **NotificationPressure** | **Chosen** — catalog backlog since P10; focus/interruption dogfood; needs Observation ADR first |
| Dogfood polish | Parallel OK (Companion physical-device) — not Phase 18 primary theme |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 18 v1 track — **done** (NotificationPressure).
2. Open Phase 18 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P18-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 18 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 17 on `phase/17-wearable-charts`; Phase 18 on `phase/18-notification-pressure`.

## After gate
Paste **build-qa** command for **P18-E1-T1**.
