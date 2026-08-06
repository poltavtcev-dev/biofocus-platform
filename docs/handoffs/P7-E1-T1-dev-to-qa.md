# Dev → QA: P7-E1-T1

## Meta
- **Task ID:** P7-E1-T1
- **Title:** Feature confidence contract + ADR + wire
- **Role that built:** Dev
- **Date:** 2026-08-06
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P7-E1-T1; brief `docs/handoffs/P7-E1-T1-pm-brief.md`
- **Branch:** `phase/7-trust-layer` (from `phase/6-dogfood-fixes` tip)

## What changed
- **ADR-007** in `docs/decision-log.md`: Feature-level confidence vs Observation.confidence; v1 formula `coverage × mean(evidence Observation.confidence)`; missing-input = omit empty / lower confidence when partial; no SQLite Feature schema; rejected UI-only heuristics + second registry.
- Domain: `bio_spec::Feature.confidence: Confidence` (`[0.0, 1.0]`); `Confidence::saturating_from` / `ONE` / `ZERO`.
- Catalog compute: shared helpers in `feature_engine::catalog::confidence`; all `register_catalog_v1` nodes set confidence (Focus/Stress/Fatigue/CSR/calendar).
- IPC: `get_feature_snapshot` Feature DTO includes `confidence` (camelCase `f64`).
- Docs: `02-domain-model`, `06-feature-catalog`, `09-api`, `12-development`, `16-glossary`, `00-vision`.
- Tests: rich vs thin FocusScore coverage; low Observation confidence → lower StressIndex confidence; idle empty; helper unit tests.

## Crates / files touched
- `docs/decision-log.md` (ADR-007)
- `docs/02-domain-model.md`, `06-feature-catalog.md`, `09-api.md`, `12-development.md`, `16-glossary.md`, `00-vision.md`
- `crates/bio-spec/src/{feature,observation}.rs`, `crates/bio-spec/tests/contracts.rs`
- `crates/feature-engine/src/catalog/{confidence,focus_score,stress_index,fatigue_index,context_switch_rate,meeting_density,recovery_between_meetings,calendar_meeting,mod}.rs`
- `crates/feature-engine/src/{snapshot,alert,engine}.rs`
- `crates/knowledge-engine`, `crates/report-engine` (test Feature literals)
- `apps/desktop/src-tauri/src/lib.rs` (FeatureDto + tests)

## How to verify (commands)
```bash
cargo test -p bio-spec -p feature-engine -p knowledge-engine -p report-engine -p pipeline
cd apps/desktop/src-tauri && cargo test --lib
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-007 recorded (relationship, v1 formula, missing-input policy, rejected alternatives)
- [ ] AC2: `Feature` + `get_feature_snapshot` JSON expose confidence ∈ `[0.0, 1.0]`; no new SQLite schema
- [ ] AC3: Catalog path computes confidence; thin/empty per ADR (omit empty; lower when partial) + tested
- [ ] AC4: Unit tests — rich > thin; missing HRV/context → lower; idle-safe
- [ ] AC5: Domain + catalog + API docs updated
- [ ] AC6: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; calm copy (data quality, not clinical)

## Risks / not covered
- Explanation factor breakdown → **P7-E2-T1** (out of scope).
- UI does not yet visualize confidence (data only in snapshot JSON).
- Fatigue Focus-only evidence mean uses upstream Feature.confidence when no local Observations — documented in catalog.

## Notes for QA
- Prefer asserting FocusScore confidence: full inputs → `1.0`; no HRV → `2/3`; typing-only → `1/3`.
- StressIndex: Observation confidence `0.5` → Feature confidence `0.5`.
- Empty `&[]` run → empty Features (idle-safe).
