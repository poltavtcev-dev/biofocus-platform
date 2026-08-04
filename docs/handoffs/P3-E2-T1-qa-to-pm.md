# QA → PM: P3-E2-T1

## Meta
- **Task ID:** P3-E2-T1
- **Title:** DAG scheduler skeleton
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E2-T1-dev-to-qa.md`
- **Verdict:** Pass
- **Branch:** `phase/3-pipeline-features` @ `800e752`

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine` → **5 passed** (`empty_engine…`, `duplicate_node_id…`, `cycle_detected`, `unknown_dependency…`, `two_node_dag_runs_in_dependency_order`)
  - `cargo check -p feature-engine` → ok
- AC results:
  - **AC1 Pass** — `FeatureEngine::register` + `run(&[Observation])` → `EngineOutput { features, signals }`; trait `FeatureNode` / `ComputeContext` / `NodeOutput`
  - **AC2 Pass** — `FeatureEngineError` via `thiserror`; production paths без `unwrap`/`expect` (только `#[cfg(test)]`)
  - **AC3 Pass** — `two_node_dag_runs_in_dependency_order`: порядок `node_a`→`node_b`, upstream `FeatA` доступен зависимому узлу
  - **AC4 Pass** — нет UI / Menubar / SQLite schema / ADR / FocusScore·Stress formulas (только skeleton)
  - **AC5 Pass** — Dev handoff на месте
  - **Global DoD Pass** — UI↛DB; glossary `Observation`→`Feature`/`Signal`; sync run, idle-friendly empty DAG/snapshot
- Extra checks:
  - CI job включает `-p feature-engine`
  - Kahn cycle / unknown dep / duplicate id покрыты unit-тестами
  - Scope risks из Dev handoff (self-dep, multi-edge, catalog nodes, runtime wire) — **не блокеры** для T1

## Defects (if any)
- Нет.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P3-E2-T1 → Done; Ready → **P3-E2-T2**; refresh Active assignment / execution order
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG (E2-T1 done → E2-T2 next)
- [x] Other docs: `docs/12-development.md` Feature Engine entrypoint; brief `P3-E2-T2-pm-brief.md`; T1 brief → Done

## Suggested next Ready task
- **P3-E2-T2** — `ContextSwitchRate` + `FocusScore` (v1) · Dev · `crates/feature-engine`

## Notes for PM
- Код уже закоммичен на `phase/3-pipeline-features` (`P3-E2-T1: Feature Engine DAG scheduler skeleton.`). Working tree был clean на момент QA; этот отчёт — новый untracked/dirty файл для коммита кластера.
- PR кластера E2 — не требуется после одной T1; можно копить до E2-T2/T3 или открыть по запросу.
- Carry из E1 (tip-cursor, unbounded dedupe, payload fingerprint) не регрессировали — вне scope crate.
