# 12. Local Development Guide

> **Extending BioFocus** (where to add plugins, pipeline stages, Features, IPC): see root [`CONTRIBUTING.md`](../CONTRIBUTING.md). This file is local setup, commands, and git cadence.

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
cargo test -p companion
cargo test -p pipeline
cargo test -p desktop

# Desktop (Epic E3 Done)
cd apps/desktop
pnpm install
pnpm tauri dev
# pnpm tauri build  # → BioFocus.app

# QA mock statuses (webview)
# http://localhost:…/?mockStatus=idle|ready|error
# http://localhost:…/?mockAlert=green|yellow|red
# combine: ?mockStatus=ready&mockAlert=yellow
```

Ingest (Phase 2): loopback **`127.0.0.1:8787`**, crate `crates/ingest`.
Desktop host starts ingest on launch (`IngestConfig::load` + persist worker) and stops it on exit.

**Pipeline (Phase 3 E1):** crate `crates/pipeline`.
- **Intake (T1):** `pipeline::accept_observations(&[Observation])` → `AcceptedBatch` (`AcceptedForProcessing`). Empty = **Ok**. Helpers: `accept_owned`, `accept_iter`.
- **Dedupe (T2):** `pipeline::dedupe_observations(&mut DedupeState, &[Observation])` → `DedupedBatch` (`Deduped`); wire `dedupe_accepted`. Rule: drop if same `id` **or** same `(provider_id, data_type, timestamp, payload JSON)` already seen (first wins; in-memory seen-set). SQLite rows never rewritten.
- **Normalize (T3):** `pipeline::normalize_observations(&[Observation])` → `NormalizedBatch` (`Normalized`); wire `normalize_deduped`. Known types → canonical payload/units (`heart_rate` / `hrv` / `context_window` / `keystrokes`); unknown → pass-through; unparseable known → skip. SQLite rows never rewritten.
- **Quality chain helper:** `pipeline::run_quality_pipeline(batch, &mut DedupeState)` → accept → dedupe → normalize.
- **Feature Worker (T4):** `runtime::spawn_feature_worker(source, hook, config)` — idle-safe poll (`recv_timeout` ≥1s when empty); desktop `feature_host::{start,stop}_feature_host` with app lifecycle. Hook stub: `NoopFeatureHook` until Feature Engine (E2). Storage cursor: `ObservationRepository::list_after_created_cursor` (no new schema; launch at DB tip).

**Feature Engine (Phase 3 E2):** crate `crates/feature-engine`.
- **DAG skeleton (T1):** `FeatureEngine::register` + `FeatureEngine::run(&[Observation])` → `EngineOutput { features, signals }`. Nodes implement `FeatureNode` (`id` / `depends_on` / `compute`). Kahn topo; errors via `thiserror` (duplicate / unknown dep / cycle / node failed). Empty DAG/snapshot → `Ok` empty.
- **Focus catalog (T2):** `feature_engine::register_focus_v1(&mut engine)` — `ContextSwitchRate` → `FocusScore` (window 15m / step 1m; see `/docs/06-feature-catalog.md`).
- **Stress/Fatigue (T3):** `feature_engine::register_stress_v1(&mut engine)` — `StressIndex` + `FatigueIndex` (needs FocusScore already registered); contiguous StressIndex > 75 for > 5m → transient `Signal` `High_Stress` (`Severity::High`). Full catalog: `feature_engine::register_catalog_v1` = focus + stress.
- **Alert level (E3-T1):** `feature_engine::map_alert_level(&EngineOutput) → AlertLevel` — Red if `High_Stress`; Yellow if latest StressIndex or FatigueIndex > 60; else Green (empty → Green).
- **IPC alert (E3-T2):** Desktop `get_status` includes `alertLevel` (`green`/`yellow`/`red`). Feature Worker hook runs catalog → map → shared `AlertState`. Idle/Ready/Error (`dbStatus`) unchanged. UI reads via IPC only.
- **Menubar alert UX (E3-T3):** Shell shows calm Steady/Elevated/High indicator (color + copy) from `alertLevel`; tray tooltip includes the label. Poll ~5s. QA: `?mockAlert=green|yellow|red`.
- **Feature snapshot IPC (P4-E1-T1):** `feature_engine::FeatureSnapshot` from last `EngineOutput`; desktop caches via `SnapshotState` + `invoke("get_feature_snapshot")` (pure read). `get_status` stays lean. Contract: `docs/09-api.md`.
- **Dashboard shell (P4-E1-T2):** separate Tauri window `label: dashboard` (`index.html?view=dashboard`); Menubar **Open Dashboard** → `invoke("open_dashboard")` (show/focus; CloseRequested → hide). Calm loading/empty/error via snapshot IPC. QA mocks: `?view=dashboard&mockSnapshot=empty|ready|error` (see `apps/desktop/README.md`).
- **Recharts Feature series (P4-E1-T3):** Dashboard `ChartSlot` — LineChart for `FocusScore` / `StressIndex` / `FatigueIndex` (+ `ContextSwitchRate` secondary axis when present); calm labels; refresh on open + ~30s. Dep: `recharts` in `apps/desktop`.
- **knowledge-engine skeleton (P4-E2-T1):** pluggable `InsightRule` + `KnowledgeEngine::evaluate(&[Feature], &[Signal]) → Result<Vec<Insight>>`; types `Insight` / `EvidenceRef` from `bio-spec`; empty/no-match → `Ok([])`.
- **Rule Insights v1 (P4-E2-T2):** `knowledge_engine::register_insights_v1` — `High_Stress` Signal + elevated `ContextSwitchRate` (≥1.0) rules with calm copy / `EvidenceRef`; host must register (empty engine still `Ok([])`). IPC/UI → T3.
```bash
cargo test -p feature-engine
cargo test -p knowledge-engine
cargo test -p desktop
cd apps/desktop && pnpm exec tsc --noEmit
```

### Pipeline E2E (P3-E2-T4)

Integration suite (`docs/11-testing.md` §5): fixture / mock Observation stream → `pipeline::run_quality_pipeline` (accept → dedupe → normalize) → `feature_engine::register_catalog_v1` + `FeatureEngine::run` → asserted Features / Signals (in-memory; no SQLite schema, no UI).

```bash
# Full pipeline crate (unit + E2E)
cargo test -p pipeline

