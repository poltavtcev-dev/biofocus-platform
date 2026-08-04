# BioFocus Platform

Local-First open-source platform for productivity, physiological stress, and recovery analysis.

## License

[AGPLv3](./LICENSE) — see ADR-004 in `docs/decision-log.md`.

## Documentation

- **Contributing / how to extend:** [`CONTRIBUTING.md`](./CONTRIBUTING.md) (plugins, pipeline, Features, PRs)
- Vision & scope: `docs/00-vision.md`
- Project canvas: `docs/PROJECT_CANVAS.md`
- Sprint plan: `docs/SPRINT_ROADMAP.md`
- Agent team workflow: `docs/17-agent-workflow.md` / `AGENTS.md`
- Architecture freeze: `docs/ARCHITECTURE_STATUS.md`
- Local setup: `docs/12-development.md`

## Quick start (Core)

```bash
# Requires Rust stable (see rust-toolchain.toml)
cargo check
cargo test
cargo test -p storage
cargo test -p desktop
```

## Quick start (Desktop)

```bash
cd apps/desktop
pnpm install
pnpm tauri dev
# pnpm tauri build  # → BioFocus.app
```

Requires Node.js >= 20, pnpm, and platform Tauri prerequisites
(see `docs/12-development.md`).

## CI

GitHub Actions: `.github/workflows/ci.yml` (push/PR to `main` or `master`).

From Phase 2 onward: land work via **PR** (see `docs/12-development.md` § Git workflow) — do not push straight to `main`.
