# Dev → QA: P17-E2-T1

## Meta
- **Task ID:** P17-E2-T1
- **Title:** Expand iOS Companion / HealthKit ingest per ADR-018
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P17-E2-T1-pm-brief.md` / ADR-018
- **Branch:** `phase/17-wearable-charts`

## What changed
- iOS Companion emits ADR-018 types via existing Auto-sync path (HK observer → queue → `POST /v1/ingest`):
  - `step_count`, `active_energy`, `sleep_interval` (required family when samples exist)
  - keep `heart_rate` + soft-optional `hrv`
  - soft-optional `oxygen_saturation` only when HK has samples
- `bio_spec` validators + router + ingest reject codes (`invalid_step_count` / `invalid_active_energy` / `invalid_sleep_interval` / `invalid_oxygen_saturation`)
- Pipeline normalize for the four new types (aliases / SpO2 fraction→percent / sleep stage closed-set)
- Rust companion scripted sample helpers for CI/dogfood posts
- Docs: contracts emit shipped; `12-development` dogfood; iOS README
- **No** chart ranges / Feature formulas / SQLite migration / Mi Cloud / workout Observation family

### Crates / apps / files
- `crates/bio-spec/src/{step_count,active_energy,sleep_interval,oxygen_saturation}.rs` + error/router/lib
- `crates/bio-spec/tests/contracts.rs`
- `crates/pipeline/src/normalize.rs` + `lib.rs`
- `crates/ingest/src/routes.rs`
- `apps/companion/src/lib.rs` + `tests/sample_ingest.rs`
- `apps/companion/ios/{StepCount,ActiveEnergy,SleepInterval,OxygenSaturation}Sample.swift`
- `apps/companion/ios/HealthKitSyncCoordinator.swift`, `ContentView.swift`, `project.pbxproj`, `README.md`
- Docs: `07-contracts`, `12-development`, companion iOS README

## How to verify (commands)
```bash
cargo test -p bio-spec
cargo test -p pipeline --lib
cargo test -p companion
# optional Swift compile-check (see apps/companion/ios/README.md)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Companion observes/queries HK → queue → ingest (no busy-loop)
- [ ] AC2 Emits step_count / active_energy / sleep_interval; keep HR/HRV; soft-optional SpO2
- [ ] AC3 bio_spec validators + ingest reject; persist existing observations store
- [ ] AC4 Privacy/idle: no raw HK dumps; calm copy; soft-omit sparse
- [ ] AC5 Docs shipped for Companion emit; charts/Features → E3
- [ ] AC6 Tests validators + scripted companion path
- [ ] AC7 No Dashboard series / Feature formulas / migration / Mi Cloud / workout Obs
- [ ] AC8 Handoff present
- [ ] Global DoD

## Risks / not covered
- Live SpO2/sleep sparse on Mi — by design.
- Chart/`get_feature_series` / catalog Features → **P17-E3**.
- Full Xcode device run not executed in this chat (swiftc parse check recommended).

## Notes for QA
- Do not mark Done / touch canvas.
