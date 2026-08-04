# BioFocus Desktop

Tauri v2 + React/TypeScript shell for the local-first BioFocus Core.

## Prerequisites

- Node.js >= 20, pnpm
- Rust stable (`rust-toolchain.toml` at repo root)
- macOS: Xcode CLT / full Xcode for linking

## Commands

```bash
pnpm install
pnpm tauri dev      # window + tray shell
pnpm tauri build    # production bundle
```

From repo root, Core crates still build via `cargo check` / `cargo test`.
`src-tauri` is a Cargo workspace member and depends on `runtime` + `storage`.

## Menubar status (Phase 1)

Compact window + tray tooltip show a neutral Core state: **Idle** / **Ready** /
**Error**. Status is loaded only via IPC (`get_status`; `core_ping` fallback).

QA mock (no DB): open with `?mockStatus=idle|ready|error` to force a state.

## Boundary

UI talks to Core only through Tauri IPC. The frontend must not import
`rusqlite`, open `biofocus_main.db`, or run SQL.
