# PM Brief → PM: PM-GATE-POST-P20 (closed)

**From:** PM (close of Phase 20 CognitiveLoad)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **`DeepWorkScore`** → Phase 21 opened; Ready **P21-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 20 E1–E3 (ADR-021 → `CognitiveLoad` Feature → dogfood + Combined demand chart)  
**Evidence:** `docs/handoffs/P20-E3-T1-qa-to-pm.md` · `docs/handoffs/P20-*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, Companion polish-as-primary, CircadianOffset-as-primary, AttentionStability-as-primary. Next brief: `docs/handoffs/P21-E1-T1-pm-brief.md`.  
**Branch note:** Phase 20 cluster stays on `phase/20-cognitive-load` (PR after freeze). Phase 21 on `phase/21-deep-work-score`.

## Task
**PM-GATE-POST-P20 — Choose next Phase 21+ slice**

## Decision (locked)
**Primary Phase 21 track = `DeepWorkScore`** — lock sustained-focus catalog Feature from existing Feature inputs (**FocusScore** + **ContextSwitchRate**) via **ADR-022** → ship in `feature-engine`. No new Observation family required. Calm non-clinical framing only (“sustained focus in this window” — not flow/burnout diagnosis). Complements shipped `CognitiveLoad` (demand) with a focus-quality composite.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| Companion polish (as primary) | Parallel OK — not Phase 21 theme |
| CircadianOffset | Deferred — sleep/activity exist, but chronotype framing is riskier; prefer Focus-ladder composite first |
| AttentionStability | Deferred — sibling backlog; DeepWorkScore is the clearer product name / P7-era candidate |
| **`DeepWorkScore`** | **Chosen** — catalog backlog; FocusScore + CSR already shipped; same Feature-level pattern as CognitiveLoad |
| Other | Not needed |

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 21 v1 track — **done** (`DeepWorkScore`).
2. Open Phase 21 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P21-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 21 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 20 on `phase/20-cognitive-load`; Phase 21 on `phase/21-deep-work-score`.

## After gate
Paste **build-qa** command for **P21-E1-T1**.
