# PM Brief → Dev: P3-E2-T3

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass with notes; PM close 2026-08-04)  
**Date:** 2026-08-04  
**Closed previous:** P3-E2-T2 (QA Pass with notes; CSR + FocusScore v1)

## Task
**P3-E2-T3 — `StressIndex` + `FatigueIndex` (v1) + High_Stress Signal**

## Why
Focus catalog на месте (`register_focus_v1`). Нужны Stress/Fatigue Features и transient `Signal` при пороге High_Stress — вход для Menubar alerts (E3).

## Acceptance Criteria
1. Реализованы v1 по `/docs/06-feature-catalog.md`: `StressIndex` / `FatigueIndex` (упрощения inputs OK если задокументированы в handoff + rustdoc).
2. Порог catalog: `StressIndex` > 75 дольше 5 мин → transient `Signal` с `signal_type` = `High_Stress`; severity согласована с `bio-spec::Severity` (рекомендация: `Severity::High`).
3. Output Features + provenance Observation IDs; Signals в `EngineOutput.signals`.
4. Unit-тесты на порог High_Stress + базовые значения Stress/Fatigue (`docs/11-testing.md`).
5. Nodes через существующий DAG API; нет UI / новой SQLite-схемы / ADR.
6. Handoff: `docs/handoffs/P3-E2-T3-dev-to-qa.md`.

## Out of scope
- Pipeline E2E Observation → Feature/Signal suite (→ **P3-E2-T4**)
- Alert level mapping / Menubar (→ **P3-E3**)
- Wire `FeatureHook` → real engine in desktop (можно позже в E2)
- Persistence Features/Signals (нужен ADR)

## Constraints
- Production: no `unwrap` / `expect`
- Modules: `crates/feature-engine` (primary); `bio-spec` только для `Severity` / `Signal` типов
- Features/Signals stay in-memory / derived until ADR
- Branch: `phase/3-pipeline-features`
- Catalog: `/docs/06-feature-catalog.md`
- Carry: tip-cursor / unbounded dedupe (E1) — вне scope; CSR catalog note закрыта PM docs-pass

## After QA Pass
PM → Ready **P3-E2-T4** (pipeline E2E). PR кластера E2 — по готовности T3/T4 или по запросу.
