# PM Brief → PM: PM-GATE-POST-P24 (closed)

**From:** PM (close of Phase 24 CircadianOffset)  
**To:** PM (this gate)  
**Status:** Done (2026-08-12) — chose **`SustainedLoadIndicator`** → Phase 25 opened; Ready **P25-E1-T1**  
**Date:** 2026-08-12  
**Closed previous:** Phase 24 E1–E3 (**ADR-025** → `CircadianOffset` Feature → dogfood + **Schedule alignment** chart)  
**Evidence:** `docs/handoffs/P24-E3-T1-qa-to-pm.md` · `docs/handoffs/P24-*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, Companion polish-as-primary, TypingRhythm, precise GPS. Next brief: `docs/handoffs/P25-E1-T1-pm-brief.md`.  
**Branch note:** Phase 24 cluster stays on `phase/24-circadian-offset` (PR after freeze). Phase 25 on `phase/25-sustained-load`.

## Task
**PM-GATE-POST-P24 — Choose next Phase 25+ slice**

## Decision (locked)
**Primary Phase 25 track = `SustainedLoadIndicator`** — lock catalog Feature for calm **prolonged high-load** from existing Feature inputs (**StressIndex** + **FatigueIndex** + schedule proxy, prefer **MeetingDensity**) via **ADR-026** → ship in `feature-engine`. Catalog backlog calm rename of “burnout risk”. Calm non-clinical framing only (“prolonged load in this window” — **not** burnout / clinical diagnosis). Prefer Feature-level composition over new Observation families. Complements shipped **`CognitiveLoad`** (current combined demand) with a longer-horizon persistence question.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| Companion polish (as primary) | Parallel OK — not Phase 25 theme |
| TypingRhythm | Deferred — keystroke content / surveillance framing risk; FocusScore already uses typing rate |
| Precise GPS walk tracking | **Rejected** — ADR-024 bar; no reopen without new ADR + approve |
| DeepFocusLikelihood | Deferred — overlaps shipped DeepWorkScore (sustained-focus intensity) |
| **`SustainedLoadIndicator`** | **Chosen** — catalog backlog; Stress / Fatigue / MeetingDensity shipped; distinct from CognitiveLoad (prolonged vs current demand) |

## Universality note
- Core path remains provider-agnostic Observation / Feature contracts.
- Dogfood path today: Mac Desktop + iOS Companion + Apple Health.
- Phase 25 must **not** invent clinical burnout labels or workplace manager dashboards.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 25 v1 track — **done** (`SustainedLoadIndicator`).
2. Open Phase 25 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P25-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 25 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 24 on `phase/24-circadian-offset`; Phase 25 on `phase/25-sustained-load`.

## After gate
Paste **build-qa** command for **P25-E1-T1**.
