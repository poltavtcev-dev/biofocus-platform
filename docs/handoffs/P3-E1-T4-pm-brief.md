# PM Brief → Dev: P3-E1-T4

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-04  
**Closed previous:** P3-E1-T3 (QA Pass with notes; PR #11 code · QA/PM docs close)

## Task
**P3-E1-T4 — Runtime Feature Worker wire (idle-safe)**

## Why
Intake → dedupe → normalize готовы как sync stages. Нужен Core worker, который подтягивает новые Observations и прогоняет pipeline без busy-loop, со стартом/стопом вместе с Desktop host. Полный Feature DAG — в E2; здесь допустима заглушка/hook.

## Acceptance Criteria
1. Worker в Core периодически или по событию читает новые Observations → прогоняет pipeline stages (`accept` → `dedupe` → `normalize`).
2. При отсутствии новых данных — sleep / event wait (нет spin / busy-loop). Idle footprint соблюдён.
3. Desktop host стартует/останавливает worker с приложением.
4. Заглушка/hook к feature-engine допустима (полный DAG → **P3-E2**).
5. Тест на idle freeze после stop (счётчики/вызовы не растут).
6. Handoff: `docs/handoffs/P3-E1-T4-dev-to-qa.md`.

## Out of scope
- Feature DAG / FocusScore math (→ **P3-E2**)
- Menubar alert levels (→ **P3-E3**)
- ADR / новая SQLite-схема для Features/Signals
- Changing normalize/dedupe rules (T2/T3 frozen unless bugfix)

## Constraints
- Production: no `unwrap` / `expect`
- Modules: `crates/runtime`, `pipeline`, host/`apps/desktop/src-tauri`
- UI ↛ SQLite; Features/Signals remain in-memory / derived until ADR
- Carry notes: dedupe seen-set bounds / TTL; payload fingerprint key-order (T2 QA)
- Branch: `phase/3-pipeline-features` (от свежего `main`)

## After QA Pass
PM → Ready **P3-E2-T1** (DAG scheduler skeleton).
