# 12. Local Development Guide

> **Extending BioFocus** (where to add plugins, pipeline stages, Features, IPC): see root [`CONTRIBUTING.md`](../CONTRIBUTING.md). This file is local setup, commands, and git cadence.

## Workspace location (macOS / iCloud)

Prefer a **local** path outside iCloud Desktop/Documents (e.g. `~/Developer/AI Project/BioFocus`). iCloud “Optimize Mac Storage” can leave `dataless` stubs so `Cargo.toml` looks empty and `cargo` / `git` fail. Do not keep the active git worktree only on Desktop if Desktop syncs to iCloud.

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

Ingest (Phase 2+): default loopback **`127.0.0.1:8787`**, crate `crates/ingest`.
Desktop host starts ingest on launch (`IngestConfig::load` + persist worker) and stops it on exit.

**LAN opt-in (P5-E1-T1 / ADR-005):** for a physical phone on the same Wi-Fi, restart Desktop with:
```bash
export BIOFOCUS_INGEST_LAN=1
# optional override: export BIOFOCUS_INGEST_BIND_HOST=0.0.0.0   # or a specific LAN IPv4
```
Without these knobs, bind stays `127.0.0.1` (Simulator / same-machine unchanged). `POST /v1/ingest` still requires Bearer (`BIOFOCUS_INGEST_TOKEN` / `~/.biofocus/pairing_token`).

**Base URL hint (P5-E1-T2):** after LAN opt-in, read bind mode + usable URLs from:
- `GET /v1/status` → `bind_mode` / `base_url_hints` (snake_case), or
- IPC `get_pairing_token` → `bindMode` / `baseUrlHints` / `ingestBaseUrl` (primary = first hint).

```bash
curl -s http://127.0.0.1:8787/v1/status | jq '{bind_mode, base_url_hints}'
# point companion at base_url_hints[0], e.g. http://192.168.x.x:8787
```
Loopback mode always reports `http://127.0.0.1:<port>`. Hints are derived on read/startup (no busy-loop).

**Companion LAN UI (P5-E2-T1 Done):** Desktop **Companion** section shows copyable primary base URL (`ingestBaseUrl` from `get_pairing_token`), bind-mode / LAN opt-in copy, and existing token Show / Copy / QR. When `bindMode=lan` but primary is still loopback, UI surfaces a calm fallback hint (`BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>`). No pairing busy-loop (load on mount + manual Reload).

**Pipeline (Phase 3 E1):** crate `crates/pipeline`.
- **Intake (T1):** `pipeline::accept_observations(&[Observation])` → `AcceptedBatch` (`AcceptedForProcessing`). Empty = **Ok**. Helpers: `accept_owned`, `accept_iter`.
- **Dedupe (T2):** `pipeline::dedupe_observations(&mut DedupeState, &[Observation])` → `DedupedBatch` (`Deduped`); wire `dedupe_accepted`. Rule: drop if same `id` **or** same `(provider_id, data_type, timestamp, payload JSON)` already seen (first wins; in-memory seen-set). SQLite rows never rewritten.
- **Normalize (T3):** `pipeline::normalize_observations(&[Observation])` → `NormalizedBatch` (`Normalized`); wire `normalize_deduped`. Known types → canonical payload/units (`heart_rate` / `hrv` / `context_window` / `keystrokes`); unknown → pass-through; unparseable known → skip. SQLite rows never rewritten.
- **Quality chain helper:** `pipeline::run_quality_pipeline(batch, &mut DedupeState)` → accept → dedupe → normalize.
- **Feature Worker (T4):** `runtime::spawn_feature_worker(source, hook, config)` — idle-safe poll (`recv_timeout` ≥1s when empty); desktop `feature_host::{start,stop}_feature_host` with app lifecycle. Hook stub: `NoopFeatureHook` until Feature Engine (E2). Storage cursor: `ObservationRepository::list_after_created_cursor` (no new schema; launch at DB tip).

