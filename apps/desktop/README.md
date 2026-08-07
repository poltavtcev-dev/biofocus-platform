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
cache; Pattern Discovery baseline rules included when history supports them;
calm empty state when none / thin history). Soft refresh ~30s. Rows show a
calm category label (`Pattern` / `Focus` / `Stress`) from Core `category` —
title and description are rendered as returned (no clinical UI chrome).

QA mocks:
- `?view=dashboard&mockSnapshot=empty|ready|error`
  (`ready` includes a multi-window series for chart smoke)
- `?view=dashboard&mockInsights=empty|ready|pattern|error`
  (`ready` includes pattern + stress + focus sample Insights;
  `pattern` is a single `focus_vs_recent_baseline_v1`-shaped Insight)

Dogfood (live pattern Insight): run the desktop app with multi-day afternoon
FocusScore evidence (UTC 13:00–17:00 windows, confidence ≥ 0.4, |Δ| ≥ 10 vs
baseline mean). Thin history → empty Insights list (no error). UI never opens
SQLite — only `invoke("get_insights")`.

## Life Events quick-log (P6-E2-T1)

Menubar **Life events** section logs v1 kinds (`coffee` / `walk` / `lunch` /
`workout`) via IPC `log_life_event`. Rows persist as ordinary Observations
(`data_type: "life_event"`) through the host Observation repository — UI never
opens SQLite. Recent list via `list_recent_life_events` (manual Refresh; no
busy-loop). Calm, non-evaluative copy only.

QA mock: `?mockLifeEvents=empty|ready|error`

## Companion pairing (P2-E3-T2 / P5-E2-T1)

The shell **Companion** section loads pairing via IPC `get_pairing_token`:
copyable **Base URL** (`ingestBaseUrl` / primary `baseUrlHints`), plus token
Show / Copy / QR. Loopback by default; after LAN opt-in the primary hint is a
LAN URL when discovery succeeds. No cloud account; frontend never opens
`~/.biofocus` itself. See `docs/12-development.md` (Base URL hint).

## Boundary

UI talks to Core only through Tauri IPC. The frontend must not import
`rusqlite`, open `biofocus_main.db`, or run SQL.
