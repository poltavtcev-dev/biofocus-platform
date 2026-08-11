# PM Brief → PM: PM-GATE-POST-P18 (closed)

**From:** PM (close of Phase 18 notification pressure)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **Live NC OS mapping** → Phase 19 opened; Ready **P19-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 18 E1–E3 (ADR-019 → collector → `NotificationPressure`)  
**Evidence:** `docs/handoffs/P18-E3-T1-qa-to-pm.md` · `docs/handoffs/P18-E*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, `CognitiveLoad`-as-primary, dogfood-polish-as-primary. Next brief: `docs/handoffs/P19-E1-T1-pm-brief.md`.  
**Branch note:** Phase 18 cluster stays on `phase/18-notification-pressure` (PR after freeze). Phase 19 on `phase/19-live-nc-mapping`.

## Task
**PM-GATE-POST-P18 — Choose next Phase 19+ slice**

## Decision (locked)
**Primary Phase 19 track = Live NC OS mapping** — unlock privacy-safe live emits from `SystemNotificationEventProbe` so shipped `NotificationPressure` can dogfood on real macOS Notification Center metadata (still coarse counts/labels only; **no** body/title/content). Pattern mirrors Phase 14 (allowlist unlock after Git soft-fail).

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| Dogfood polish (as primary) | Parallel OK (Companion physical-device) — not Phase 19 theme |
| `CognitiveLoad` | Deferred — catalog backlog; prefer live notification inputs before composite Feature |
| **Live NC OS mapping** | **Chosen** — closes ADR-019 soft-fail gap; unlocks shipped Feature dogfood |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 19 v1 track — **done** (Live NC OS mapping).
2. Open Phase 19 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P19-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 19 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 18 on `phase/18-notification-pressure`; Phase 19 on `phase/19-live-nc-mapping`.

## After gate
Paste **build-qa** command for **P19-E1-T1**.