**Feature Engine (Phase 3 E2):** crate `crates/feature-engine`.
- **DAG skeleton (T1):** `FeatureEngine::register` + `FeatureEngine::run(&[Observation])` → `EngineOutput { features, signals }`. Nodes implement `FeatureNode` (`id` / `depends_on` / `compute`). Kahn topo; errors via `thiserror` (duplicate / unknown dep / cycle / node failed). Empty DAG/snapshot → `Ok` empty.
- **Focus catalog (T2):** `feature_engine::register_focus_v1(&mut engine)` — `ContextSwitchRate` → `FocusScore` (window 15m / step 1m; see `/docs/06-feature-catalog.md`).
- **Stress/Fatigue (T3):** `feature_engine::register_stress_v1(&mut engine)` — `StressIndex` + `FatigueIndex` (needs FocusScore already registered); contiguous StressIndex > 75 for > 5m → transient `Signal` `High_Stress` (`Severity::High`).
- **Calendar Features (P6-E3-T2):** `feature_engine::register_calendar_v1(&mut engine)` — `MeetingDensity` (busy overlap fraction 0–1) + `RecoveryBetweenMeetings` (mean free gap minutes) from `calendar_event` Observations. Full catalog: `feature_engine::register_catalog_v1` = focus + stress + calendar.
- **Alert level (E3-T1):** `feature_engine::map_alert_level(&EngineOutput) → AlertLevel` — Red if `High_Stress`; Yellow if latest StressIndex or FatigueIndex > 60; else Green (empty → Green).
- **IPC alert (E3-T2):** Desktop `get_status` includes `alertLevel` (`green`/`yellow`/`red`). Feature Worker hook runs catalog → map → shared `AlertState`. Idle/Ready/Error (`dbStatus`) unchanged. UI reads via IPC only.
- **Menubar alert UX (E3-T3):** Shell shows calm Steady/Elevated/High indicator (color + copy) from `alertLevel`; tray tooltip includes the label. Poll ~5s. QA: `?mockAlert=green|yellow|red`.
- **Feature snapshot IPC (P4-E1-T1):** `feature_engine::FeatureSnapshot` from last `EngineOutput`; desktop caches via `SnapshotState` + `invoke("get_feature_snapshot")` (pure read). `get_status` stays lean. Contract: `docs/09-api.md`.
- **Feature confidence (P7-E1-T1 / ADR-007):** domain `Feature.confidence` ∈ `[0.0, 1.0]` = coverage × mean evidence `Observation.confidence`; catalog nodes compute it; snapshot IPC exposes `confidence`. No Feature SQLite schema.
- **Explanation factors (P7-E2-T1):** optional `Feature.factors` `{ id, label, share }` — calm input-share breakdown; `FocusScore` emits renormalized shares (sum 1.0); IPC omits key when empty. No ADR (additive optional field); no SQLite schema.
- **Dashboard shell (P4-E1-T2):** separate Tauri window `label: dashboard` (`index.html?view=dashboard`); Menubar **Open Dashboard** → `invoke("open_dashboard")` (show/focus; CloseRequested → hide). Calm loading/empty/error via snapshot IPC. QA mocks: `?view=dashboard&mockSnapshot=empty|ready|error` (see `apps/desktop/README.md`).
- **Recharts Feature series (P4-E1-T3):** Dashboard `ChartSlot` — LineChart for `FocusScore` / `StressIndex` / `FatigueIndex` (+ `ContextSwitchRate` secondary axis when present); calm labels; refresh on open + ~30s. Dep: `recharts` in `apps/desktop`.
- **knowledge-engine skeleton (P4-E2-T1):** pluggable `InsightRule` + `KnowledgeEngine::evaluate(&[Feature], &[Signal]) → Result<Vec<Insight>>`; types `Insight` / `EvidenceRef` from `bio-spec`; empty/no-match → `Ok([])`.
- **Rule Insights v1 (P4-E2-T2):** `knowledge_engine::register_insights_v1` — `High_Stress` Signal + elevated `ContextSwitchRate` (≥1.0) rules with calm copy / `EvidenceRef`; host must register (empty engine still `Ok([])`). IPC/UI → T3.
- **Insights IPC + Dashboard list (P4-E2-T3):** Desktop `invoke("get_insights")` — evaluate-on-read over cached `FeatureSnapshot`; host `KnowledgeEngine::new()` + `register_insights_v1` at startup. Dashboard Insights list (evidence refs) + calm empty state; refresh on open + ~30s. Contract: `docs/09-api.md`.
- **report-engine builder (P4-E3-T1):** `report_engine::build_report(&[Feature], &[Insight]) → Result<ReportDocument>` — deterministic offline `markdown` + `llm_prompt` (no HTTP). Empty inputs → calm minimal report. LLM interpret-only; Feature math stays in `feature-engine`. Format: `docs/09-api.md` § report-engine.
- **Optional local LLM (P4-E3-T2):** **Off by default.** Set `BIOFOCUS_LOCAL_LLM=1` to opt in. Then `report_engine::interpret_report(&doc, &LocalLlmConfig::from_env()).await` POSTs only `ReportDocument::llm_prompt` to an OpenAI-compatible base URL (default `http://127.0.0.1:11434/v1` — local Ollama). Optional: `BIOFOCUS_LOCAL_LLM_BASE_URL`, `BIOFOCUS_LOCAL_LLM_MODEL` (default `llama3.2`), `BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS` (default `30`). HTTP timeout applies. **Never auto-called on app startup** — hosts must invoke only on explicit user action. Privacy: when disabled, no network. When enabled, the prompt text (offline report facts + interpret-only instructions) leaves the BioFocus process toward the configured base URL only — prefer localhost; a remote URL is the operator’s choice/responsibility. No Feature math in this path; no vendor cloud telemetry by default.
- **Report UX (P4-E3-T3):** Dashboard `ReportSlot` — explicit «Generate report» → `invoke("generate_report")`. Host assembles offline `build_report` from cached Feature snapshot + evaluate-on-read Insights; optional `interpret_report` only when `LocalLlmConfig::from_env().enabled`. Soft `llmStatus` / `llmError` on timeout or failure. Never called on Dashboard open or 30s Feature/Insights poll. Contract: `docs/09-api.md` § `generate_report`. QA mocks: `?mockReport=…`.
- **Life Events quick-log (P6-E2-T1):** Menubar Life events — `invoke("log_life_event")` / `invoke("list_recent_life_events")`. Host validates via `bio_spec` and inserts into the existing Observation store (`data_type: "life_event"`). No UI→SQLite; no life-event poll. Contract: `docs/09-api.md`. Browser QA: `?mockLifeEvents=…` (see `apps/desktop/README.md`).
```bash
cargo test -p feature-engine
cargo test -p knowledge-engine
cargo test -p report-engine
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

**Local Calendar (P6-E3-T1, opt-in):** export a Calendar (or fixture) to `.ics`, then:
```bash
export BIOFOCUS_CALENDAR=1
export BIOFOCUS_CALENDAR_ICS="$HOME/Desktop/biofocus-dogfood.ics"
# restart Desktop
```
Emits `calendar_event` Observations (`uid` / `start` / `end` / `all_day` / `busy`) via rare poll ≥60s on the same channel → persist. No Google/Outlook OAuth. Titles/bodies are not stored or logged. Default off.

### Collector test suite (P2-E2-T3)

```bash
# Unit + integration (mock probes; no Accessibility / NSWorkspace required)
cargo test -p macos-collector

