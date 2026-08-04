# PM Brief → QA + Dev: P3-E2-T4

**From:** PM  
**To:** QA (lead) + Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E2-T3 (QA Pass with notes; StressIndex + FatigueIndex + High_Stress)

## Task
**P3-E2-T4 — Pipeline E2E (Observation → Feature / Signal)**

## Why
Catalog Features v1 и High_Stress Signal готовы. Нужен integration/E2E путь: mock Observation stream → quality pipeline → Feature Engine → проверяемые Features/Signals (`docs/11-testing.md` §5).

## Acceptance Criteria
1. E2E/integration suite: mock Observation stream или fixture file → `pipeline` (accept → dedupe → normalize) → `feature-engine` (`register_catalog_v1` или эквивалент) → Features/Signals с проверяемыми значениями.
2. Покрытие минимум happy path + один негативный/edge (пустой batch / unknown type pass-through / нет High_Stress ниже порога — на выбор, задокументировать в handoff).
3. Как гонять suite — секция в `docs/12-development.md` (команды `cargo test …`).
4. Нет UI / Menubar / новой SQLite-схемы / ADR; Features/Signals остаются derived/in-memory.
5. Handoff: `docs/handoffs/P3-E2-T4-dev-to-qa.md` (или QA-authored, если QA ведёт реализацию тестов).

## Out of scope
- Alert level mapping / IPC / Menubar (→ **P3-E3**)
- Wire desktop `FeatureHook` → live engine (можно follow-up; не блокер AC)
- Persistence Features/Signals (нужен ADR)
- Полный Baevsky / calendar-day Fatigue (catalog later)

## Constraints
- Production: no `unwrap` / `expect`
- Modules: `crates/pipeline` + `crates/feature-engine` (+ storage fixtures если нужны)
- Idle-safe: suite не крутит busy-loop
- Branch: `phase/3-pipeline-features`
- Carry: tip-cursor / unbounded dedupe (E1) — вне scope; не регрессировать без явного fix

## After QA Pass
PM → Ready **P3-E3-T1** (alert level mapping). PR кластера E2 — после T4 или по запросу.
