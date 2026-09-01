# Dev → QA: P26-E1-T1

## Meta
- **Task ID:** P26-E1-T1
- **Title:** ADR-027: lock OSS public-launch scope (gates, non-goals, relationship to packaging runbook)
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P26-E1-T1-pm-brief.md`
- **Branch:** `phase/26-oss-public-launch`

## What changed
- **ADR-027** in `docs/decision-log.md` (summary table + detail): Phase 26 v1 locks
  - Primary = **OSS Public Launch Hygiene**
  - SoT path **`docs/19-oss-public-launch.md`** (body deferred to E2 — file still missing by design)
  - Relationship to **`docs/18-packaging-runbook.md`**: packaging = signed/notarized distribution ops; OSS doc = source / license / contributor / visibility honesty — do not merge files; do not commit signing secrets
  - Three checklist layers: (1) repo/docs hygiene → (2) post–2026-09-01 `main` catch-up → (3) notarized GitHub Release before public visibility
  - Freeze vs after-freeze table (docs + local unsigned dry-run notes OK during freeze; PR / public Release flip wait)
  - AGPLv3 Core open; personal self-tracking framing in public copy
  - Schema: **none** — no migration; no new Observation; no Feature math
  - E2 authors `19-oss…` + touchpoints; E3 dry-run vs packaging runbook
  - Rejected: IDE; weather; App Store primary; Companion polish-as-primary; TypingRhythm; DeepFocusLikelihood; precise GPS; Feature/Observation math; Pattern Discovery as primary; PR during freeze; cloud accounts/telemetry by default; secret Feature formulas; public Release during freeze
- Docs planned/ADR notes: `12-development`, `00-vision` (ADR-027 pointer), `16-glossary`
- **No** `docs/19-oss-public-launch.md` body yet (→ P26-E2)
- **No** crates / apps code; **no** migration

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` for this task).
- Branch: `phase/26-oss-public-launch`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-027" docs/decision-log.md docs/12-development.md docs/00-vision.md docs/16-glossary.md

# SoT path + packaging relationship + layers + freeze
rg -n "19-oss-public-launch|18-packaging-runbook|2026-09-01|notarized|freeze" \
  docs/decision-log.md docs/12-development.md

# Rejected alts
rg -n "TypingRhythm|DeepFocusLikelihood|App Store|telemetry|secret|PR during freeze|IDE plugin|weather|Pattern Discovery|precise GPS" \
  docs/decision-log.md

# Launch doc body still deferred (E2)
test ! -f docs/19-oss-public-launch.md && echo "19 missing OK (E2)"

# No Feature / migration work this task
git diff --name-only -- crates/ apps/ || true
# expect: empty for this task's staged/unstaged code (docs-only)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-027 records Phase 26 primary = OSS Public Launch Hygiene; SoT `19-oss…`; vs `18-packaging`; personal self-tracking framing
- [ ] AC2: Three checklist layers locked; freeze vs after-freeze stated
- [ ] AC3: Rejected alternatives documented (IDE/weather/App Store/Companion/TypingRhythm/DeepFocusLikelihood/GPS/Feature math/Pattern Discovery/PR freeze/cloud/telemetry/secret formulas)
- [ ] AC4: Schema none — no migration; no new Observation; no Feature formula
- [ ] AC5: E2 author launch doc; E3 dry-run vs packaging sketched
- [ ] AC6: Docs touched (`12` / `00-vision` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P26-E1-T1-dev-to-qa.md`
- [ ] Global DoD: AGPLv3 open; packaging ≠ secret math; PR freeze; no launch-doc body / Release in E1

## Risks / not covered
- Full prose of `docs/19-oss-public-launch.md` deferred to **P26-E2** (must still obey ADR-027).
- Exact GitHub visibility / Release naming left for E2/E3 within locked layers.

## Notes for QA
- Do **not** expect `docs/19-oss-public-launch.md` to exist yet — absence is intentional until E2.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty PM-gate docs may exist on the branch — out of AC unless they contradict ADR-027.