# Integration only: emit Observation → channel → persist → SQLite + stop/idle
cargo test -p macos-collector --test collector_integration

# Broader Phase 2 ingest path (HTTP + persist), optional:
cargo test -p ingest
```

Integration coverage (`tests/collector_integration.rs`): `context_window`, `keystrokes`, and synthetic / ICS `calendar_event` land in storage via `spawn_persist_worker`; after `stop_stream`, probe call counts freeze (no busy-loop). Pause for collectors = `stop_stream` (plugin trait has no separate pause API).

### Companion sample path (P2-E3-T1)

```bash
cargo test -p companion

# Live Desktop ingest (token from file or env):
export BIOFOCUS_INGEST_TOKEN="$(cat ~/.biofocus/pairing_token)"
cargo run -p companion --bin biofocus-companion-sample -- 74
```

Rust client + CLI in `apps/companion`; iOS HealthKit stub under `apps/companion/ios/` (runnable target → **P5-E3-T1**). Same-machine / Simulator: `http://127.0.0.1:8787`. Physical phone on LAN: enable `BIOFOCUS_INGEST_LAN=1` on Desktop, then copy the **Base URL** from Companion UI (or `base_url_hints[0]` from `/v1/status` / pairing IPC).

### Pairing UX (P2-E3-T2 + P5-E2-T1)

In the Desktop shell (**Companion** section):

1. **Base URL** shows the primary ingest URL (LAN hint when available, else loopback) with **Copy URL** / **Reload**.
2. **Show** reveals the local pairing token (masked by default).
3. **Copy** puts the token on the clipboard for paste into companion / CLI / iOS stub.
4. **Show QR** displays a QR of the token (scan with the phone camera → copy text).

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

| Job | Runner | When |
| :--- | :--- | :--- |
| **Detect paths** | ubuntu | Always (cheap); decides what to run |
| **rust-core** | ubuntu | `crates/**`, `apps/companion/**`, `Cargo.toml` / `Cargo.lock`, `rust-toolchain.toml`, this workflow |
| **desktop** | macos (×10 min) | PR: `apps/desktop/**` or workspace meta / this workflow. **Push to `main`:** also when rust paths change (merge gate for crate→desktop breaks) |
| **CI gate** | ubuntu | Always — success if heavy jobs succeeded or were skipped (docs/handoffs-only → green without burning minutes) |

- **rust-core:** `cargo check --workspace --exclude desktop`, `cargo test -p bio-spec -p runtime -p storage -p ingest -p pipeline -p feature-engine -p plugin-sdk -p macos-collector -p companion`
- **desktop:** `cargo test -p desktop`, `pnpm install` + `pnpm build`, UI↛DB boundary grep
- **Manual full suite:** Actions → CI → **Run workflow** (`workflow_dispatch`)
- **Local before PR** (when skipping macOS on a crates-only PR): `cargo test -p desktop` + `pnpm build` on a Mac if you touched IPC / Tauri surface

Triggers: push/PR to `main` or `master`, plus `workflow_dispatch`. Docs / handoffs / `.cursor` alone do **not** start rust or desktop jobs.

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

**Status (2026-08-05):** Phase 1–4 **Done** (Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24); Phase 4 E1–E3 on `phase/4-dashboard-ai`, cluster PR pending). **Phase 5 active** — Wearable dogfood; **P5-E1 + P5-E2 Done** (LAN bind ADR-005 + advertise hints + Companion LAN UI); Ready **P5-E3-T1** (runnable iOS HealthKit companion). Branch: `phase/5-wearable-dogfood`. Brief: `docs/handoffs/P5-E3-T1-pm-brief.md`. Platform vision (L1–L5, Personal Pattern Discovery, horizon P6–P12+) accepted in `/docs/00-vision.md`.  
**Git policy:** few **code** PRs; commit messages describe the change only — no personal device inventories.
