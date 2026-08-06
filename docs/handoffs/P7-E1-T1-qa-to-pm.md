# QA → PM: P7-E1-T1

## Meta
- **Task ID:** P7-E1-T1
- **Title:** Feature confidence contract + ADR + wire
- **Date:** 2026-08-06
- **Dev/UX handoff:** `docs/handoffs/P7-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- **Commands:**
  - `cargo test -p bio-spec -p feature-engine -p knowledge-engine -p report-engine -p pipeline` — all passed (earlier in build)
  - `cargo test -p feature-engine -- catalog::confidence … rich/missing/idle … stress low_observation` — 8/8 ok
  - `cd apps/desktop/src-tauri && cargo test --lib` — 26/26 ok (confidence field asserted on snapshot DTO)
- **AC results:**
  - **AC1 Pass** — ADR-007 in `docs/decision-log.md` (table + detail): relationship Observation vs Feature confidence; v1 `coverage × mean_obs`; omit empty / lower on partial; rejected UI-only heuristics, second registry, SQLite Feature rows for v1
  - **AC2 Pass** — `bio_spec::Feature.confidence: Confidence`; IPC `FeatureDto.confidence: f64`; storage migrate still only `observations` (no Feature schema)
  - **AC3 Pass** — all `register_catalog_v1` nodes compute confidence via shared helpers; empty windows omit Features; partial FocusScore → lower coverage
  - **AC4 Pass** — rich (3/3) > missing HRV (2/3) > typing-only (1/3); low obs confidence lowers StressIndex; idle `&[]` → empty
  - **AC5 Pass** — domain / catalog / API / glossary / vision / development notes updated
  - **AC6 Pass** — `docs/handoffs/P7-E1-T1-dev-to-qa.md` present
  - **Global DoD Pass** — no new unwrap/expect in prod confidence path (`saturating_from`); UI↛DB; calm “data quality” wording; Ubiquitous Language intact
- **Extra:** snapshot serde includes `confidence`; desktop DTO test asserts `confidence: 1.0`

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P7-E1-T1 → Done; Ready **P7-E2-T1** (Explanation factors)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS.md` note Feature confidence shipped (ADR-007); fold into next code PR with `phase/7-trust-layer`

## Suggested next Ready task
- **P7-E2-T1** — Explanation factors (per PM brief / sprint roadmap)

## Notes for PM
- Branch: `phase/7-trust-layer` (uncommitted working tree at QA time — commit/PR with Phase 7 cluster when ready; no PR required for handoff alone).
- Out of scope confirmed: explanation factors, new bio Features, Dashboard redesign.
