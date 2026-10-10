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
Recharts series for `FocusScore`, `StressIndex`, `FatigueIndex`,
`ContextSwitchRate`, wearable scores, **`CognitiveLoad`**, **`DeepWorkScore`**,
**`AttentionStability`**, **`DeskAwayPresence`**, **`CircadianOffset`**, and
**`SustainedLoadIndicator`** when present (calm labels **Combined demand** /
**Sustained focus** / **Focus stability** / **Away from desk** /
**Schedule alignment** / **Prolonged load**; scores 0–100; CSR on secondary axis). Insights list via `get_insights` (evaluate-on-read over the same Feature
cache; Pattern Discovery baseline rules included when history supports them;
calm empty state when none / thin history). Suggestions (Recommendations)
via `get_recommendations` — evaluate-on-read after Insights on the same
snapshot (ADR-009); calm empty state when none. Soft refresh ~30s. Insight
rows show a calm category label (`Закономерность` / `Фокус` / `Напряжение` /
`Общая нагрузка` for `demand` / `Длительная нагрузка` for `prolonged_load`);
suggestion rows use Core title/suggestion as returned, with `pace` labeled
`Темп` (optional personal hints — not medical advice; no clinical or burnout
UI chrome). Empty Insights / Suggestions stay quiet.

Report slot (P4-E3-T3 / P11-E3-T1): calm **Local AI** provider status via
`get_local_llm_status` (`disabled` / `ready` / `error` from host env — no HTTP
probe, no secrets). Explicit **Generate report** → `generate_report` builds
offline markdown through pack `biofocus.default` @ `1` (Features + Insights +
Recommendations) and optionally interprets when `BIOFOCUS_LOCAL_LLM=1`. Soft
`llmStatus` on timeout/error so markdown is kept. Never auto-invoked on open
or soft poll.

QA mocks:
- `?view=dashboard&mockSnapshot=empty|ready|error`
  (`ready` includes a multi-window series for chart smoke)
- `?view=dashboard&mockInsights=empty|ready|pattern|demand|prolonged|error`
  (`ready` includes pattern + stress + focus sample Insights;
  `pattern` is a single `focus_vs_recent_baseline_v1`-shaped Insight;
  `demand` / `prolonged` mirror `cognitive_load_elevated_v1` /
  `sustained_load_elevated_v1` calm copy)
- `?view=dashboard&mockRecommendations=empty|ready|pace|demand|error`
  (`ready` / `pace` = sample `focus_dip_pace_hint_v1`-shaped Recommendation;
  `demand` = sample `combined_demand_pace_hint_v1`)
- `?view=dashboard&mockReport=idle|ready|disabled|ok|error`
- `?view=dashboard&mockLlmStatus=disabled|ready|error`

Dogfood (live pattern Insight + pace suggestion): run the desktop app with
multi-day afternoon FocusScore evidence (UTC 13:00–17:00 windows, confidence
≥ 0.4) where **current Focus is lower** than baseline by |Δ| ≥ 10. Thin
history → empty Insights / Suggestions (no error). UI never opens SQLite —
only `invoke("get_insights")` / `invoke("get_recommendations")` /
`invoke("get_local_llm_status")` / `invoke("generate_report")`.

## Life Events quick-log (P6-E2-T1)

Menubar **Life events** section logs v1 kinds (`coffee` / `walk` / `lunch` /
`workout`) via IPC `log_life_event`. Rows persist as ordinary Observations
(`data_type: "life_event"`) through the host Observation repository — UI never
opens SQLite. Recent list via `list_recent_life_events` (manual Refresh; no
busy-loop). Calm, non-evaluative copy only.

QA mock: `?mockLifeEvents=empty|ready|error`

## Git watched folders (P14-E3-T1)

Menubar **Git folders** lists/adds/removes personal allowlist roots and saves via
IPC `get_git_watched_roots` / `set_git_watched_roots` into
`~/.biofocus/git-watched-roots.toml` (or `$BIOFOCUS_HOME/…` in tests). UI never
opens the file or SQLite. Empty list → live Git probe stays idle. Also requires
`BIOFOCUS_GIT_ACTIVITY=1` for the collector. Calm personal copy only — not
workplace monitoring. Dogfood: `docs/12-development.md` § Git activity dogfood.

QA mock: `?mockGitRoots=empty|ready|error`

## Companion pairing (P2-E3-T2 / P5-E2-T1)

The shell **Companion** section loads pairing via IPC `get_pairing_token`:
copyable **Base URL** (`ingestBaseUrl` / primary `baseUrlHints`), plus token
Show / Copy / QR. Loopback by default; after LAN opt-in the primary hint is a
LAN URL when discovery succeeds. No cloud account; frontend never opens
`~/.biofocus` itself. See `docs/12-development.md` (Base URL hint).

## Boundary

UI talks to Core only through Tauri IPC. The frontend must not import
`rusqlite`, open `biofocus_main.db`, or run SQL.
