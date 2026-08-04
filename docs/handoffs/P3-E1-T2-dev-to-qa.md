# Dev → QA: P3-E1-T2

## Meta
- **Task ID:** P3-E1-T2
- **Title:** Deduplication stage
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E1 / P3-E1-T2; brief `docs/handoffs/P3-E1-T2-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Добавлена стадия dedupe в `crates/pipeline`: in-memory seen-set [`DedupeState`], API `dedupe_observations` / `dedupe_owned` / `dedupe_accepted`.
  - Маркер стадии `PipelineStage::Deduped`; результат `DedupedBatch` (kept + `dropped_count`).
  - **Явное правило:** дубликат, если уже seen **либо** тот же `Observation.id`, **либо** тот же content key `(provider_id, data_type, timestamp_secs, payload.to_string())`. First wins; порядок kept сохранён.
  - SQLite / persisted Observations **не** трогаются (только pipeline view).
  - Unit-тесты: in-batch (id + content), cross-batch seen-set, empty batch Ok, intake→dedupe wire, `clear()`.
- Crates / files:
  - `crates/pipeline/src/dedupe.rs` — **new** (rule + state + API + tests)
  - `crates/pipeline/src/lib.rs` — `PipelineStage::Deduped`, re-exports, crate docs
  - `crates/pipeline/src/error.rs` / `intake.rs` — комментарии (без смены контракта intake)

## Deduplication rule (документация для QA)

| Check | Duplicate when… |
| :--- | :--- |
| Id | `id` уже в `DedupeState.seen_ids` |
| Content | `(provider_id, data_type, timestamp, payload JSON string)` уже в `seen_content` |

Payload fingerprint = compact `serde_json::Value::to_string()` (равные Value, собранные одинаково → одинаковый ключ).  
Окно = lifetime одного `DedupeState` (cross-batch); `clear()` сбрасывает.

## How to verify (commands)
```bash
cargo check -p pipeline
cargo test -p pipeline
# optional workspace (exclude desktop):
cargo check --workspace --exclude desktop
```

Ожидание: **10** unit-тестов `pipeline` зелёные (3 intake + 7 dedupe).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: stage удаляет дубликаты по **явному** правилу (id **или** content key) — задокументировано в `dedupe.rs` module docs + этот handoff
- [ ] AC2: immutable Observations в БД не переписываются (нет storage/SQL в diff; только in-memory `DedupeState`)
- [ ] AC3: unit-тесты duplicate **in-batch** + **cross-batch** (seen-set) присутствуют и зелёные
- [ ] AC4: idle-safe — sync HashSet, нет busy-loop / spin / потоков
- [ ] AC5: нет Feature formulas, Menubar/UI, normalize logic, новой SQLite-таблицы
- [ ] Global DoD: нет `unwrap`/`expect` в production paths `crates/pipeline/src/` (только `#[cfg(test)]`); UI↛DB; glossary `Observation`

## Entrypoint (для QA / T3)
| Symbol | Role |
| :--- | :--- |
| `pipeline::DedupeState` | In-memory seen window (`new` / `clear` / counts) |
| `pipeline::dedupe_observations(&mut state, &[Observation])` | **Primary** dedupe API |
| `pipeline::dedupe_owned` / `dedupe_accepted` | Owned / post-intake helpers |
| `pipeline::DedupedBatch` | Успех: `stage()`, `observations()`, `dropped_count()`, `into_observations()` |
| `pipeline::PipelineStage::Deduped` | Маркер стадии T2 |
| `pipeline::accept_*` / `AcceptedBatch` | T1 intake (без изменений контракта) |

Типичный поток: `accept_observations` → `dedupe_accepted(&mut state, accepted)`.

## Risks / not covered
- Payload key order: два логически равных JSON с разным порядком ключей могут дать разные fingerprints — acceptable for T2; зафиксировано.
- Нет TTL / bounded window size для seen-set (растёт с уникальными ключами) — достаточно для unit stage; runtime bounds → T4 / later.
- Normalize → **P3-E1-T3**; worker wire → **T4**; Feature DAG — out of scope.
- `StageFailed` для dedupe по-прежнему reserved (default path всегда `Ok`).

## Notes for QA
- `expect` только в `#[cfg(test)]` helpers (`obs_with` / intake samples).
- Коммит не обязателен до закрытия кластера E1; дерево может быть dirty до PR.
