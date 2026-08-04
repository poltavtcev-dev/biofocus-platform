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
```

Ingest (Phase 2): loopback **`127.0.0.1:8787`**, crate `crates/ingest`.
Desktop host starts ingest on launch (`IngestConfig::load` + persist worker) and stops it on exit.

**Pipeline (Phase 3 E1):** crate `crates/pipeline`.
- **Intake (T1):** `pipeline::accept_observations(&[Observation])` → `AcceptedBatch` (`AcceptedForProcessing`). Empty = **Ok**. Helpers: `accept_owned`, `accept_iter`.
- **Dedupe (T2):** `pipeline::dedupe_observations(&mut DedupeState, &[Observation])` → `DedupedBatch` (`Deduped`); wire `dedupe_accepted`. Rule: drop if same `id` **or** same `(provider_id, data_type, timestamp, payload JSON)` already seen (first wins; in-memory seen-set). SQLite rows never rewritten.
- **Normalize (T3):** `pipeline::normalize_observations(&[Observation])` → `NormalizedBatch` (`Normalized`); wire `normalize_deduped`. Known types → canonical payload/units (`heart_rate` / `hrv` / `context_window` / `keystrokes`); unknown → pass-through; unparseable known → skip. SQLite rows never rewritten.
- Runtime worker — T4.

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

Классика: **связанные задачи** → одна ветка → **немного осмысленных коммитов** → **PR**, когда кластер готов. Не коммит на каждый handoff и не один mega-PR на весь спринт.

| Уровень | Правило |
| :--- | :--- |
| **Ветка** | Кластер связанной работы: `phase/N-…`, `epic/P?-E?-…`, `feat/…`. |
| **Commit** | Когда единица работы готова к шарингу (можно batch Task IDs). Handoff-файлы обязательны для ролей — git-каденция отдельно. |
| **Push / PR** | Когда связанные задачи в кластере **Done** (или пользователь сказал «PR»). Предпочтительно **squash merge**. |
| **Спринт** | Не обязан совпадать с одним PR; несколько PR за спринт — норма. |

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

**Status (2026-08-04):** Phase 2 **merged** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)). Phase 3: **P3-E1-T1/T2/T3 Done** ([PR #5](https://github.com/poltavtcev-dev/biofocus-platform/pull/5)/[#6](https://github.com/poltavtcev-dev/biofocus-platform/pull/6), [PR #8](https://github.com/poltavtcev-dev/biofocus-platform/pull/8)/[#9](https://github.com/poltavtcev-dev/biofocus-platform/pull/9), [PR #11](https://github.com/poltavtcev-dev/biofocus-platform/pull/11)); Ready **P3-E1-T4**.  
**Git policy:** classic related-work PRs (see above) — supersedes commit-per-task / sprint-only push.
