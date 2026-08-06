# QA → PM: P7-E2-T1

## Meta
- **Task ID:** P7-E2-T1
- **Title:** Explanation factors on Features
- **Date:** 2026-08-06
- **Dev/UX handoff:** `docs/handoffs/P7-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p bio-spec -p feature-engine -p knowledge-engine -p report-engine -p pipeline` → **Pass** (all green; feature-engine 47 tests incl. factor cases)
  - `cd apps/desktop/src-tauri && cargo test --lib` → **Pass** (27 tests; includes `feature_snapshot_dto_exposes_factors_when_present`)
  - Targeted: `factors_renormalize_when_hrv_missing`, `factors_labels_are_calm_non_clinical`, `signal_feature_insight_serde_smoke`, IPC factors test → **Pass**
- AC results:
  - **AC1 Pass** — Factor shape `{ id, label, share }` in `bio_spec::ExplanationFactor` + `docs/02-domain-model.md` + FocusScore catalog note + `docs/09-api.md`. Decision-log note under ADR-007 (no separate ADR — additive optional field). Calm labels only.
  - **AC2 Pass** — `FocusScore` emits factors alongside value + confidence + provenance. Full window: shares `0.40` / `0.35` / `0.25` for `typing` / `stability` / `hrv`.
  - **AC3 Pass** — `get_feature_snapshot` FeatureDto serializes `factors` when present; omits key when empty (`skip_serializing_if`). No UI→SQLite. `migrate.rs` unchanged (no Feature factors schema).
  - **AC4 Pass** — Share-sum policy tested (full + renormalize missing HRV); empty `&[]` → no Features (`idle_empty_snapshot_emits_no_features`).
  - **AC5 Pass** — `docs/handoffs/P7-E2-T1-dev-to-qa.md` present.
  - **Global DoD Pass** — no `unwrap`/`expect` in FocusScore prod path; calm non-clinical factor labels; glossary + vision L2 updated.
- Extra checks:
  - Other catalog Features keep empty factors (omit-until-present) — acceptable per AC (≥1 emitter).
  - Out of scope respected: no LLM explanations, no Dashboard Why? redesign, no new bio Features.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move P7-E2-T1 to Done; Ready **P7-E3-T1**; Epic P7-E2 ✅ if T1 is the only task
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` Trust-layer note (optional); cluster PR note already on `phase/7-trust-layer`

## Suggested next Ready task
- **P7-E3-T1** — First bio-backed Trust Features (per brief After QA Pass)

## Notes for PM
- Code + docs already on branch `phase/7-trust-layer` (uncommitted until cluster commit). Fold with P7-E1 confidence into one Phase 7 cluster PR when Ready to ship.
- Stress/Fatigue/CSR/calendar still omit factors — intentional; extend later if product needs “Why?” beyond FocusScore.
