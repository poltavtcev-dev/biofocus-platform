# Dev → QA: P26-E3-T1

## Meta
- **Task ID:** P26-E3-T1
- **Title:** Dry-run release checklist vs packaging runbook (ADR-027)
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P26-E3-T1-pm-brief.md` · ADR-027 · `docs/19-oss-public-launch.md`
- **Branch:** `phase/26-oss-public-launch`

## What changed
- Added **§ Dry-run release checklist** to `docs/19-oss-public-launch.md`:
  - During freeze vs after freeze + layer (2)
  - Hard rule: unsigned local dry-run ≠ layer (3)
  - References `docs/18-packaging-runbook.md` — no credential recipes copied
  - Maintainer dry-run notes (2026-08-12): honest gaps (no Developer ID Application observed; no local unsigned `.app` this pass; notarization credentials not exercised; freeze active)
- Touchpoints: `12-development`, glossary, packaging related-docs, vision pointer
- Status banner: dry-run checklist added; **public launch not Done**
- **No** public Release / visibility flip / PR; **no** crates/apps; **no** migration / Feature math

## Crates / apps / files touched
| Path | Change |
| :--- | :--- |
| `docs/19-oss-public-launch.md` | dry-run checklist + notes |
| `docs/12-development.md` | Phase 26 → E3 dry-run |
| `docs/16-glossary.md` | E3 dry-run |
| `docs/18-packaging-runbook.md` | link to dry-run § |
| `docs/00-vision.md` | pointer mentions dry-run § |
| **Not touched** | `crates/` / `apps/` |

## How to verify (commands)
```bash
rg -n "Dry-run release checklist|During PR freeze|After freeze|Hard rule|Maintainer dry-run notes|public launch not Done" \
  docs/19-oss-public-launch.md

rg -n "18-packaging-runbook" docs/19-oss-public-launch.md

# No secret recipes in 19
rg -n "app-specific-password|\.p12|notarytool submit|Developer ID Application:" docs/19-oss-public-launch.md \
  && echo "FAIL: secrets/ops leaked" || echo "OK: no packaging secrets in 19"

# Discoverability
rg -n "Dry-run|dry-run" docs/12-development.md docs/16-glossary.md docs/18-packaging-runbook.md docs/00-vision.md

# No code
git diff --name-only -- crates/ apps/
# expect: empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dry-run checklist in SoT; references 18; no secret duplication
- [ ] AC2: Freeze vs after-freeze + layer 2→3; unsigned ≠ layer 3
- [ ] AC3: Optional maintainer notes honest (gaps OK)
- [ ] AC4: Touchpoints; public launch not Done
- [ ] AC5: Schema/product none
- [ ] AC6: No public Release / visibility flip / PR
- [ ] AC7: Handoff present
- [ ] Global DoD: ADR-027; AGPLv3; personal framing

## Risks / not covered
- Full `pnpm tauri build` unsigned dogfood not executed this pass — notes record absence of local `.app`.
- Layer (3) remains blocked until freeze ends, `main` catch-up, and Developer ID + notarization credentials are available outside git.

## Notes for QA
- Closing Phase 26 / **PM-GATE-POST-P26** is PM-only — still must **not** claim public launch Done.
- Do **not** open a PR (freeze until 2026-09-01).
