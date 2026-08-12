# PM Brief → Dev: P26-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · **ADR-027** locked · next **P26-E2-T1**  
**Date:** 2026-08-12  
**Closed previous:** **PM-GATE-POST-P25** (chose **OSS Public Launch Hygiene**); Phase 25 SustainedLoadIndicator complete  
**Evidence:** `docs/handoffs/PM-GATE-POST-P25-pm-brief.md` · `docs/handoffs/P25-E3-T1-qa-to-pm.md`  
**Phase:** Phase 26 OSS Public Launch Hygiene — Epic P26-E1  
**Branch:** `phase/26-oss-public-launch`

## Task
**P26-E1-T1 — ADR-027: lock OSS public-launch scope (gates, non-goals, relationship to packaging runbook)**

## Why
Vision already points maintainers at **`docs/19-oss-public-launch.md`**, but the file is **missing**. Feature catalog backlog has no safe distinct math left (`TypingRhythm` / `DeepFocusLikelihood` barred). PM gate post–P25 chose OSS launch hygiene over IDE / weather / App Store listing / Companion polish / Feature-math. Lock an ADR before writing the launch doc so E2/E3 stay scoped: AGPLv3 Core open, App Store deferred, no cloud accounts/telemetry by default, packaging ≠ secret Feature math.

## Acceptance Criteria
1. Record **ADR-027** in `docs/decision-log.md`: Phase 26 v1 primary = **OSS Public Launch Hygiene**. Lock: SoT path `docs/19-oss-public-launch.md`; relationship to `docs/18-packaging-runbook.md` (signed/notarized distribution ops) vs OSS (source, license, contributor path, GitHub Release visibility); personal self-tracking framing in public copy.
2. Lock checklist layers for E2/E3:  
   - (1) repo / docs honesty & hygiene  
   - (2) post–**2026-09-01** `main` catch-up / related cluster merges (after PR freeze)  
   - (3) notarized GitHub Release **before** flipping public visibility  
   State what may run **during** freeze (docs + local unsigned dry-run notes) vs what waits until freeze ends.
3. Rejected alternatives documented (IDE plugin; weather ambient; App Store listing as primary; Companion polish-as-primary; TypingRhythm; DeepFocusLikelihood; precise GPS; new Observation / Feature catalog math this phase; Pattern Discovery rule expansion as this phase primary; opening a PR during freeze; cloud accounts / telemetry by default; secret/proprietary Feature formulas).
4. Schema: **none** — no migration; no new Observation `data_type`; no Feature formula work in Phase 26.
5. Short sketch: E2 authors `docs/19-oss-public-launch.md` + touchpoints (vision/README/`12-development` links as needed); E3 dry-run checklist vs packaging runbook (unsigned/local OK in freeze; notarized public Release only after freeze + merge policy).
6. Docs touch: `12-development` / `00-vision` / glossary as “planned / ADR” notes — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P26-E1-T1-dev-to-qa.md`.

## Out of scope
- Authoring the full `docs/19-oss-public-launch.md` body (→ **P26-E2**)
- Cutting a public GitHub Release / flipping repo visibility
- Opening a PR (PR freeze until 2026-09-01)
- App Store listing / productization
- New collectors / Features / Observations
- IDE plugin; weather; TypingRhythm; DeepFocusLikelihood
- Pattern Discovery / Recommendations expansion

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. OSS launch DoD)
- Branch: `phase/26-oss-public-launch`
- AGPLv3 Core stays open; commercial packaging ≠ closed Feature math
- Personal self-tracking only — not workplace monitoring framing in public docs
- LLM remains L5 interpret-only
- **No migration**
- Prefer extend existing packaging runbook references — do not duplicate secret signing credentials into the repo

## After QA Pass
PM → mark P26-E1-T1 Done; Ready **P26-E2-T1** shaped by ADR-027 (author launch doc).
