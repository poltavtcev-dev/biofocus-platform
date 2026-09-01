# PM Brief → PM: PM-GATE-POST-P12

**From:** PM (close of Phase 12)  
**To:** PM (next chat)  
**Status:** Done (2026-08-10) — chose **IDE/Git plugin wave-2** → Phase 13 opened; Ready **P13-E1-T1**  
**Date:** 2026-08-10  
**Closed previous:** P12-E3-T1 (QA Pass with notes — `AmbientMediaShare` + packaging runbook); Epic **P12-E3** ✅; **Phase 12** ✅  
**Evidence:** `docs/handoffs/P12-E3-T1-qa-to-pm.md`  
**Outcome:** Deferred weather/light + App Store packaging product. Next brief: `docs/handoffs/P13-E1-T1-pm-brief.md`.

## Task
**PM-GATE-POST-P12 — Choose next Phase 13+ slice**

## Why
Phase 0–12 ladder is complete through ambient Now Playing + commercial packaging runbook. Vision (`docs/00-vision.md`) says open later slices via PM gate, not all at once. Do **not** invent a full Phase 13 backlog without an explicit scope ADR / Ready brief.

## Acceptance Criteria
1. Pick **one** primary Phase 13 v1 track from candidates (or document a different justified slice):
   - **IDE/Git plugin wave** (deferred from ADR-010 / source priority after Browser)
   - **Weather / light ambient** (deferred from ADR-012)
   - **App Store / deeper packaging product** (beyond runbook)
   - **Other** — must cite vision / dogfood rationale
2. Open Phase 13 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` (Sprint numbers, goal, Epic E1 = ADR if architecture boundaries needed).
3. Write `docs/handoffs/P13-E1-T1-pm-brief.md` (or renamed first task) with AC, out of scope, branch, role.
4. Update execution canvas QUEUE / focus / horizon to Phase 13 Ready.
5. Reaffirm **PR freeze until 2026-09-01** — no PR for this gate; Phase 12 cluster stays on `phase/12-ambient-packaging` for post-freeze PR.

## Out of scope
- Implementing collectors / Features / installers in this gate chat
- Opening a PR
- Shipping multiple unrelated tracks in one phase without ADR justification

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Local-First · Capability Plugin Model · calm non-clinical copy
- Features only with real Observation inputs
- Commercial packaging ≠ closed Core Feature math

## After gate
Paste **build-qa** (or next-mode) command for the first Ready Phase 13 task.
