# QA → PM: P3-E1-T2

## Meta
- **Task ID:** P3-E1-T2
- **Title:** Deduplication stage
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E1-T2-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo check -p pipeline` → **ok**
  - `cargo test -p pipeline` → **10 passed** (3 intake + 7 dedupe)
  - `cargo check --workspace --exclude desktop` → **ok**
- AC results (pass/fail per item):
  - **AC1 Pass** — явное правило: duplicate если already seen `id` **или** content key `(provider_id, data_type, timestamp_secs, payload.to_string())`; first wins; задокументировано в `dedupe.rs` module docs + crate docs + Dev handoff
  - **AC2 Pass** — нет storage/SQL в diff; только in-memory `DedupeState` (`HashSet`); SQLite rows не трогаются
  - **AC3 Pass** — unit-тесты in-batch (id + content) + cross-batch seen-set зелёные; также empty / distinct / intake wire / `clear()`
  - **AC4 Pass** — sync HashSet; нет threads / timers / spin / busy-loop
  - **AC5 Pass** — diff коммита `0453aa1`: только `crates/pipeline/{dedupe,lib,error,intake}.rs` + Dev handoff. Нет Feature formulas / Menubar / normalize logic / новой SQLite-таблицы
  - **Global DoD Pass** — `unwrap`/`expect` только в `#[cfg(test)]`; UI↛DB; glossary `Observation`
- Extra checks (edge / security):
  - Порядок kept сохраняется (first wins) — покрыто тестами id/content
  - `dedupe_accepted` wire от T1 intake — ок
  - `StageFailed` зарезервирован; default dedupe path всегда `Ok` — ок для T2
  - Scope: normalize → T3; worker → T4

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P3-E1-T2 → Done; Ready = **P3-E1-T3**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs (по необходимости): `docs/ARCHITECTURE_STATUS.md` / `docs/12-development.md` (dedupe entrypoint); brief для T3
- [ ] Git: код на `phase/3-pipeline-features` (`0453aa1`); PR по политике кластера E1 (T2+), не обязателен на один handoff

## Suggested next Ready task
- **P3-E1-T3** — Normalization & calibration

## Notes for PM
- Primary API: `pipeline::dedupe_observations(&mut DedupeState, &[Observation])` → `DedupedBatch` (`PipelineStage::Deduped`); helpers `dedupe_owned` / `dedupe_accepted`.
- Accepted notes (не блокеры): (1) payload fingerprint = compact `Value::to_string()` — разный порядок ключей JSON может дать разные fingerprints; (2) seen-set без TTL/bounds — достаточно для unit stage; runtime bounds → T4 / later.
- Типичный поток: `accept_observations` → `dedupe_accepted(&mut state, accepted)`.
