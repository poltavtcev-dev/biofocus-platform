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
cargo test
cargo test -p storage
cargo test -p desktop

# Desktop (Epic E3 Done)
cd apps/desktop
pnpm install
pnpm tauri dev
# pnpm tauri build  # → BioFocus.app

# QA mock statuses (webview)
# http://localhost:…/?mockStatus=idle|ready|error
```

## CI

GitHub Actions (no CD): `.github/workflows/ci.yml`

- **rust-core** (`ubuntu-latest`): `cargo check --workspace --exclude desktop`, `cargo test -p bio-spec -p runtime -p storage`
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

**Status (2026-08-04):** Phase 1 Foundation **fully closed** (E1–E4 Done; `docs/14-roadmap.md` [x]). Phase 2 not started.  
**Follow-up:** sanitize `dbError` paths on host (see E3-T4 QA notes).
