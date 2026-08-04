# QA → PM: P2-E1-T2

## Meta
- **Task ID:** P2-E1-T2
- **Title:** Pairing token persistence
- **Date:** 2026-08-04
- **Dev/UX handoff:** `docs/handoffs/P2-E1-T2-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/2-ingest-http` (T1+T2 на ветке; commit/PR → `main` ещё не сделан)

## What was verified
- Commands run + results:
  - `cargo check -p ingest` → **ok**
  - `cargo test -p ingest` → **ok** — **20** tests
    - unit (`lib`): 8 (auth ×3 + token ×5)
    - `ingest_http`: 9 (incl. `persisted_token_is_accepted_wrong_token_rejected`)
    - `pairing_token`: 3 (round-trip / `IngestConfig::load` + `BIOFOCUS_HOME` / env override)
- AC results (pass/fail per item):
  - **AC1 Pass** — `generate_pairing_token` = 32 bytes → 64 hex; `load_or_create` пишет `$HOME/.biofocus/pairing_token` или `$BIOFOCUS_HOME/pairing_token`; Unix `0600` via temp+rename
  - **AC2 Pass** — reload same token; HTTP accept persisted Bearer (`persisted_token_is_accepted_…`)
  - **AC3 Pass** — missing/wrong Bearer → **401** (T1 tests + persisted wrong-token case)
  - **AC4 Pass** — `.gitignore` has `.biofocus/` + `pairing_token`; `git check-ignore` ok; secret not tracked
  - **AC5 Pass** — `docs/10-security.md` §2 table; `docs/12-development.md` path + `BIOFOCUS_INGEST_TOKEN` / `BIOFOCUS_HOME`
  - **AC6 Pass** — generate→persist→reload + env override + reject wrong token (temp/`BIOFOCUS_HOME`)
  - **Global DoD Pass** — `unwrap`/`expect` только в `#[cfg(test)]` / integration tests; UI↛DB; loopback-only; idle accept unchanged
- Extra checks (edge / security):
  - Resolve order: non-empty `BIOFOCUS_INGEST_TOKEN` → file load-or-create (env does not rewrite disk)
  - Empty on-disk token → `EmptyTokenFile` hard error (no silent regenerate) — intentional per Dev
  - `IngestConfig::load()` / `from_env()` → `IngestResult`; skeleton const only for `Default` / `with_token` / tests
  - Docs note: host wire `IngestConfig::load` → **T4**; QR UX → **E3-T2**

## Defects (if any)
- Нет блокеров AC.
- **Note:** ветка `phase/2-ingest-http` всё ещё с untracked `crates/ingest/` + uncommitted docs — нужен **commit + PR → main** (carry с T1).
- **Note (out of scope):** mid-batch `503` contract → **T3**; host не вызывает `load()` при старте app → **T4**.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P2-E1-T2 → Done; Ready → **P2-E1-T3** (persist + mid-batch backpressure)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs (optional): `ARCHITECTURE_STATUS.md` — pairing token persistence landed
- [ ] Brief для Dev: `docs/handoffs/P2-E1-T3-pm-brief.md` (если ещё нет)
- [ ] Напомнить Dev: commit + PR `phase/2-ingest-http` → `main` (T1+T2 вместе ок)

## PM updates (2026-08-04)
- [x] SPRINT_ROADMAP — T2 Done; Ready T3; T4 AC + `IngestConfig::load`
- [x] ARCHITECTURE_STATUS, 14-roadmap
- [x] Brief `docs/handoffs/P2-E1-T3-pm-brief.md`
- [x] Execution canvas
- [ ] **Still open:** commit + PR `phase/2-ingest-http` (T1+T2 code uncommitted)

## Suggested next Ready task
- **P2-E1-T3** — Persist ingest → `ObservationRepository` (+ mid-batch 503 contract)

## Notes for PM
- Публичный API: `token::{resolve_ingest_token, load_or_create_pairing_token, generate_pairing_token, default_pairing_token_path}`, consts `BIOFOCUS_HOME_ENV`, `INGEST_TOKEN_ENV`, `PAIRING_TOKEN_FILE`.
- Path contract: `~/.biofocus/pairing_token` | `$BIOFOCUS_HOME/pairing_token` | override `BIOFOCUS_INGEST_TOKEN`.
- Параллельно по-прежнему доступен маленький **P2-E0-T1** (sanitize `dbError`).
