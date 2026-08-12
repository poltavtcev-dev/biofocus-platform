# PM Brief → Dev: P26-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · SoT `19-oss…` authored · next **P26-E3-T1**  
**Date:** 2026-08-12  
**Closed previous:** **P26-E1-T1** (**ADR-027** locked; QA Pass)  
**Evidence:** `docs/handoffs/P26-E1-T1-qa-to-pm.md` · `docs/handoffs/P26-E1-T1-dev-to-qa.md`  
**Phase:** Phase 26 OSS Public Launch Hygiene — Epic P26-E2  
**Branch:** `phase/26-oss-public-launch`

## Task
**P26-E2-T1 — Author `docs/19-oss-public-launch.md` + hygiene touchpoints per ADR-027**

## Why
Vision already links **`docs/19-oss-public-launch.md`**, but the file is still **missing**. **ADR-027** locked SoT path, packaging vs OSS split (`18-packaging-runbook.md`), three checklist layers, and freeze vs after-freeze gates. E2 authors the launch SoT body + touchpoints so maintainers have an honest public-beta checklist — still **no** public Release flip and **no** Feature math.

## Acceptance Criteria
1. Create **`docs/19-oss-public-launch.md`** as Phase 26 OSS / visibility SoT per **ADR-027**. Must cover: purpose; relationship to **`docs/18-packaging-runbook.md`** (ops stay in 18 — no secret duplication); personal self-tracking framing; AGPLv3 Core open / packaging ≠ secret Feature math; App Store deferred.
2. Document the **three locked layers** as a maintainer checklist:  
   - (1) repo / docs honesty & hygiene  
   - (2) post–**2026-09-01** `main` catch-up / related cluster merges (after PR freeze)  
   - (3) notarized GitHub Release **before** flipping public visibility  
   State clearly what is allowed **during** freeze (docs + local unsigned dry-run **notes**) vs what waits until freeze ends.
3. Hygiene touchpoints: update links/pointers as needed in `docs/00-vision.md`, README (and/or CONTRIBUTING if present), and `docs/12-development.md` so they resolve to the new SoT without claiming “public launch Done”.
4. Schema / product: **none** — no migration; no new Observation / Feature math; no Pattern Discovery expansion; no cloud accounts / telemetry by default.
5. Explicitly **do not** cut a public GitHub Release or flip repo visibility in this task; leave dry-run / unsigned notes to **P26-E3**.
6. Docs status notes: `12-development` / glossary reflect E2 SoT authored (ADR-027 remains locked).
7. Handoff: `docs/handoffs/P26-E2-T1-dev-to-qa.md`.

## Out of scope
- Dry-run release checklist / local unsigned dry-run execution notes (→ **P26-E3**)
- Cutting a public notarized GitHub Release / flipping visibility
- Opening a PR (PR freeze until 2026-09-01)
- App Store listing / productization
- New collectors / Features / Observations / Feature catalog math
- IDE plugin; weather; TypingRhythm; DeepFocusLikelihood; Pattern Discovery expansion
- Copying signing credentials / secrets into the repo

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. OSS launch DoD)
- Obey **ADR-027** locks — do not reopen rejected alts
- Branch: `phase/26-oss-public-launch`
- AGPLv3 Core stays open; commercial packaging ≠ closed Feature math
- Personal self-tracking only — not workplace monitoring framing in public docs
- LLM remains L5 interpret-only
- **No migration**
- Prefer reference `18-packaging-runbook.md` — do not merge packaging ops into `19-oss…`

## After QA Pass
PM → mark P26-E2-T1 Done; Ready **P26-E3-T1** (dry-run checklist vs packaging runbook).
