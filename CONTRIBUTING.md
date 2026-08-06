# Contributing to BioFocus

Thanks for helping improve BioFocus — a **local-first** open-source platform for productivity, physiological stress, and recovery analysis.

By contributing you agree that your work is licensed under **[AGPLv3](./LICENSE)** (see ADR-004 in [`docs/decision-log.md`](docs/decision-log.md)). Product vision: [`docs/00-vision.md`](docs/00-vision.md).

```text
Clone & run → pick change type → edit the right crate → respect hard rules → tests/CI → PR to main
```

## First run

Full setup (Rust, Node/pnpm, Tauri, ingest, collectors): **[`docs/12-development.md`](docs/12-development.md)**.

```bash
git clone https://github.com/poltavtcev-dev/biofocus-platform.git
cd biofocus-platform

cargo check --workspace --exclude desktop
cargo test -p bio-spec -p runtime -p storage -p ingest -p pipeline

cd apps/desktop && pnpm install && pnpm tauri dev
```

CI runs on every PR to `main` (`.github/workflows/ci.yml`): Rust core on Ubuntu + desktop job on macOS.

## Where to change what

| Want to… | Start in |
| :--- | :--- |
| New Observation shape / `data_type` contract | `crates/bio-spec`, [`docs/07-contracts.md`](docs/07-contracts.md) |
| Persist / query Observations | `crates/storage` (**schema only via ADR**) |
| HTTP ingest / pairing token | `crates/ingest`, `apps/desktop/src-tauri/src/ingest_host.rs` |
| macOS collector or new device plugin | `crates/plugin-sdk`, `crates/macos-collector`, [`docs/08-plugin-sdk.md`](docs/08-plugin-sdk.md) |
| Quality pipeline (dedupe / normalize) | `crates/pipeline` |
| Feature Worker / Core host helpers | `crates/runtime` (`spawn_feature_worker`) |
| Feature formulas / Signals | `crates/feature-engine`, [`docs/06-feature-catalog.md`](docs/06-feature-catalog.md) |
| Menubar / shell UI | `apps/desktop` — **UI → IPC only**, never SQLite |
| Companion sample / CLI | `apps/companion` |
| Architecture / roadmap / glossary | `docs/` — especially [`ARCHITECTURE_STATUS.md`](docs/ARCHITECTURE_STATUS.md), [`14-roadmap.md`](docs/14-roadmap.md), [`16-glossary.md`](docs/16-glossary.md) |

Workspace layout overview: [`docs/13-project-structure.md`](docs/13-project-structure.md).

```text
Collectors / companion ──► bounded channel ──► persist (SQLite)
                                                    │
                              Feature Worker poll ◄─┘
                                    │
                         pipeline (accept→dedupe→normalize)
                                    │
                         feature-engine hook (DAG / Features)
                                    │
                         IPC ──► Menubar / shell UI
```

## Hard rules (non-negotiable)

1. **UI ↛ SQLite** — frontend talks only via Tauri IPC; Core owns storage and Features/Signals.
2. **No `unwrap()` / `expect()`** in production paths — use `thiserror` / explicit `Result`.
3. **Bounded channels** — no unbounded `mpsc`; prefer `runtime::observation_channel`.
4. **Idle footprint** — no busy-loops; sleep / event wait / rare timer when idle; stop must freeze background work.
5. **No new SQLite schema** without an ADR + maintainer approve ([`docs/ARCHITECTURE_STATUS.md`](docs/ARCHITECTURE_STATUS.md), [`docs/decision-log.md`](docs/decision-log.md)). Features/Signals stay in-memory / derived until decided otherwise.
6. **Ubiquitous Language** — use `Observation`, `Signal`, `Feature` as defined in [`docs/16-glossary.md`](docs/16-glossary.md).
7. **Crates ≠ Tauri** — code under `crates/` must compile without Tauri ([`docs/15-engineering-principles.md`](docs/15-engineering-principles.md)).
8. **Privacy** — no raw biometrics / absolute paths in IPC, HTTP status, or logs meant for UI ([`docs/10-security.md`](docs/10-security.md)).

## Recipes

### 1. Normalize a new or existing `data_type`

