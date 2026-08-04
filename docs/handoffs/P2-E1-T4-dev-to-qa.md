# Dev → QA: P2-E1-T4

## Meta
- **Task ID:** P2-E1-T4
- **Title:** Host wire + `GET /v1/status` + idle DoD
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E1-T4; brief `docs/handoffs/P2-E1-T4-pm-brief.md`
- **Branch:** `phase/2-ingest-http`

## What changed
- Desktop host on startup: default DB open → `IngestConfig::load()` → `observation_channel` → `spawn_persist_worker` → loopback serve (`127.0.0.1`, default port 8787).
- Clean shutdown on Tauri `ExitRequested`: stop accept (oneshot) + join persist/discard worker (no orphan spin).
- `GET /v1/status` returns snake_case `{version, db_status, db_error?}` **without** Observation / biometric fields; soft-fail DB probe → `db_status: "error"`.
- Shell UI unchanged: still IPC `get_status` for menubar; HTTP status is companion/debug only.
- Idle: Axum accept + worker `blocking_recv` (no busy-loop).

### Crates / files
- `crates/ingest/src/status.rs` (new)
- `crates/ingest/src/{lib,routes,server}.rs` — `IngestState` builders, status route, `serve_*` take `IngestState`
- `crates/ingest/tests/ingest_http.rs` — status shape + soft error + temp DB probe
- `apps/desktop/src-tauri/src/ingest_host.rs` (new)
- `apps/desktop/src-tauri/src/lib.rs`, `Cargo.toml`
- `docs/09-api.md`, `docs/12-development.md`, `docs/10-security.md`

## How to verify (commands)
```bash
cargo check -p ingest
cargo test -p ingest
cargo check -p desktop
cargo test -p desktop
```

### Manual smoke (optional; not flaky full-app in CI)
```bash
cd apps/desktop && pnpm tauri dev
# another terminal:
curl -s http://127.0.0.1:8787/v1/status
# expect: {"version":"0.1.0","db_status":"ok"} (or error + db_error)
# Quit app → port should stop accepting (no leftover busy process)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Desktop startup opens default DB (WAL+migrate), `IngestConfig::load`, channel → persist worker → serve with `tx`, loopback only (`127.0.0.1:8787` default)
- [ ] AC2: Clean shutdown stops accept + worker (no orphan spin)
- [ ] AC3: `GET /v1/status` → version + db ok|error, no Observation / biometric payload
- [ ] AC4: Idle — no busy-loop after bind (tokio accept + blocking_recv)
- [ ] AC5: Shell UI still IPC `get_status`; HTTP status for companion/debug only
- [ ] AC6: Tests cover status JSON shape + soft db error mapping; manual smoke documented
- [ ] AC7: Handoff present (`docs/handoffs/P2-E1-T4-dev-to-qa.md`)
- [ ] Global DoD: no `unwrap`/`expect` in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Full Tauri app launch is **manual smoke only** (CI runs `cargo test -p desktop` unit tests, not GUI).
- If default DB open fails at host start: HTTP still starts; Observations are drained/discarded with warn logs; `/v1/status` reports soft `error`.
- Port conflict (8787 in use) → serve error logged; app shell still runs.
- QR / pairing UX → **P2-E3-T2**; collectors → E2; sprint PR → later.

## Notes for QA
- HTTP JSON is **snake_case** (`db_status`); IPC remains **camelCase** (`dbStatus`) — intentional.
- `IngestState::new` / `with_version` / `with_db_probe` / `with_db_path` for tests and host.
- Public API still: `bind_loopback`, `serve_listener`, `serve_with_shutdown` (now take `IngestState`), `StatusResponse`, `probe_db_at`.
