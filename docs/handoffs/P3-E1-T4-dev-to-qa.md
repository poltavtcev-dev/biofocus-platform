# Dev → QA: P3-E1-T4

## Meta
- **Task ID:** P3-E1-T4
- **Title:** Runtime Feature Worker wire (idle-safe)
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E1 / P3-E1-T4; brief `docs/handoffs/P3-E1-T4-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Core **Feature Worker** in `crates/runtime`: polls an [`ObservationSource`], runs pipeline quality stages (`accept` → `dedupe` → `normalize` via [`pipeline::run_quality_pipeline`]), then a **noop Feature Engine hook** (`NoopFeatureHook`).
  - **Idle-safe:** empty poll → `recv_timeout(poll_interval)` (default 1s); no spin. Stop via oneshot → join; tests assert poll/hook counters freeze.
  - **Storage cursor** (no schema change): `ObservationRepository::list_after_created_cursor` + `max_created_cursor` on existing `created_at`.
  - **Desktop host** starts/stops worker with the app (`feature_host::{start,stop}_feature_host`); SQLite source opens at tip (skips historical backlog on launch).
- Crates / apps / files:
  - `crates/runtime/src/feature_worker.rs` (+ exports in `lib.rs`, errors, `Cargo.toml` → `pipeline`)
  - `crates/runtime/tests/feature_worker.rs` — idle freeze + process-then-idle
  - `crates/pipeline/src/quality.rs` — `run_quality_pipeline`
  - `crates/storage/src/observation_repo.rs` — cursor list + `ObservationCreated`
  - `crates/storage/tests/observation_repo.rs` — cursor coverage
  - `apps/desktop/src-tauri/src/feature_host.rs` + wire in `lib.rs` setup / `ExitRequested`

## How to verify (commands)
```bash
cargo check -p runtime -p pipeline -p storage
cargo test -p runtime -p pipeline -p storage
cargo check -p desktop
# optional:
cargo check --workspace --exclude desktop
```

Ожидание:
- `feature_worker_stop_freezes_poll_count` + `feature_worker_processes_queued_observations_then_idles` — green
- `list_after_created_cursor_incremental_and_idle_empty` — green
- `pipeline` quality unit tests — green

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Worker в Core читает новые Observations → прогоняет pipeline (`accept` → `dedupe` → `normalize`)
- [ ] AC2: Нет новых данных → sleep / event wait (`recv_timeout`); нет busy-loop / spin
- [ ] AC3: Desktop host стартует worker в `setup`, останавливает на `ExitRequested`
- [ ] AC4: Заглушка Feature Engine (`NoopFeatureHook`); полный DAG → P3-E2
- [ ] AC5: Тест idle freeze после `stop` (счётчик poll не растёт)
- [ ] Global DoD: нет `unwrap`/`expect` в prod paths (`feature_worker`, `feature_host`, storage cursor); UI↛DB; нет новой SQLite-схемы; glossary `Observation`

## Entrypoint (для QA / E2)
| Symbol | Role |
| :--- | :--- |
| `runtime::spawn_feature_worker(source, hook, config)` | Primary worker spawn |
| `runtime::FeatureWorkerHandle::stop` | Graceful stop + join |
| `runtime::ObservationSource` / `FeatureHook` | Inject storage / Feature Engine |
| `runtime::NoopFeatureHook` | Stub until P3-E2 |
| `pipeline::run_quality_pipeline` | accept → dedupe → normalize |
| `storage::ObservationRepository::list_after_created_cursor` | Incremental read |
| `feature_host::start_feature_host` / `stop_feature_host` | Desktop lifecycle |

## Risks / not covered
- Worker cursor starts at **DB tip** on launch (backlog not replayed). Intentional for always-on menubar footprint.
- Separate SQLite connection from ingest persist (WAL); no shared in-process notify — poll interval only.
- Feature DAG / scores / Menubar alerts — out of scope (→ E2 / E3).
- Dedupe seen-set still unbounded in-process (carry note from T2 QA).

## Notes for QA
- `expect` only in `#[cfg(test)]` / integration test helpers.
- Soft-fail on desktop: if default DB path/open fails, Feature Worker is not started (logged); Menubar still works via ingest/`get_status`.
- Manual smoke (optional): launch desktop → exit → no hung thread (worker join on `ExitRequested` before ingest shutdown).
