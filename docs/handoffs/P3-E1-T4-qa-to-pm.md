# QA → PM: P3-E1-T4

## Meta
- **Task ID:** P3-E1-T4
- **Title:** Runtime Feature Worker wire (idle-safe)
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P3-E1-T4-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo check -p runtime -p pipeline -p storage` → **ok**
  - `cargo test -p runtime -p pipeline -p storage` → **ok**
    - pipeline: **20 passed** (incl. `quality::` happy/empty)
    - runtime: `feature_worker_stop_freezes_poll_count` + `feature_worker_processes_queued_observations_then_idles` **ok**; host tests **5 ok**
    - storage: `list_after_created_cursor_incremental_and_idle_empty` **ok** (13 observation_repo + migrate/open)
  - `cargo check -p desktop` → **ok**
- AC results (pass/fail per item):
  - **AC1 Pass** — `spawn_feature_worker` → `poll_new` → `run_quality_pipeline` (accept → dedupe → normalize) → `FeatureHook::on_normalized`
  - **AC2 Pass** — empty poll → `recv_timeout(poll_interval)` (default 1s); no spin; zero interval rejected by `validate_feature_worker_config`
  - **AC3 Pass** — desktop `setup` → `start_feature_host`; `ExitRequested` → `stop_feature_host` before ingest shutdown
  - **AC4 Pass** — `NoopFeatureHook` stub; full DAG deferred to P3-E2
  - **AC5 Pass** — idle freeze after `stop` covered by integration test (poll counter freezes)
  - **Global DoD Pass** — no `unwrap`/`expect` in `feature_worker` / `feature_host` / storage cursor prod paths (`expect` only in tests); UI↛DB; no new SQLite schema (cursor on existing `created_at`); glossary `Observation`
- Extra checks (edge / security):
  - Soft-fail desktop: DB path/open failure → worker not started (logged), no panic
  - Separate WAL connection + tip cursor on launch (no historical replay) — intentional
  - Code on `main` via [PR #13](https://github.com/poltavtcev-dev/biofocus-platform/pull/13); CONTRIBUTING via [PR #14](https://github.com/poltavtcev-dev/biofocus-platform/pull/14)

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P3-E1-T4 → Done; Ready = **P3-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/ARCHITECTURE_STATUS.md` / `docs/12-development.md` (worker entrypoint); `docs/14-roadmap.md`; brief `P3-E2-T1-pm-brief.md`; mark T4 brief Done
- [ ] Git: code already on `main` (PR #13); QA/PM docs close → follow-up PR if desired

## Suggested next Ready task
- **P3-E2-T1** — DAG scheduler skeleton (`crates/feature-engine`)

## Notes for PM
- Primary API: `runtime::spawn_feature_worker(source, hook, config)` + `FeatureWorkerHandle::stop`; desktop `feature_host::{start,stop}_feature_host`.
- Pipeline wire: `pipeline::run_quality_pipeline`.
- Storage: `ObservationRepository::list_after_created_cursor` / `max_created_cursor` (no schema ADR).
- Accepted notes (не блокеры): (1) launch cursor = DB tip — backlog not replayed; (2) poll-only, no in-process notify from ingest; (3) T2 carry: unbounded dedupe seen-set / payload key-order fingerprint → later; (4) Feature DAG / scores / Menubar alerts → E2 / E3.
