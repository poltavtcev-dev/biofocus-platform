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

## Dashboard shell (P4-E1-T2 / T3)

Separate Tauri window (`label: dashboard`, `?view=dashboard`). Open from Menubar
via **Open Dashboard** (`invoke("open_dashboard")`). Snapshot via
`get_feature_snapshot` only — loading / empty / error / ready. Chart slot shows
Recharts series for `FocusScore`, `StressIndex`, `FatigueIndex`, and
`ContextSwitchRate` when present (calm labels; scores 0–100; CSR on secondary
axis). Insights list via `get_insights` (evaluate-on-read over the same Feature
cache; calm empty state when none). Soft refresh ~30s.

QA mocks:
- `?view=dashboard&mockSnapshot=empty|ready|error`
  (`ready` includes a multi-window series for chart smoke)
- `?view=dashboard&mockInsights=empty|ready|error`
  (`ready` includes two sample Insights with evidence refs)

## Companion pairing (P2-E3-T2)

The shell **Companion** section loads the local pairing token via IPC
`get_pairing_token` (Show / Copy / QR). Paste into the companion CLI or iOS stub.
No cloud account; frontend never opens `~/.biofocus` itself.

## Boundary

UI talks to Core only through Tauri IPC. The frontend must not import
`rusqlite`, open `biofocus_main.db`, or run SQL.
