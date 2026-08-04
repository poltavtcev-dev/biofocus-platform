# Dev → QA: P3-E1-T1

## Meta
- **Task ID:** P3-E1-T1
- **Title:** Pipeline crate skeleton + Observation intake
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E1 / P3-E1-T1; brief `docs/handoffs/P3-E1-T1-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Crate `pipeline` больше не stub-only: публичный intake API принимает batch `Observation` (`bio-spec`) и возвращает `Result<AcceptedBatch, PipelineError>`.
  - Успех → stage `PipelineStage::AcceptedForProcessing` (passthrough, порядок сохранён).
  - **Пустой batch → Ok** (idle-friendly; не Err).
  - Ошибки типизированы через `thiserror` (`IntakeRejected`, `StageFailed` — зарезервированы; default T1 path не reject’ит).
  - Unit-тесты: happy path (N=2), empty batch, owned/iter parity.
  - CI: `cargo test -p pipeline` добавлен в `rust-core` job.
- Crates / files:
  - `crates/pipeline/Cargo.toml` (`thiserror`; dev: `serde_json`, `uuid`)
  - `crates/pipeline/src/lib.rs` — re-exports + `PipelineStage`
  - `crates/pipeline/src/error.rs` — `PipelineError` / `PipelineResult`
  - `crates/pipeline/src/intake.rs` — `accept_observations` / `accept_owned` / `accept_iter` + tests
  - `.github/workflows/ci.yml`

## How to verify (commands)
```bash
cargo check -p pipeline
cargo test -p pipeline
# optional workspace (exclude desktop):
cargo check --workspace --exclude desktop
```

Ожидание: 3 unit-теста `pipeline` зелёные.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: публичный API принимает `&[Observation]` (и helpers owned/iter) → `Result` с `AcceptedBatch` / `PipelineError` (`thiserror`)
- [ ] AC2: unit-тесты happy path (N≥1) + пустой batch; контракт empty = **Ok**
- [ ] AC3: нет busy-loop / spin (чистая sync-функция, без I/O и потоков)
- [ ] AC4: нет Feature formulas, Menubar/UI, HTTP ingest changes, новой SQLite-таблицы
- [ ] AC5: entrypoint задокументирован (ниже + crate docs)
- [ ] Global DoD: нет `unwrap`/`expect` в production paths `crates/pipeline/src/` (только в `#[cfg(test)]`); UI↛DB; glossary `Observation`

## Entrypoint (для QA / следующих задач)
| Symbol | Role |
| :--- | :--- |
| `pipeline::accept_observations(&[Observation])` | **Primary** intake |
| `pipeline::accept_owned(Vec<Observation>)` | Owned batch без clone slice |
| `pipeline::accept_iter(impl IntoIterator<Item=Observation>)` | Iterator helper |
| `pipeline::AcceptedBatch` | Успех: `stage()`, `observations()`, `into_observations()` |
| `pipeline::PipelineStage::AcceptedForProcessing` | Маркер стадии T1 |
| `pipeline::PipelineError` | Structured errors (`thiserror`) |

## Risks / not covered
- Intake пока не валидирует/не reject’ит (variants `IntakeRejected` / `StageFailed` зарезервированы) — политика → T2+.
- Dedupe → **P3-E1-T2**; normalize → **T3**; runtime worker wire → **T4**.
- Feature DAG / Menubar — out of scope.

## Notes for QA
- `expect` в `intake::tests::sample_observation` — только test helper.
- Нет изменений `bio-spec` API (helpers не потребовались).
