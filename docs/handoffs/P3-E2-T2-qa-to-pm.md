# QA → PM: P3-E2-T2

## Meta
- **Task ID:** P3-E2-T2
- **Title:** `ContextSwitchRate` + `FocusScore` (v1)
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E2-T2-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/3-pipeline-features` @ `446a412`

## What was verified
- Commands run + results:
  - `cargo check -p feature-engine` → ok
  - `cargo test -p feature-engine` → **9 passed** (5 skeleton + 4 catalog: CSR rate, CSR empty, FocusScore high, switches ↓ FocusScore)
- AC results:
  - **AC1 Pass** — `FocusScore` window 15m / step 1m (`WINDOW_SECS=900`, `STEP_SECS=60`); `ContextSwitchRate` из `context_window` (`bundle_id` switches / 15). v1 упрощения задокументированы (Dev handoff + rustdoc): app_category → CSR stability; typing `rate_per_min`; HRV peak @ 45 ms RMSSD
  - **AC2 Pass** — Features с `provenance` Observation IDs (CSR: context in window; FocusScore: union keystrokes/hrv/context). Тест `counts_switches…` проверяет `provenance.len()==4`; FocusScore — contains keystroke/HRV IDs
  - **AC3 Pass** — синтетические unit-тесты в `catalog/context_switch_rate.rs` + `catalog/focus_score.rs` (`docs/11-testing.md` §1)
  - **AC4 Pass** — `register_focus_v1` → `FeatureEngine::register` / `run`; нет UI / Menubar / SQLite schema / ADR / Stress·Fatigue·Signals
  - **AC5 Pass** — Dev handoff на месте
  - **Global DoD Pass** — production без `unwrap`/`expect` (только `#[cfg(test)]`); UI↛DB; sync compute, idle-friendly empty snapshot
- Extra checks:
  - DAG: `FocusScoreNode` depends on `ContextSwitchRate`; `register_focus_v1` регистрирует в порядке CSR → FocusScore
  - Public API: `feature_engine::{register_focus_v1, ContextSwitchRateNode, FocusScoreNode, WINDOW_SECS, STEP_SECS}`
  - Scope risks из Dev handoff (T3 Stress/Fatigue, T4 E2E, FeatureHook wire) — **не блокеры**

## Defects (if any)
- Нет блокеров.
- **Note (docs):** `ContextSwitchRate` есть в `docs/02-domain-model.md`, но **нет** секции в `docs/06-feature-catalog.md` (там только FocusScore / Stress / Fatigue). Код + handoff достаточны для AC; PM может добавить catalog entry при удобном docs-pass (не блокирует Done).

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P3-E2-T2 → Done; Ready → **P3-E2-T3**; refresh Active assignment / execution order
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG (E2-T2 done → E2-T3 next)
- [ ] Other docs (optional): `docs/06-feature-catalog.md` — секция `ContextSwitchRate` + v1 note для FocusScore inputs; `docs/12-development.md` entrypoint `register_focus_v1` если ещё не отражено

## Suggested next Ready task
- **P3-E2-T3** — `StressIndex` + `FatigueIndex` (v1) + High_Stress Signal · Dev · `feature-engine` (+ `bio-spec` Severity)

## Notes for PM
- Код уже закоммичен на `phase/3-pipeline-features` (`P3-E2-T2: ContextSwitchRate + FocusScore catalog v1.`). Working tree был clean до этого отчёта; `P3-E2-T2-qa-to-pm.md` — новый файл для коммита кластера.
- PR кластера E2 — не обязателен после T2; можно копить до T3/T4 или открыть по запросу.
- Carry E1 (tip-cursor, unbounded dedupe) не регрессировали — вне scope crate.
