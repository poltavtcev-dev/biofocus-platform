# Dev|UX → QA: P3-E2-T4

## Meta
- **Task ID:** P3-E2-T4
- **Title:** Pipeline E2E (Observation → Feature / Signal)
- **Role that built:** QA (lead; authored suite per PM brief)
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P3-E2-T4; `docs/handoffs/P3-E2-T4-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Integration E2E suite: fixture Observation JSON → `run_quality_pipeline` → `register_catalog_v1` → Features/Signals with asserted values (`docs/11-testing.md` §5).
- Happy path + two edges (empty batch; no High_Stress below threshold).
- Documented how to run the suite in `docs/12-development.md` (§ Pipeline E2E).
- No production API changes; Features/Signals remain derived/in-memory; no UI / Menubar / SQLite schema / ADR.

## Crates / apps / files touched
- `crates/pipeline/Cargo.toml` — `feature-engine` as `[dev-dependencies]`
- `crates/pipeline/tests/pipeline_e2e.rs` — E2E tests
- `crates/pipeline/tests/fixtures/e2e_happy_path.json` — mock Observation stream
- `docs/12-development.md` — § Pipeline E2E commands + coverage notes

## How to verify (commands)
```bash
cargo test -p pipeline --test pipeline_e2e
cargo test -p pipeline
cargo test -p feature-engine
```

## Acceptance Criteria checklist (for QA)
- [x] AC1 — E2E/integration: fixture → pipeline → feature-engine → Features/Signals with checkable values
- [x] AC2 — Happy path + edge(s): empty batch (idle Ok) **and** no High_Stress below threshold (documented)
- [x] AC3 — How to run suite in `docs/12-development.md` (`cargo test -p pipeline --test pipeline_e2e`)
- [x] AC4 — No UI / Menubar / new SQLite schema / ADR; Features/Signals in-memory
- [x] AC5 — This handoff
- [x] Global DoD — no new production `unwrap`/`expect`; UI↛DB; idle-safe (sync, no busy-loop)

## Risks / not covered
- Desktop `FeatureHook` → live engine wire (out of scope; follow-up / E3 path).
- Persistence of Features/Signals (needs ADR).
- Alert level mapping / Menubar (→ P3-E3).
- Carry E1 tip-cursor / unbounded dedupe — not touched.

## Notes for QA
- Fixture uses non-canonical aliases (`rmssd`, `bundleId`/`appName`, `hr`, `window_seconds`) so normalize is exercised end-to-end; duplicate HRV id proves dedupe (15 → 14).
- Primary documented edge for AC2: empty batch; secondary: no High_Stress when RMSSD high.
