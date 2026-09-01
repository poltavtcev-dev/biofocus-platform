# QA → PM: P17-E1-T1

## Meta
- **Task ID:** P17-E1-T1
- **Title:** Lock Phase 17 contracts (HealthKit depth + chart ranges) as ADR + docs
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P17-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
rg -n "ADR-018" docs/decision-log.md docs/07-contracts.md docs/09-api.md \
  docs/12-development.md docs/10-security.md docs/16-glossary.md \
  docs/04-storage.md docs/SPRINT_ROADMAP.md docs/06-feature-catalog.md
# hits across ADR summary/detail + contracts

rg -n "step_count|active_energy|sleep_interval|oxygen_saturation|get_feature_series" \
  docs/decision-log.md docs/07-contracts.md docs/09-api.md
# locked payloads + IPC sketch present

git diff --name-only -- 'crates/' 'apps/'
# empty — no Companion/UI/crate implementation
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 New ADR after ADR-017 (ADR-018) locks wearable + charts + analysis + rejected | **Pass** — summary table + detail; does not overload ADR-017 (sequencing pointer updated) |
| AC2 Contract docs + intent contract-locked; impl → E2/E3 | **Pass** — `07` / `09` / catalog stubs / security / development / glossary / storage / PARKED intent |
| AC3 Epic split in SPRINT_ROADMAP (E1/E2/E3) | **Pass** — ADR-018 named on epics; Ready still P17-E1-T1 (not Done) |
| AC4 No Companion/UI code; no Feature formulas beyond stubs; no migration | **Pass** — no crates/apps diffs; no Feature-history table to apply |
| AC5 Handoff | **Pass** — `docs/handoffs/P17-E1-T1-dev-to-qa.md` |
| Global DoD | **Pass** — calm/non-clinical; UI↛DB; glossary; PR freeze noted |

### Extra checks
- Soft-optional SpO2 documented; workout Observation family deferred to Life Event.
- Chart default steps + Snapshot=latest + recompute-on-read documented.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P17-E1-T1** to Done; Ready **P17-E2-T1**
- [ ] Execution canvas — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS.md` / `14-roadmap.md` (ADR-018 locked)
- [ ] Do **not** open PR during freeze

## Suggested next Ready task
- **P17-E2-T1** — Expand iOS Companion / HealthKit ingest per ADR-018

## Notes for PM
- Branch: `phase/17-wearable-charts`. Docs-only cluster for E1; Companion code starts in E2.
- SpO2 sparse on Mi is expected — Features in E3 must soft-omit.
