# Dev → QA: P7-E2-T1

## Meta
- **Task ID:** P7-E2-T1
- **Title:** Explanation factors on Features
- **Role that built:** Dev
- **Date:** 2026-08-06
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P7-E2-T1; brief `docs/handoffs/P7-E2-T1-pm-brief.md`
- **Branch:** `phase/7-trust-layer`

## What changed
- Domain: `bio_spec::ExplanationFactor { id, label, share }` + optional `Feature.factors` (`serde` default + skip empty) — additive field, **no** new ADR (note under ADR-007 in `decision-log.md`).
- Catalog: `FocusScore` emits calm factors (`typing` / `stability` / `hrv`) with renormalized `share` (sum = 1.0 over present components); empty window still omits Feature.
- Other catalog nodes keep `factors: []` (omit-until-present on IPC).
- IPC: `get_feature_snapshot` Feature DTO exposes `factors` when non-empty; omits key when empty. UI ↛ SQLite; no Feature SQLite schema.
- Docs: domain, catalog (`FocusScore`), API, glossary, vision L2, `12-development`, decision-log note.
- Tests: full shares 0.40/0.35/0.25; missing HRV renormalizes; calm labels; serde omit; IPC round-trip.

## Crates / files touched
- `crates/bio-spec/src/{feature,lib}.rs`, `crates/bio-spec/tests/contracts.rs`
- `crates/feature-engine/src/catalog/{focus_score,stress_index,fatigue_index,context_switch_rate,meeting_density,recovery_between_meetings}.rs` (+ Feature literals in engine/alert/snapshot)
- `crates/knowledge-engine`, `crates/report-engine` (test Feature literals)
- `apps/desktop/src-tauri/src/lib.rs` (FeatureDto + factors tests)
- `docs/{decision-log,02-domain-model,06-feature-catalog,09-api,12-development,16-glossary,00-vision}.md`

## How to verify (commands)
```bash
cargo test -p bio-spec -p feature-engine -p knowledge-engine -p report-engine -p pipeline
cd apps/desktop/src-tauri && cargo test --lib
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Factor shape documented (id + calm label + share) in domain/contracts/catalog; no ADR unless beyond additive (decision-log note only)
- [ ] AC2: `FocusScore` emits factors with value + confidence + provenance
- [ ] AC3: `get_feature_snapshot` exposes factors or omit-until-present; UI↛SQLite; no new Feature SQLite schema
- [ ] AC4: Unit tests — share sum policy; thin/empty idle-safe
- [ ] AC5: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod; UI↛DB; glossary; calm copy

## Risks / not covered
- StressIndex / Fatigue / CSR / calendar Features do **not** emit factors yet (omit-until-present) — out of scope beyond ≥1 catalog Feature.
- Dashboard “Why?” UI not required.
- LLM explanations explicitly out of scope.

## Notes for QA
- Full FocusScore inputs → factors shares `0.40`, `0.35`, `0.25` (ids `typing`, `stability`, `hrv`).
- No HRV → two factors; shares `0.40/0.75` and `0.35/0.75`; sum ≈ 1.0.
- Empty `&[]` → no Features (idle-safe).
- IPC: empty factors → JSON key absent; present factors → camelCase `id`/`label`/`share`.
