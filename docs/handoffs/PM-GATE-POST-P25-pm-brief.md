# PM Brief → PM: PM-GATE-POST-P25

**From:** PM (close of Phase 25 SustainedLoadIndicator)  
**To:** PM (this gate)  
**Status:** Ready  
**Date:** 2026-08-12  
**Closed previous:** Phase 25 E1–E3 (**ADR-026** → `SustainedLoadIndicator` Feature → dogfood + **Prolonged load** chart)  
**Evidence:** `docs/handoffs/P25-E3-T1-qa-to-pm.md` · `docs/handoffs/P25-*-qa-to-pm.md`  
**Branch note:** Phase 25 cluster stays on `phase/25-sustained-load` (PR after freeze). New Phase 26 branch only after this gate picks a track.

## Task
**PM-GATE-POST-P25 — Choose next Phase 26+ slice**

## Why
Phase 25 shipped calm **prolonged-load** persistence (`SustainedLoadIndicator`) distinct from CognitiveLoad. Catalog / vision leftovers remain: IDE, weather ambient, App Store packaging, Companion polish-as-primary, TypingRhythm, DeepFocusLikelihood. Pick **one** primary track for Phase 26 — or an explicit “other” with ADR-needed scope — without opening a PR during freeze.

## Candidates (pick one primary)

| Candidate | Notes / known bar |
| :--- | :--- |
| IDE plugin | Still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) unless a new Observation contract is approved |
| Weather ambient | Cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Commercial track beyond P12 runbook; OSS launch path in `docs/19-oss-public-launch.md` |
| Companion polish (as primary) | Parallel OK historically — not a Feature-math theme unless scoped |
| TypingRhythm | Deferred — keystroke content / surveillance framing risk; FocusScore already uses typing rate |
| DeepFocusLikelihood | Deferred — overlaps shipped DeepWorkScore (sustained-focus intensity) |
| Precise GPS walk tracking | **Rejected** — ADR-024 bar; no reopen without new ADR + approve |
| Other | Allowed if inputs exist and Global DoD / calm non-clinical copy hold |

## Universality note
- Core path remains provider-agnostic Observation / Feature contracts.
- Dogfood path today: Mac Desktop + iOS Companion + Apple Health.
- Phase 26 must **not** invent clinical burnout / diagnosis labels or workplace manager dashboards.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 26 v1 track (or explicit defer-all with stated next hygiene).
2. Open Phase 26 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` (or document pause).
3. Write `docs/handoffs/P26-E1-T1-pm-brief.md` (or equivalent first task) when a track is chosen.
4. Update execution canvas to Phase 26 Ready (or gate-paused state).
5. Reaffirm **PR freeze until 2026-09-01** — Phase 25 on `phase/25-sustained-load`; new branch only after choice.

## Out of scope (this gate chat)
- Implementing Feature math / collectors
- Opening a PR
- Reopening precise GPS without ADR + approve

## After gate
Paste **build-qa** (or next Ready) command for the first Phase 26 task.
