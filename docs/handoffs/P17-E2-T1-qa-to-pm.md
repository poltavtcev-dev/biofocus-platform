# QA → PM: P17-E2-T1

## Meta
- **Task ID:** P17-E2-T1
- **Title:** Expand iOS Companion / HealthKit ingest per ADR-018
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P17-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
cargo test -p bio-spec          # ok (incl. ADR-018 modules + contracts)
cargo test -p pipeline --lib    # ok (36) — step/energy/sleep/SpO2 normalize
cargo test -p companion         # ok (6) — scripted ADR-018 validate + HR ingest
# Swift compile-check (iphonesimulator) — exit 0
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 HK observer → queue → ingest, no busy-loop | **Pass** — `HealthKitSyncCoordinator` hourly background delivery + observers for new types |
| AC2 Emit ADR-018 families; keep HR/HRV; soft-optional SpO2 | **Pass** — Swift DTOs + coordinator enqueue; SpO2 soft-omit |
| AC3 Validators + ingest reject; existing store | **Pass** — bio-spec + routes reject codes; no migration |
| AC4 Privacy/idle / calm copy | **Pass** — README/UI copy non-clinical; no raw HK dumps in DTOs |
| AC5 Docs emit shipped; charts → E3 | **Pass** — `07-contracts` / `12-development` / iOS README |
| AC6 Tests | **Pass** — validators + companion scripted + normalize |
| AC7 No chart/Feature/migration/Mi Cloud/workout Obs | **Pass** |
| AC8 Handoff | **Pass** |
| Global DoD | **Pass** |

### Extra checks
- Soft-omit path: SpO2 / missing sleep stage covered by validators + Companion soft-enqueue.
- Auto-sync off → observers do not flush (`isAutoSyncEnabled` guard).

## Defects (if any)
- None blocking.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — **P17-E2-T1** Done; Ready **P17-E3-T1**
- [ ] Execution canvas
- [ ] Optionally `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` (Companion emit shipped)
- [ ] No PR during freeze

## Suggested next Ready task
- **P17-E3-T1** — Chart ranges IPC/UI (`get_feature_series`) + Features from wearable Observations

## Notes for PM
- Branch: `phase/17-wearable-charts`. SpO2 sparse on Mi expected.
- Physical-device dogfood still recommended after PM close.