1. Document the payload contract in [`docs/07-contracts.md`](docs/07-contracts.md) (and `bio-spec` if the type is new).
2. Add rules + unit tests in `crates/pipeline` (`normalize` stage). Unknown types should **pass through**; unparseable known types should **skip** (not panic).
3. Run `cargo test -p pipeline`.
4. Do **not** rewrite SQLite rows — pipeline is in-memory / derived.

Entry helpers: `pipeline::accept_*` → `dedupe_*` → `normalize_*`, or `pipeline::run_quality_pipeline`.

### 2. Add a collector plugin

1. Implement `plugin_sdk::BioFocusPlugin` (`start_stream` / `stop_stream` with a bounded `ObservationSender`).
2. Emit only the agreed `data_type` + payload keys (see contracts). Prefer mockable probes for tests.
3. Wire start/stop in the desktop host the same way as `ingest_host` + `macos-collector` (start in setup, stop on `ExitRequested`).
4. Prove idle: after `stop_stream`, probe/work counters must not keep growing.
5. Run `cargo test -p macos-collector` (and relevant host tests).

### 3. Extend the Feature path

1. Quality stages are already wired: runtime Feature Worker polls new Observations → `run_quality_pipeline` → `FeatureHook`.
2. Replace / extend the noop hook with real DAG work in `crates/feature-engine` per [`docs/06-feature-catalog.md`](docs/06-feature-catalog.md).
3. Keep math deterministic (no LLM in Feature formulas). Persist Features only if an ADR says so.
4. Tests: `cargo test -p runtime` (worker idle freeze) and eventually `cargo test -p feature-engine`.

### 4. Add a desktop IPC command

1. Add a `#[tauri::command]` in `apps/desktop/src-tauri` and register it in `invoke_handler`.
2. Return JSON-safe status only — **no** Observation payloads, HRV samples, or filesystem paths (use `StorageError::public_message` patterns).
3. Call Core crates from Rust; never open SQLite from React/TS.
4. Add a small unit test on the host payload shape when practical; run `cargo test -p desktop`.

## What to work on

- Active sprint queue: [`docs/SPRINT_ROADMAP.md`](docs/SPRINT_ROADMAP.md)
- Phase goals: [`docs/14-roadmap.md`](docs/14-roadmap.md)

Open an issue or ask maintainers before:

- Jumping ahead of the current Phase (e.g. dashboard / LLM reports while Phase 3 pipeline is open)
- Schema migrations or new persisted entities
- Large refactors across crate boundaries

## Git & pull requests

**Do not push directly to `main`.**

**PR freeze (maintainers / agents):** until **2026-09-01** inclusive — do **not** open PRs or merge to `main` via PR. Work on feature branches; local commits OK. Details: [`docs/12-development.md`](docs/12-development.md), [`.cursor/rules/06-git-agent-policy.mdc`](.cursor/rules/06-git-agent-policy.mdc).

After the freeze:

```bash
git checkout main && git pull
git checkout -b feat/short-slug   # or fix/…, docs/…

# … implement + tests …
git add … && git commit -m "Short why (area + intent)"

git push -u origin HEAD
gh pr create --base main --title "…" --body "## Summary
- …
## Test plan
- [ ] cargo test -p <touched crates>
- [ ] CI green (rust-core + desktop)
"
```

- Prefer **squash merge** after CI is green.
- Related changes can share one branch / few meaningful commits (see [`docs/12-development.md`](docs/12-development.md) § Git workflow).
- External contributors do **not** need the Cursor multi-agent Dev→QA→PM handoff. That workflow (`AGENTS.md`, [`docs/17-agent-workflow.md`](docs/17-agent-workflow.md)) is optional for maintainers using Cursor.

## Security & privacy checklist

- [ ] No pairing token or `~/.biofocus` secrets committed
- [ ] No biometric raw samples in logs, IPC, or HTTP `/v1/status`
- [ ] Error strings for UI/HTTP are path-free
- [ ] Opt-in collectors stay off by default when they need sensitive permissions

Details: [`docs/10-security.md`](docs/10-security.md).

## Questions

- Architecture freeze / modification policy: [`docs/ARCHITECTURE_STATUS.md`](docs/ARCHITECTURE_STATUS.md)
- Local commands & pairing: [`docs/12-development.md`](docs/12-development.md)
- Domain terms: [`docs/16-glossary.md`](docs/16-glossary.md)

If something in the docs conflicts with frozen architecture, **architecture docs win** — open an issue before coding around them.
