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

## Git workflow (PR per sprint)

С Phase 2: **не пушить напрямую в `main`**. Работа спринта — в ветке, в `main` только через PR после зелёного CI.

```bash
# start sprint / phase work
git checkout main && git pull
git checkout -b phase/2-ingestion   # or sprint/3-…

# … implement + handoffs …

git push -u origin HEAD
gh pr create --base main --title "Phase 2: …" --body "## Summary
- …
## Test plan
- [ ] CI green (rust-core + desktop)
"
# merge after CI Pass (GitHub UI or: gh pr merge --squash)
```

Naming: `phase/N-short-slug` or `sprint/N-short-slug`.  
Phase 1 landed as direct push to `main` (foundation exception); do not repeat.

## Status

**Status (2026-08-04):** Phase 2 — **P2-E1-T1…T3** Done; Ready **P2-E1-T4**.  
**Follow-up:** commit T3 on branch; sprint PR later; sanitize `dbError` (**P2-E0-T1**).
