# 12. Local Development Guide

## Prerequisites
- Rust stable (edition 2024) — `rustup` / `rust-toolchain.toml`
- Node.js >= 20.x, pnpm
- System deps for Tauri v2 on macOS (Xcode CLT)

## Commands
```bash
git clone https://github.com/poltavtcev-dev/biofocus-platform.git
cd biofocus-platform

# Core
cargo check
cargo check --workspace --exclude desktop
cargo test
cargo test -p storage
cargo test -p ingest
cargo test -p macos-collector
cargo test -p desktop

# Desktop (Epic E3 Done)
cd apps/desktop
pnpm install
pnpm tauri dev
# pnpm tauri build  # → BioFocus.app

# QA mock statuses (webview)
# http://localhost:…/?mockStatus=idle|ready|error
```

Ingest (Phase 2): loopback **`127.0.0.1:8787`**, crate `crates/ingest`.
Desktop host starts ingest on launch (`IngestConfig::load` + persist worker) and stops it on exit.

**Active window collector (P2-E2-T1):** on macOS, Desktop also starts `ActiveWindowPlugin` (poll ≥1s → `context_window` Observations on the same channel → persist worker). No Accessibility permission. Payload: `bundle_id` + `app_name` only.

**Input aggregates (P2-E2-T2, opt-in):** `export BIOFOCUS_INPUT_AGGREGATES=1` then restart Desktop. Requires Accessibility. Emits `keystrokes` Observations (`count` / `window_secs` / `rate_per_min`) on the same channel. Default off.

### Collector test suite (P2-E2-T3)

```bash
# Unit + integration (mock probes; no Accessibility / NSWorkspace required)
cargo test -p macos-collector

# Integration only: emit Observation → channel → persist → SQLite + stop/idle
cargo test -p macos-collector --test collector_integration

# Broader Phase 2 ingest path (HTTP + persist), optional:
cargo test -p ingest
```

Integration coverage (`tests/collector_integration.rs`): `context_window` and `keystrokes` land in storage via `spawn_persist_worker`; after `stop_stream`, probe call counts freeze (no busy-loop). Pause for collectors = `stop_stream` (plugin trait has no separate pause API).

**HTTP status (companion/debug):** `curl -s http://127.0.0.1:8787/v1/status` → `version` + `db_status` (no Observation payload). Menubar still uses IPC `get_status`.

**Pairing token**
- Default file: `~/.biofocus/pairing_token` (created on first `IngestConfig::load`)
- Override: `export BIOFOCUS_INGEST_TOKEN=…` (skips file)
- Tests / custom root: `export BIOFOCUS_HOME=/tmp/biofocus-test` → `$BIOFOCUS_HOME/pairing_token`
- Details: `docs/10-security.md`

## CI

GitHub Actions (no CD): `.github/workflows/ci.yml`

- **rust-core** (`ubuntu-latest`): `cargo check --workspace --exclude desktop`, `cargo test -p bio-spec -p runtime -p storage -p ingest`
- **desktop** (`macos-latest`): `cargo test -p desktop`, `pnpm install` + `pnpm build`, UI↛DB boundary grep

Triggers: push/PR to `main` or `master`.

## Git workflow (commit after build · push at sprint gate)

С Phase 2: **не пушить напрямую в `main`**. Агенты: `.cursor/rules/06-git-agent-policy.mdc`.

| Уровень | Правило |
| :--- | :--- |
| **Commit (build)** | Сразу после Dev/UX билда: код + `docs/handoffs/{TASK}-dev-to-qa.md`, subject с Task ID. |
| **Commit (PM Done)** | Docs/kanban/qa-to-pm, если tree dirty после закрытия задачи. |
| **Push / PR** | **Раз на спринт** (или epic gate): push ветки → `gh pr create` → merge после зелёного CI. Может выполнить Dev, QA или PM — кто закрывает спринт. |
| **Ветка** | Вся работа спринта в одной ветке `phase/N-…` или `sprint/N-…`. |

```bash
# start sprint / phase work
git checkout main && git pull
git checkout -b phase/2-ingestion   # or sprint/3-…

# after each Dev/UX build (handoff written, tests green):
git add … && git commit -m "P2-E?-T?: …"

# after PM Done (if docs still dirty):
git add docs/ … && git commit -m "P2-E?-T?: close task docs/kanban"

# end of sprint / epic gate — push once and open PR
git push -u origin HEAD
gh pr create --base main --title "Phase 2 Sprint …" --body "## Summary
- …
## Test plan
- [ ] CI green (rust-core + desktop)
"
# merge after CI Pass (GitHub UI or: gh pr merge --squash)
```

Naming: `phase/N-short-slug` or `sprint/N-short-slug`.  
Phase 1 landed as direct push to `main` (foundation exception); do not repeat.

## Status

**Status (2026-08-04):** Phase 2 — **P2-E2-T1/T2 Done**; **P2-E2-T3** in progress (collector integration + idle) on `phase/2-ingest-http`; Epic E1 Done.  
**Git:** commit-after-build on branch; **sprint PR** at Sprint 3–4 gate (`docs/handoffs/SPRINT-GATE.md`). Follow-up: sanitize `dbError` (**P2-E0-T1**).
