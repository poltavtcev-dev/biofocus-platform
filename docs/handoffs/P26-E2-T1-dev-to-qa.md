# Dev → QA: P26-E2-T1

## Meta
- **Task ID:** P26-E2-T1
- **Title:** Author `docs/19-oss-public-launch.md` + hygiene touchpoints per ADR-027
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P26-E2-T1-pm-brief.md` · ADR-027
- **Branch:** `phase/26-oss-public-launch`

## What changed
- Authored **`docs/19-oss-public-launch.md`** (Phase 26 OSS / visibility SoT):
  - Purpose; vs **`docs/18-packaging-runbook.md`** (ops stay in 18; no secret duplication)
  - Personal self-tracking framing; AGPLv3 open / packaging ≠ secret Feature math; App Store deferred
  - Three locked layers as maintainer checklist + freeze vs after-freeze table
  - Explicit: **public launch not Done**; no Release/visibility flip in E2; dry-run → E3
- Hygiene touchpoints: `00-vision`, README, CONTRIBUTING, `12-development`, glossary; cross-link from packaging runbook → 19
- **No** crates/apps code; **no** migration; **no** Feature math; **no** public Release

## Crates / apps / files touched
| Path | Change |
| :--- | :--- |
| `docs/19-oss-public-launch.md` | **new** — SoT body |
| `docs/00-vision.md` | pointer — SoT authored; launch not Done |
| `docs/12-development.md` | Phase 26 note → E2 SoT authored |
| `docs/16-glossary.md` | E2 SoT authored |
| `README.md` | link + personal framing |
| `CONTRIBUTING.md` | link + framing |
| `docs/18-packaging-runbook.md` | related-docs link to 19 |
| **Not touched** | `crates/` / `apps/` |

## How to verify (commands)
```bash
test -f docs/19-oss-public-launch.md && echo "SoT exists"

rg -n "18-packaging-runbook|personal self-tracking|AGPLv3|App Store deferred|2026-09-01|notarized|public launch not Done|freeze" \
  docs/19-oss-public-launch.md

rg -n "19-oss-public-launch" \
  docs/00-vision.md README.md CONTRIBUTING.md docs/12-development.md docs/16-glossary.md \
  docs/18-packaging-runbook.md

# Must NOT claim Done / must defer dry-run
rg -n "public launch not Done|P26-E3|dry-run" docs/19-oss-public-launch.md docs/12-development.md

# No secrets duplicated from packaging into 19
rg -n "app-specific-password|\.p12|notarytool submit|Developer ID Application:" docs/19-oss-public-launch.md \
  && echo "FAIL: secrets/ops leaked into 19" || echo "OK: no packaging secrets in 19"

# No code / Feature math this task
git diff --name-only -- crates/ apps/
# expect: empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `19-oss…` SoT — purpose; vs 18; framing; AGPLv3; App Store deferred
- [ ] AC2: Three layers + freeze vs after-freeze
- [ ] AC3: Touchpoints resolve (vision / README / CONTRIBUTING / 12-dev); no “public launch Done”
- [ ] AC4: Schema/product none
- [ ] AC5: No public Release / visibility flip; dry-run left to E3
- [ ] AC6: 12-development / glossary reflect E2 SoT authored
- [ ] AC7: Handoff present
- [ ] Global DoD: ADR-027 obeyed; PR freeze; no secrets in repo

## Risks / not covered
- Layer (1) checklist items beyond “SoT exists” remain unchecked for maintainers (honesty pass is ongoing).
- Dry-run / unsigned execution notes deferred to **P26-E3**.

## Notes for QA
- Kanban Done / canvas / Ready P26-E3 are PM-only after QA Pass.
- Do **not** open a PR (freeze until 2026-09-01).
