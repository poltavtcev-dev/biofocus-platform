# PM Brief → Dev: P3-E1-T2

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes → PM close 2026-08-04)  
**Date:** 2026-08-04  
**Closed previous:** P3-E1-T1 (QA Pass with notes; PR #5 code · PR #6 handoff)  
**Evidence:** PR #8 (code) · PR #9 (QA handoff) · `P3-E1-T2-qa-to-pm.md`

## Task
**P3-E1-T2 — Deduplication stage**

## Why
Intake уже принимает `Observation` batch (`pipeline::accept_observations`). Следующий слой — убрать/пометить дубликаты до normalize и Feature Engine, не трогая immutable rows в SQLite.

## Acceptance Criteria
1. Stage в `crates/pipeline` удаляет или помечает дубликаты по **явному правилу** (задокументировать в коде + handoff): например same `id` already seen in window, или same `(provider_id, data_type, timestamp, payload hash)`.
2. Immutable Observations в БД **не** переписываются (in-memory / pipeline-only).
3. Unit-тесты: duplicate in-batch + cross-batch (in-memory seen-set).
4. Idle-safe: нет busy-loop / spin.
5. Нет Feature formulas, Menubar/UI, новой SQLite-таблицы, normalize logic (→ T3).
6. Handoff: `docs/handoffs/P3-E1-T2-dev-to-qa.md`.

## Out of scope
- Normalization / calibration (→ **P3-E1-T3**)
- Runtime worker wire (→ **T4**)
- Feature DAG (→ **P3-E2**)
- Menubar alerts (→ **P3-E3**)
- ADR / schema changes

## Constraints
- Production: no `unwrap` / `expect`
- Ubiquitous Language: `Observation`
- Build on T1 entrypoint (`accept_*` / `AcceptedBatch` / `PipelineStage`); можно добавить stage marker (напр. `Deduped`)
- Branch: `phase/3-pipeline-features` (от свежего `main`) или `epic/p3-e1-pipeline`
- Commit когда единица готова; PR — по кластеру E1 (T2+), не обязателен на один handoff

## After QA Pass
PM → Ready **P3-E1-T3** (normalization & calibration).
