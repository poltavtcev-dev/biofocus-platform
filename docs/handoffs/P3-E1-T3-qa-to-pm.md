# QA → PM: P3-E1-T3

## Meta
- **Task ID:** P3-E1-T3
- **Title:** Normalization & calibration
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E1-T3-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo check -p pipeline` → **ok**
  - `cargo test -p pipeline` → **18 passed** (3 intake + 7 dedupe + 8 normalize)
  - `cargo check --workspace --exclude desktop` → **ok**
- AC results (pass/fail per item):
  - **AC1 Pass** — known types `heart_rate` / `hrv` / `context_window` / `keystrokes` with explicit rules in `normalize.rs` module docs + unit tests (aliases, unit calibration hz→bpm / s→ms, extras stripped)
  - **AC2 Pass** — unknown `data_type` → **pass-through** (test `unknown_data_type_is_pass_through`); unparseable known → **skip** + `skipped_count` (test `heart_rate_missing_bpm_is_skipped`); documented in Dev handoff + module docs
  - **AC3 Pass** — unit tests per known type + unknown + empty + intake→dedupe→normalize chain
  - **AC4 Pass** — sync stage; no threads / timers / busy-loop
  - **AC5 Pass** — diff `0a4c70d` / PR #11: only `crates/pipeline/{normalize,lib,error,intake,Cargo.toml}` + Dev handoff. No Feature scores / Menubar / SQLite schema / runtime worker
  - **Global DoD Pass** — `unwrap`/`expect` only in `#[cfg(test)]`; UI↛DB; output still `Observation` payloads (not Feature)
- Extra checks (edge / security):
  - `context_window` strips `window_title` / `keystrokes` keys — privacy-safe
  - `keystrokes` recomputes `rate_per_min`, drops content-like extras
  - SQLite never touched (in-memory payload rewrite only)
  - Scope: worker → T4; Feature DAG → E2

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P3-E1-T3 → Done; Ready = **P3-E1-T4**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/ARCHITECTURE_STATUS.md` / `docs/12-development.md` (normalize entrypoint); `hrv` canon in `docs/07-contracts.md`; brief `P3-E1-T4-pm-brief.md`
- [x] Git: code on `main` via [PR #11](https://github.com/poltavtcev-dev/biofocus-platform/pull/11); QA/PM docs close → follow-up PR

## Suggested next Ready task
- **P3-E1-T4** — Runtime Feature Worker wire (idle-safe)

## Notes for PM
- Primary API: `pipeline::normalize_observations(&[Observation])` → `NormalizedBatch` (`PipelineStage::Normalized`); helpers `normalize_owned` / `normalize_deduped`.
- Typical flow: `accept_*` → `dedupe_*` → `normalize_*`.
- Accepted notes (не блокеры): (1) `hrv` canon (`rmssd_ms` + optional `sdnn_ms` / `pnn50`) was defined in T3 code — freeze into `07-contracts.md`; (2) alias/unit matrix is v1; (3) T2 notes still open: payload key-order fingerprint; seen-set bounds → T4/later.
