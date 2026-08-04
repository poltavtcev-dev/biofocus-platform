# Dev → QA: P3-E2-T2

## Meta
- **Task ID:** P3-E2-T2
- **Title:** `ContextSwitchRate` + `FocusScore` (v1)
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E2 / P3-E2-T2; brief `docs/handoffs/P3-E2-T2-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Catalog nodes in `feature-engine::catalog`: `ContextSwitchRateNode` + `FocusScoreNode` (DAG: CSR → FocusScore).
  - Helper `register_focus_v1(&mut FeatureEngine)` регистрирует оба узла.
  - Sliding window **15m / step 1m**; Features + `provenance` Observation IDs.
  - Unit-тесты на синтетических данных (CSR rate, high FocusScore, switches ↓ FocusScore).
  - Нет UI / Menubar / новой SQLite-схемы / ADR / Stress·Fatigue·Signals.
- Crates / files:
  - `crates/feature-engine/src/catalog/mod.rs` — `register_focus_v1` + re-exports
  - `crates/feature-engine/src/catalog/window.rs` — 15m/1m helpers
  - `crates/feature-engine/src/catalog/context_switch_rate.rs` — CSR v1 + tests
  - `crates/feature-engine/src/catalog/focus_score.rs` — FocusScore v1 + tests
  - `crates/feature-engine/src/lib.rs` — public API / docs
  - `crates/feature-engine/src/node.rs` — doc link to catalog

## v1 formulas (simplifications)

| Feature | Rule |
| :--- | :--- |
| **ContextSwitchRate** | `context_window` sorted by time; switch = consecutive `bundle_id` change; value = `switches / 15` (per nominal window minute). Empty context → no Feature for step. |
| **FocusScore** | Weighted (renormalized if missing): typing `mean(rate_per_min)/200*100` (0.40) + stability from upstream CSR `100 - rate*50` (0.35; replaces app_category taxonomy) + HRV comfort peak 100 @ 45 ms RMSSD (0.25). Output clamped 0–100. |

## How to verify (commands)
```bash
cargo check -p feature-engine
cargo test -p feature-engine
```

Ожидание: **9** unit-тестов зелёные (5 skeleton + 4 catalog).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: v1 `FocusScore` (window 15m / step 1m) + `ContextSwitchRate` из `context_window`; упрощения задокументированы (этот handoff + module rustdoc)
- [ ] AC2: Output Features + `provenance` Observation IDs
- [ ] AC3: Unit-тесты на синтетических данных (`docs/11-testing.md` §1)
- [ ] AC4: Nodes через существующий DAG API (`FeatureNode` / `register` / `run`); нет UI / новой SQLite-схемы / ADR
- [ ] AC5: handoff этот файл
- [ ] Global DoD: no `unwrap`/`expect` в production paths; UI↛DB; glossary terms; idle-safe (sync compute, no spin)

## Entrypoint
| Symbol | Role |
| :--- | :--- |
| `feature_engine::register_focus_v1` | Register CSR + FocusScore |
| `feature_engine::ContextSwitchRateNode` | Catalog node / Feature id `ContextSwitchRate` |
| `feature_engine::FocusScoreNode` | Catalog node / Feature id `FocusScore` (depends on CSR) |
| `feature_engine::WINDOW_SECS` / `STEP_SECS` | 900 / 60 |
| `feature_engine::FeatureEngine::run` | Topo compute on Observation snapshot |

## Risks / not covered
- Нет app_category taxonomy — stability через CSR (см. формулу).
- Stress/Fatigue + `High_Stress` Signal → **P3-E2-T3**.
- E2E Observation → Feature suite → **P3-E2-T4**.
- Wire `FeatureHook` → real engine / desktop — out of scope (brief).
- Много minute-step Features на длинном snapshot (до ~24h cap в helper) — OK for v1; не персист.
- `expect` только в `#[cfg(test)]`.

## Notes for QA
- Пустой snapshot / нет релевантных типов → `Ok` + empty (или без catalog Features).
- CSR и FocusScore эмитят **по одному Feature на каждый minute-aligned end**, где есть данные в окне.
- Carry notes E1 (tip-cursor, unbounded dedupe) не трогались.
