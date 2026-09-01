# PM Brief → Dev: P26-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · Phase 26 closed · next **PM-GATE-POST-P26**  
**Date:** 2026-08-12  
**Closed previous:** **P26-E2-T1** (`docs/19-oss-public-launch.md` authored; QA Pass)  
**Evidence:** `docs/handoffs/P26-E2-T1-qa-to-pm.md` · `docs/handoffs/P26-E2-T1-dev-to-qa.md`  
**Phase:** Phase 26 OSS Public Launch Hygiene — Epic P26-E3  
**Branch:** `phase/26-oss-public-launch`

## Task
**P26-E3-T1 — Dry-run release checklist vs packaging runbook (ADR-027)**

## Why
**ADR-027** + SoT `docs/19-oss-public-launch.md` lock three layers. Layer (1) SoT exists. Layers (2)–(3) wait until after PR freeze. E3 ships a **dry-run checklist** (and optional local unsigned dry-run **notes**) aligned with `docs/18-packaging-runbook.md` so maintainers know what to rehearse now vs what is forbidden until freeze ends + `main` catch-up. Still **no** public notarized Release and **no** visibility flip in this task.

## Acceptance Criteria
1. Add a **dry-run release checklist** (prefer section in `docs/19-oss-public-launch.md` and/or a clearly linked subsection that **references** `docs/18-packaging-runbook.md` — do **not** copy signing secrets / credential recipes into git).
2. Checklist must distinguish:  
   - **During freeze (now):** unsigned / local dry-run **notes** OK; docs/handoffs OK; **no** PR; **no** public notarized GitHub Release as visibility flip.  
   - **After freeze + layer (2) `main` catch-up:** notarized Release per packaging runbook, **then** visibility flip (layer 3).  
   Unsigned local dry-run must **not** be treated as satisfying layer (3).
3. Optional: short maintainer notes from a local unsigned dry-run attempt (what was tried, what blocked, no secrets). If credentials / notarization are unavailable, document that gap honestly rather than inventing success.
4. Touchpoints: `12-development` / SoT / packaging cross-links updated so E3 dry-run is discoverable; still state **public launch not Done**.
5. Schema / product: **none** — no migration; no Feature math; no App Store productization; no cloud/telemetry defaults.
6. Explicitly **do not** cut a public GitHub Release or flip repo visibility; **do not** open a PR.
7. Handoff: `docs/handoffs/P26-E3-T1-dev-to-qa.md`.

## Out of scope
- Merging phase clusters to `main` / opening PRs (waits until after 2026-09-01)
- Cutting a public notarized GitHub Release / flipping visibility
- App Store listing / productization
- New collectors / Features / Observations / Feature catalog math
- IDE · weather · TypingRhythm · DeepFocusLikelihood · Pattern Discovery expansion
- Committing signing credentials, notarization passwords, or Keychain material

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. OSS launch DoD)
- Obey **ADR-027** + SoT `19-oss…` — packaging ops stay in `18-packaging-runbook.md`
- Branch: `phase/26-oss-public-launch`
- AGPLv3 Core stays open; packaging ≠ secret Feature math
- Personal self-tracking framing only
- LLM remains L5 interpret-only
- **No migration**
- Prefer extend existing docs — do not invent a parallel packaging SoT

## After QA Pass
PM → mark P26-E3-T1 Done; close Epic **P26-E3** and **Phase 26** if no further P26 tasks; next = **PM-GATE-POST-P26** (or park until freeze lift) **without** claiming public launch Done / without opening a PR during freeze.
