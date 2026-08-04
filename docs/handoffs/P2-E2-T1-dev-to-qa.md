# Dev → QA: P2-E2-T1

## Meta
- **Task ID:** P2-E2-T1
- **Title:** Active window Observation stream
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E2-T1; brief `docs/handoffs/P2-E2-T1-pm-brief.md`

## What changed
- Extended `crates/plugin-sdk` with `BioFocusPlugin`, `Capability`, `PluginError`.
- New `crates/macos-collector`: poll frontmost app (≥1s), emit `context_window` Observations on change; `NSWorkspace` on macOS (no Accessibility); injectable mock probe for tests.
- Desktop `ingest_host` clones Observation channel → starts/stops `ActiveWindowPlugin` with ingest lifecycle (channel → persist worker; UI still ↛ SQLite).
- Documented payload in `docs/07-contracts.md`, plugin notes in `08-plugin-sdk.md`, privacy in `10-security.md`, commands in `12-development.md`.

## How to verify (commands)
```bash
cargo test -p macos-collector
cargo test -p plugin-sdk
cargo check -p desktop
cargo test -p desktop
cargo test -p ingest
```

Optional manual (macOS): `pnpm tauri dev` → switch apps → inspect SQLite `observations` for `data_type = 'context_window'` (or log/channel); quit cleanly.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: macOS (or adapter) emits Observation `data_type = context_window`
- [ ] AC2: payload metadata only (`bundle_id`, `app_name`) — no keystrokes/clipboard/screenshots/window_title
- [ ] AC3: poll ≥1s / emit on change; no busy-loop (`tokio::interval` + `select!` stop)
- [ ] AC4: stop/pause joins cleanly (`stop_stream` / host shutdown)
- [ ] AC5: Observations via Core channel → persist worker; UI ↛ SQLite
- [ ] AC6: tests with mock probe (emit on change + stop idle)
- [ ] AC7: this handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; glossary (`Observation`); idle footprint

## Risks / not covered
- Full GUI smoke + live SQLite inspection not run in this session (mock probe covers AC logic).
- On non-macOS, system probe returns `None` (no emissions) — expected.
- First poll tick may emit immediately after start (by design: establish baseline frontmost).
- Window title deferred (would need Accessibility) → T2+.

## Notes for QA
- Provider id: `com.biofocus.macos.context`.
- Payload contract: `docs/07-contracts.md` § context_window.
- Commit only after PM Done (repo policy: commit per task).
