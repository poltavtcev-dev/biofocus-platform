# PM Brief → PM: PM-GATE-POST-P25 (closed)

**From:** PM (close of Phase 25 SustainedLoadIndicator)  
**To:** PM (this gate)  
**Status:** Done (2026-08-12) — chose **OSS Public Launch Hygiene** → Phase 26 opened; Ready **P26-E1-T1**  
**Date:** 2026-08-12  
**Closed previous:** Phase 25 E1–E3 (**ADR-026** → `SustainedLoadIndicator` Feature → dogfood + **Prolonged load** chart)  
**Evidence:** `docs/handoffs/P25-E3-T1-qa-to-pm.md` · `docs/handoffs/P25-*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store listing, Companion polish-as-primary, TypingRhythm, DeepFocusLikelihood, precise GPS, Feature-math expansion. Next brief: `docs/handoffs/P26-E1-T1-pm-brief.md`.  
**Branch note:** Phase 25 cluster stays on `phase/25-sustained-load` (PR after freeze). Phase 26 on `phase/26-oss-public-launch`.

## Task
**PM-GATE-POST-P25 — Choose next Phase 26+ slice**

## Decision (locked)
**Primary Phase 26 track = OSS Public Launch Hygiene** — lock scope via **ADR-027** → author the missing SoT `docs/19-oss-public-launch.md` (already linked from `/docs/00-vision.md`) → dry-run checklist aligned with `docs/18-packaging-runbook.md`. Layers: (1) repo / docs hygiene, (2) post–2026-09-01 `main` catch-up / cluster merges, (3) notarized GitHub Release before flipping public visibility. **AGPLv3 Core stays open.** **Not** App Store productization. **Not** new Observation / Feature catalog math.

Rationale: Feature catalog backlog is empty of safe distinct math (`TypingRhythm` surveillance bar; `DeepFocusLikelihood` overlaps shipped `DeepWorkScore`). Remaining plugin/ambient/commercial candidates stay behind prior ADR bars. Highest leverage before freeze lift = make the OSS launch path real and honest.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` (ADR-013 bar) |
| Weather ambient | Deferred — cloud / geo vs Local-First (same bar as ADR-015) |
| App Store packaging product | Deferred — commercial listing beyond P12 runbook; vision keeps App Store deferred |
| Companion polish (as primary) | Parallel OK — not Phase 26 theme (no crisp unfinished Companion UX epic) |
| TypingRhythm | Deferred — keystroke content / surveillance framing risk; FocusScore already uses typing rate |
| DeepFocusLikelihood | Deferred — overlaps shipped DeepWorkScore |
| Precise GPS walk tracking | **Rejected** — ADR-024 bar; no reopen without new ADR + approve |
| Feature-math “other” | Deferred — no distinct catalog node with clear inputs |
| Pattern Discovery rule expansion | Strong runner-up — park for later gate (after OSS hygiene) |
| **OSS Public Launch Hygiene** | **Chosen** — vision links missing `docs/19-oss-public-launch.md`; packaging runbook exists; App Store stays deferred |

## Universality note
- Core path remains provider-agnostic Observation / Feature contracts.
- Dogfood path today: Mac Desktop + iOS Companion + Apple Health.
- Phase 26 must **not** invent clinical labels, workplace manager dashboards, cloud accounts by default, or secret Core math.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 26 v1 track — **done** (OSS Public Launch Hygiene).
2. Open Phase 26 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P26-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 26 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 25 on `phase/25-sustained-load`; Phase 26 on `phase/26-oss-public-launch`.

## After gate
Paste **build-qa** command for **P26-E1-T1**.
