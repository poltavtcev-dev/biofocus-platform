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

## Git workflow (commit per task · PR per sprint)

С Phase 2: **не пушить напрямую в `main`**.

| Уровень | Правило |
| :--- | :--- |
| **Commit** | **Один commit на задачу** (Task ID) после QA Pass + PM Done: код + handoffs + docs/kanban по этой задаче. |
| **Push / PR** | **Один PR на спринт** (или epic gate): push ветки → `gh pr create` → merge после зелёного CI. Не открывать PR на каждую задачу. |
| **Ветка** | Вся работа спринта в одной ветке `phase/N-…` или `sprint/N-…`. |

```bash
# start sprint / phase work
git checkout main && git pull
git checkout -b phase/2-ingestion   # or sprint/3-…

# after each task closes (PM Done):
git add … && git commit -m "P2-E?-T?: …"

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

**Status (2026-08-04):** Phase 2 — Epic **P2-E1** Done on branch `phase/2-ingest-http`; Ready **P2-E2-T1**.  
**Git:** commit-per-task on branch; **sprint PR** when Sprint 3–4 / agreed gate is ready. Follow-up: sanitize `dbError` (**P2-E0-T1**).
