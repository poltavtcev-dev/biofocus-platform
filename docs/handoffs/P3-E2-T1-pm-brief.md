# PM Brief → Dev: P3-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass; PM close 2026-08-04)  
 
**Date:** 2026-08-04  
**Closed previous:** P3-E1-T4 (QA Pass with notes; PR #13 code · QA/PM docs close)

## Task
**P3-E2-T1 — DAG scheduler skeleton**

## Why
E1 закрыт: Observations проходят quality pipeline через idle-safe Feature Worker (`NoopFeatureHook`). Нужен каркас Feature Engine — регистрация узлов DAG и топологический прогон на нормализованном in-memory snapshot → `Feature` (+ optional `Signal`). Без UI и без полной math catalog (→ T2+).

## Acceptance Criteria
1. `crates/feature-engine` API: зарегистрировать узлы DAG, топологический прогон на in-memory snapshot нормализованных Observations → `Vec<Feature>` (+ optional `Vec<Signal>`).
2. Ошибки через `Result` / `thiserror` (нет `unwrap`/`expect` в prod).
3. Unit-тест на 2-node DAG order (топология соблюдена).
4. Нет UI / Menubar / новой SQLite-схемы / ADR.
5. Handoff: `docs/handoffs/P3-E2-T1-dev-to-qa.md`.

## Out of scope
- `FocusScore` / `ContextSwitchRate` formulas (→ **P3-E2-T2**)
- Stress/Fatigue + High_Stress Signal (→ **P3-E2-T3**)
- E2E Observation → Feature suite (→ **P3-E2-T4**)
- Wire FeatureHook → real engine in desktop (можно stub trait alignment; полный wire допустим позже в E2)
- Menubar alerts (→ **P3-E3**)

## Constraints
- Production: no `unwrap` / `expect`
- Ubiquitous Language: `Observation` → `Feature` / `Signal` (см. `/docs/16-glossary.md`, catalog `/docs/06-feature-catalog.md`)
- Modules: `crates/feature-engine` (primary); touch `bio-spec` only if types already require it
- Features/Signals in-memory / derived until ADR
- Branch: `phase/3-pipeline-features` (от свежего `main`)
- Carry notes from E1: tip-cursor (no backlog replay); unbounded dedupe seen-set; payload key-order fingerprint

## After QA Pass
PM → Ready **P3-E2-T2** (`ContextSwitchRate` + `FocusScore` v1).
