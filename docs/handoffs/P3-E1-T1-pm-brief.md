# PM Brief → Dev: P3-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Opened:** Phase 3 (Sprint 5–6)  
**Closed previous:** Phase 2 / PR #2

## Task
**P3-E1-T1 — Pipeline crate skeleton + Observation intake**

## Why
Phase 2 уже пишет immutable `Observation` в SQLite. Phase 3 начинается с двери в Core: crate `pipeline` принимает batch и готовит почву для dedupe → normalize → Features. Без скелета нельзя наращивать стадии и Feature DAG.

## Acceptance Criteria
1. `crates/pipeline` — рабочий (не stub-only) публичный API: принять batch / slice `Observation` (`bio-spec`) → `Result` с явным успехом («принято в обработку» / passthrough struct) или ошибкой через `thiserror`.
2. Unit-тесты: happy path (N≥1), пустой batch (зафиксировать контракт: Ok empty vs Err).
3. Idle: нет spin-loop в API (чистая функция / sync stage OK).
4. Нет Feature formulas, Menubar/UI, HTTP ingest changes, новой SQLite-таблицы.
5. Handoff: `docs/handoffs/P3-E1-T1-dev-to-qa.md`.

## Out of scope
- Deduplication (→ **P3-E1-T2**)
- Normalization (→ **T3**)
- Runtime worker / desktop host wire (→ **T4**)
- Feature DAG / FocusScore (→ **P3-E2**)
- Menubar alerts (→ **P3-E3**)
- ADR / schema changes

## Constraints
- Production: no `unwrap` / `expect`
- Ubiquitous Language: `Observation` (input); ещё не обязаны эмитить `Feature`/`Signal`
- Branch: `phase/3-pipeline-features` или `epic/p3-e1-pipeline`
- Commit когда единица готова; PR — по кластеру E1 (или шире), не на каждый handoff

## After QA Pass
PM → Ready **P3-E1-T2** (deduplication stage).