# E2E only
cargo test -p pipeline --test pipeline_e2e

# Feature Engine units (catalog) — optional companion
cargo test -p feature-engine
```

Coverage (`crates/pipeline/tests/pipeline_e2e.rs`):
- **Happy path** — fixture `tests/fixtures/e2e_happy_path.json` (alias payloads, duplicate id, unknown `custom.debug` pass-through) → normalized Observations → `ContextSwitchRate` / `FocusScore` / `StressIndex` / `FatigueIndex` + `High_Stress` Signal.
- **Edge** — empty batch → idle-friendly empty output; high RMSSD → StressIndex below threshold, no `High_Stress`.

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

### Companion sample path (P2-E3-T1)

```bash
cargo test -p companion

# Live Desktop ingest (token from file or env):
export BIOFOCUS_INGEST_TOKEN="$(cat ~/.biofocus/pairing_token)"
cargo run -p companion --bin biofocus-companion-sample -- 74
```

Rust client + CLI in `apps/companion`; iOS HealthKit stub under `apps/companion/ios/`. Same-machine / Simulator: `http://127.0.0.1:8787`. Physical phone on LAN needs a reachable host (loopback bind today — LAN later).

### Pairing UX (P2-E3-T2)

In the Desktop shell (**Companion** section):

1. **Show** reveals the local pairing token (masked by default).
2. **Copy** puts the token on the clipboard for paste into companion / CLI / iOS stub.
3. **Show QR** displays a QR of the token (scan with the phone camera → copy text).

IPC: `invoke("get_pairing_token")` — see `docs/09-api.md`. No cloud account; UI does not open `~/.biofocus` itself.

**HTTP status (companion/debug):** `curl -s http://127.0.0.1:8787/v1/status` → `version` + `db_status` (no Observation payload). Menubar still uses IPC `get_status`.

**Pairing token**
- Default file: `~/.biofocus/pairing_token` (created on first `IngestConfig::load`)
- Override: `export BIOFOCUS_INGEST_TOKEN=…` (skips file; shell shows “env override”)
- Tests / custom root: `export BIOFOCUS_HOME=/tmp/biofocus-test` → `$BIOFOCUS_HOME/pairing_token`
- Share path: Desktop Companion UI (copy / QR) — do not commit the token
- Details: `docs/10-security.md`

## CI

GitHub Actions (no CD): `.github/workflows/ci.yml`

- **rust-core** (`ubuntu-latest`): `cargo check --workspace --exclude desktop`, `cargo test -p bio-spec -p runtime -p storage -p ingest -p pipeline -p plugin-sdk -p macos-collector -p companion`
- **desktop** (`macos-latest`): `cargo test -p desktop`, `pnpm install` + `pnpm build`, UI↛DB boundary grep

Triggers: push/PR to `main` or `master`.

## Git workflow (related work → PR)

**Не пушить напрямую в `main`.** Агенты: `.cursor/rules/06-git-agent-policy.mdc`.

Классика: **связанный код** → одна ветка → **мало коммитов** → **один PR** на код-кластер.  
Не коммит/PR на каждый handoff. **Docs / roadmap / canvas** — отдельно позже или в следующий code PR.

| Уровень | Правило |
| :--- | :--- |
| **Ветка** | Кластер связанного кода: `phase/N-…`, `epic/P?-E?-…`, `feat/…`. |
| **Commit** | Когда код-единица готова (batch Task IDs ок). Handoffs на диске — не триггерят PR. |
| **Push / PR** | Только substantive code vs `main` **и** (кластер готов **или** явный «PR»). Один PR на ветку. Squash preferred. |
| **Не PR** | handoffs-only, roadmap/canvas-only, второй PR на тот же tip. |
| **Спринт** | Лучше мало содержательных PR, чем много пустых. |

```bash
git checkout main && git pull
git checkout -b epic/p3-e1-pipeline   # or feat/…

# … Dev → QA → PM on related tasks; handoffs on disk …
git add … && git commit -m "P3-E1: …"

git push -u origin HEAD
gh pr create --base main --title "…" --body "## Summary
- …
## Test plan
- [ ] CI green (rust-core + desktop)
"
# prefer: gh pr merge --squash (after CI Pass)
```

Naming: `phase/N-short-slug`, `epic/…`, `feat/…`.  
Phase 1 landed as direct push to `main` (foundation exception); do not repeat.

## Status

**Status (2026-08-05):** Phase 1–3 **Done** (Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24)). Phase 4 **open** (Sprint 7–8): Dashboard + Insights + optional local LLM. Done: Epic **P4-E1** · **P4-E2-T1**. Ready: **P4-E2-T2** (Rule Insights v1) — `docs/handoffs/P4-E2-T2-pm-brief.md`. Branch: `phase/4-dashboard-ai`.  
**Git policy:** few **code** PRs; commit messages describe the change only — no personal device inventories.
