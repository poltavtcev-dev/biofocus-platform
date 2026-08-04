# PM Brief → Dev: P3-E1-T3

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes → PM close 2026-08-04)  
**Date:** 2026-08-04  
**Closed previous:** P3-E1-T2 (QA Pass with notes; PR #8 code · PR #9 QA handoff)  
**Evidence:** PR #11 (code) · `P3-E1-T3-qa-to-pm.md` (QA/PM docs close)

## Task
**P3-E1-T3 — Normalization & calibration**

## Why
После intake + dedupe нужен канонический payload/единицы для известных `data_type`, чтобы Feature Engine (E2) считал детерминированно. Без normalize worker (T4) и DAG получат сырой/разношёрстный JSON.

## Acceptance Criteria
1. Stage в `crates/pipeline` приводит payload/единицы к канону для известных типов: как минимум `heart_rate`, `hrv`, `context_window`, input aggregates (`keystrokes` / related) — **явные правила** в коде + тестах.
2. Неизвестный `data_type` → **pass-through или явный skip** (выбрать одно и зафиксировать в docs/handoff).
3. Unit-тесты на каждый известный type + unknown-type contract.
4. Idle-safe: sync stage, нет busy-loop.
5. Нет Feature scores / DAG math, Menubar/UI, новой SQLite-таблицы, runtime worker (→ T4).
6. Handoff: `docs/handoffs/P3-E1-T3-dev-to-qa.md`.

## Out of scope
- Runtime Feature Worker wire (→ **P3-E1-T4**)
- Feature DAG / FocusScore (→ **P3-E2**)
- Menubar alerts (→ **P3-E3**)
- ADR / schema changes
- Changing dedupe rule (T2 frozen unless bugfix)

## Constraints
- Production: no `unwrap` / `expect`
- Ubiquitous Language: `Observation` (normalized form still Observation payloads, not Feature)
- Build on T1/T2: typical flow `accept_*` → `dedupe_*` → normalize; add `PipelineStage::Normalized` (or equivalent)
- Branch: `phase/3-pipeline-features` (от свежего `main`)
- Notes from T2 QA (не блокеры T3): payload fingerprint key-order; seen-set bounds → T4/later

## After QA Pass
PM → Ready **P3-E1-T4** (runtime Feature Worker wire, idle-safe).
