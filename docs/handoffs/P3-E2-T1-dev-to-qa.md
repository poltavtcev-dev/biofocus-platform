# Dev → QA: P3-E2-T1

## Meta
- **Task ID:** P3-E2-T1
- **Title:** DAG scheduler skeleton
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E2 / P3-E2-T1; brief `docs/handoffs/P3-E2-T1-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Crate `feature-engine` больше не stub-only: публичный API регистрирует узлы DAG (`FeatureNode`) и выполняет топологический прогон на in-memory snapshot `&[Observation]` → `EngineOutput { features, signals }`.
  - Ошибки через `thiserror`: duplicate id, unknown dependency, cycle, node failure.
  - Unit-тест **2-node DAG order** (+ empty, duplicate, unknown dep, cycle).
  - CI: `cargo test -p feature-engine` в `rust-core` job.
  - Нет UI / Menubar / новой SQLite-схемы / catalog formulas.
- Crates / files:
  - `crates/feature-engine/Cargo.toml` (`thiserror`; dev: `serde_json`, `uuid`)
  - `crates/feature-engine/src/lib.rs` — entrypoint docs + re-exports
  - `crates/feature-engine/src/error.rs` — `FeatureEngineError` / `FeatureEngineResult`
  - `crates/feature-engine/src/node.rs` — `FeatureNode`, `ComputeContext`, `NodeOutput`
  - `crates/feature-engine/src/engine.rs` — `FeatureEngine`, Kahn topo + tests
  - `.github/workflows/ci.yml`

## How to verify (commands)
```bash
cargo check -p feature-engine
cargo test -p feature-engine
# optional workspace (exclude desktop):
cargo check --workspace --exclude desktop
```

Ожидание: 5 unit-тестов `feature-engine` зелёные (включая `two_node_dag_runs_in_dependency_order`).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: API регистрирует узлы DAG; `run(&[Observation])` → `Vec<Feature>` (+ optional `Vec<Signal>` via `EngineOutput`)
- [ ] AC2: ошибки через `Result` / `thiserror`; нет `unwrap`/`expect` в production paths
- [ ] AC3: unit-тест на 2-node DAG order (topo соблюдён; upstream Feature виден зависимому узлу)
- [ ] AC4: нет UI / Menubar / новой SQLite-схемы / ADR; нет FocusScore/Stress formulas
- [ ] AC5: handoff этот файл
- [ ] Global DoD: UI↛DB; glossary `Observation` → `Feature` / `Signal`; idle-safe (sync, no spin)

## Entrypoint (для QA / следующих задач)
| Symbol | Role |
| :--- | :--- |
| `feature_engine::FeatureEngine::new` | Empty DAG |
| `feature_engine::FeatureEngine::register` | Register `FeatureNode` (unique id) |
| `feature_engine::FeatureEngine::run` | **Primary** topo compute on Observation snapshot |
| `feature_engine::FeatureNode` | Trait: `id` / `depends_on` / `compute` |
| `feature_engine::ComputeContext` | Observations + upstream Features/Signals |
| `feature_engine::NodeOutput` | Per-node Features + Signals |
| `feature_engine::EngineOutput` | Aggregated run result |
| `feature_engine::FeatureEngineError` | Structured errors (`thiserror`) |

## Risks / not covered
- Нет catalog nodes (`FocusScore`, `ContextSwitchRate`) → **P3-E2-T2**.
- Stress/Fatigue + `High_Stress` Signal → **P3-E2-T3**.
- Wire `runtime::FeatureHook` → real engine / desktop — out of scope (brief).
- Self-dependency / multi-edge duplicates not specially tested (would inflate indegree / cycle-like).
- `expect` только в `#[cfg(test)]`.

## Notes for QA
- Пустой engine / пустой snapshot → `Ok` + empty output (как pipeline idle).
- Kahn sort стабилизирует порядок среди равных indegree по индексу регистрации.
- Carry notes E1 (tip-cursor, unbounded dedupe, payload fingerprint) не трогались — вне crate.
