# QA → PM: P3-E2-T3

## Meta
- **Task ID:** P3-E2-T3
- **Title:** `StressIndex` + `FatigueIndex` (v1) + High_Stress Signal
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E2-T3-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/3-pipeline-features` @ `d33e60f`

## What was verified
- Commands run + results:
  - `cargo check -p feature-engine` → ok
  - `cargo test -p feature-engine` → **16 passed** (5 skeleton + 4 focus + 5 stress + 2 fatigue)
- AC results (PM brief / Dev checklist):
  - **AC1 Pass** — `StressIndex` / `FatigueIndex` v1; упрощения в rustdoc + Dev handoff (RMSSD linear map; Fatigue: Focus + active minutes vs 8h + HR shift; weights renormalized)
  - **AC2 Pass** — contiguous minute samples `StressIndex > 75` с span `> 300` s → `Signal { type: High_Stress, severity: Severity::High }`. Тесты: emit after >5m; **no emit** at span == 300 s (strict `>`)
  - **AC3 Pass** — Features с provenance (Stress: HRV IDs; Fatigue: keystrokes/HR/context). Signals в `EngineOutput.signals` через `FeatureEngine::run` merge
  - **AC4 Pass** — unit-тесты порога High_Stress + базовые Stress/Fatigue (`docs/11-testing.md` §1)
  - **AC5 Pass** — nodes через DAG (`register_stress_v1` / `register_catalog_v1`); нет UI / SQLite schema / ADR; diff только `feature-engine` (+ uuid dep)
  - **AC6 Pass** — Dev handoff на месте
  - **Global DoD Pass** — production без `unwrap`/`expect` (только `#[cfg(test)]`); UI↛DB; glossary Feature/Signal; sync compute, idle-friendly empty snapshot
- Extra checks:
  - `FatigueIndex` depends on `FocusScore`; alone → `UnknownDependency` (`fatigue_requires_focus_score_node`)
  - Public API: `register_stress_v1`, `register_catalog_v1`, `StressIndexNode`, `FatigueIndexNode`, `HIGH_STRESS_*` constants
  - Scope risks (Baevsky SI, calendar-day active hours, E2E T4, Menubar E3, FeatureHook) — **не блокеры**

## Defects (if any)
- Нет блокеров.
- **Note (docs):** `docs/12-development.md` всё ещё пишет «Stress/Fatigue + High_Stress → T3» — после Done обновить entrypoints (`register_stress_v1` / `register_catalog_v1`).
- **Note (catalog):** `docs/06-feature-catalog.md` описывает Baevsky / full inputs; v1 simplifications живут в code rustdoc + handoff — PM может добавить v1 note при docs-pass (как для FocusScore на T2).

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P3-E2-T3 → Done; Ready → **P3-E2-T4**; refresh Active assignment / execution order; Epic P3-E2 T1–T3 Done
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG (E2-T3 done → E2-T4 next)
- [x] Other docs: `docs/12-development.md` entrypoints Stress/Fatigue; optional v1 note in `docs/06-feature-catalog.md`; brief `P3-E2-T4-pm-brief.md` если ещё нет

## Suggested next Ready task
- **P3-E2-T4** — Pipeline E2E (Observation → Feature / Signal) · QA (lead) + Dev · `pipeline` + `feature-engine`

## Notes for PM
- Код закоммичен: `d33e60f` (`P3-E2-T3: StressIndex + FatigueIndex v1 and High_Stress Signal.`). Working tree clean до этого отчёта; `P3-E2-T3-qa-to-pm.md` — новый файл для коммита кластера.
- PR кластера E2 — после T4 или по запросу; ветка `phase/3-pipeline-features` уже на origin.
- Carry E1 (tip-cursor, unbounded dedupe) не регрессировали — вне scope crate.
