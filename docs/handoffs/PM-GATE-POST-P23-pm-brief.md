# PM Brief → PM: PM-GATE-POST-P23 (closed)

**From:** PM (close of Phase 23 Personal Context Layer)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **`CircadianOffset`** → Phase 24 opened; Ready **P24-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 23 E1–E3 (**ADR-024** → `DeskAwayPresence` + health→prompt → dogfood + **Away from desk** chart)  
**Evidence:** `docs/handoffs/P23-E3-T1-qa-to-pm.md` · `docs/handoffs/P23-*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, Companion polish-as-primary, TypingRhythm, precise GPS. Next brief: `docs/handoffs/P24-E1-T1-pm-brief.md`.  
**Branch note:** Phase 23 cluster stays on `phase/23-personal-context` (PR after freeze). Phase 24 on `phase/24-circadian-offset`.

## Task
**PM-GATE-POST-P23 — Choose next Phase 24+ slice**

## Decision (locked)
**Primary Phase 24 track = `CircadianOffset`** — lock catalog Feature for calm **alignment of work / activity timing vs sleep timing** from existing sleep + activity (and related Feature) inputs via **ADR-025** → ship in `feature-engine`. Catalog backlog already marked “deferred (after Personal Context)” — Personal Context Layer is Done. Calm non-clinical framing only (“schedule alignment in this window” — **not** chronotype diagnosis / sleep-disorder claims). Prefer Feature-level composition over new Observation families.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| Companion polish (as primary) | Parallel OK — not Phase 24 theme |
| TypingRhythm | Deferred — keystroke content / surveillance framing risk |
| Precise GPS walk tracking | **Rejected** — ADR-024 bar; no reopen without new ADR + approve |
| **`CircadianOffset`** | **Chosen** — catalog backlog after Personal Context; sleep + activity inputs shipped (P17+); complements presence/context with timing alignment |
| Other | Not needed |

## Universality note
- Core path remains provider-agnostic Observation / Feature contracts.
- Dogfood path today: Mac Desktop + iOS Companion + Apple Health.
- Phase 24 must **not** invent clinical chronotype labels or hard-code a single wearable brand into Feature math.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 24 v1 track — **done** (`CircadianOffset`).
2. Open Phase 24 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P24-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 24 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 23 on `phase/23-personal-context`; Phase 24 on `phase/24-circadian-offset`.

## After gate
Paste **build-qa** command for **P24-E1-T1**.
