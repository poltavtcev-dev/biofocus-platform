# Dev → QA: P17-E1-T1

## Meta
- **Task ID:** P17-E1-T1
- **Title:** Lock Phase 17 contracts (HealthKit depth + chart ranges) as ADR + docs
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P17-E1-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P17-E1
- **Branch:** `phase/17-wearable-charts`

## What changed
- **ADR-018** in `docs/decision-log.md`: Phase 17 v1 contracts — HealthKit Observation `data_type`s + Dashboard chart-range IPC stance.
  - Wearable: keep `heart_rate` + soft-optional `hrv`; lock `step_count`, `active_energy`, `sleep_interval`; soft-optional `oxygen_saturation`; existing `observations` store; **no** migration.
  - Charts: ranges `1h`/`8h`/`12h`/`1d`/`1w`; `get_feature_series` recompute-on-read (ADR-008); Snapshot = **latest**; UI ↛ SQLite; default coarser steps by range.
  - Analysis: Features/Insights/Recommendations without LLM first; L5 interpret-only.
  - Rejected: Mi Cloud; Feature-history SQLite; UI→DB; clinical claims; busy-loop HK; required workout Observation family; PR during freeze.
- ADR-017 detail updated to point at ADR-018 as contract SoT (sequencing remains ADR-017).
- Contract sketches: `07-contracts`, `09-api` (`get_feature_series`), catalog backlog stubs (`EnergyScore` / `SleepDebt` / `ActivityBalance` → P17), `10-security`, `12-development`, `16-glossary`, `04-storage`.
- Epic split documented in `SPRINT_ROADMAP` Phase 17 (E1 ADR-018 · E2 Companion · E3 ranges/Features).
- Intent note updated: `PARKED-P17-wearable-dashboard-intent.md`.
- **No** Companion Swift/Rust emit code; **no** Dashboard UI; **no** Feature formulas beyond catalog stubs; **no** SQLite migration / crate code.

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` implementation):
  - `docs/decision-log.md` (ADR-018 summary + detail; ADR-017 pointer)
  - `docs/07-contracts.md`, `docs/09-api.md`, `docs/06-feature-catalog.md`
  - `docs/10-security.md`, `docs/12-development.md`, `docs/16-glossary.md`, `docs/04-storage.md`
  - `docs/SPRINT_ROADMAP.md` (epic split / ADR-018 refs — **not** Done)
  - `docs/handoffs/PARKED-P17-wearable-dashboard-intent.md`
  - `docs/handoffs/P17-E1-T1-dev-to-qa.md` (this file)
  - (pre-existing PM open: `P17-E1-T1-pm-brief.md` + status docs may be on tree)

## How to verify (commands)
```bash
# ADR + contracts present
rg -n "ADR-018" docs/decision-log.md docs/07-contracts.md docs/09-api.md docs/12-development.md docs/10-security.md docs/16-glossary.md docs/04-storage.md docs/SPRINT_ROADMAP.md

# Locked data_types + chart IPC
rg -n "step_count|active_energy|sleep_interval|oxygen_saturation|get_feature_series" docs/decision-log.md docs/07-contracts.md docs/09-api.md

# No implementation / migration from this task
git diff --name-only -- 'crates/' 'apps/' || true
rg -n "CREATE TABLE" docs/decision-log.md docs/04-storage.md | head
# Expect: no new Feature-history / wearable tables proposed for apply
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-018 in decision-log — wearable types + chart ranges + analysis ladder + rejected alts; no ADR-017 overload
- [ ] AC2: Contracts updated (`07` / `09` / catalog stubs / security / development / glossary as needed); intent marked contract-locked; impl → E2/E3
- [ ] AC3: Epic split in SPRINT_ROADMAP Phase 17 (E1/E2/E3) aligned
- [ ] AC4: No Companion/UI code; no Feature formulas beyond stubs; no SQLite migration applied
- [ ] AC5: This handoff exists
- [ ] Global DoD: calm non-clinical; UI↛DB; glossary; no PR during freeze

## Risks / not covered
- bio-spec validators / Companion HealthKit observers → **P17-E2**.
- `get_feature_series` host + range picker UI + catalog Feature math → **P17-E3**.
- SpO2 may be sparse on Mi — soft-optional by design; Features must omit when absent.

## Notes for QA
- Docs-only task — no `cargo test` required unless tree also has unrelated code (should not for this task).
- Do **not** mark Done / touch canvas; next Ready after PM close = P17-E2-T1.
