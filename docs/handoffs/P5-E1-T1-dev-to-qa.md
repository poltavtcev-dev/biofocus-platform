# Dev → QA: P5-E1-T1

## Meta
- **Task ID:** P5-E1-T1
- **Title:** Opt-in LAN ingest bind + config
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P5-E1-T1 / `docs/handoffs/P5-E1-T1-pm-brief.md`
- **Branch:** `phase/5-wearable-dogfood`

## What changed
- `IngestConfig` gains `bind_host`; default remains `127.0.0.1`.
- Opt-in knobs:
  - `BIOFOCUS_INGEST_LAN=1` (also `true`/`yes`/`on`) → bind `0.0.0.0`
  - `BIOFOCUS_INGEST_BIND_HOST=<ipv4>` → explicit override (wins over LAN flag)
- New APIs: `bind_host`, `resolve_bind_host`, constants `INGEST_LAN_ENV`, `INGEST_BIND_HOST_ENV`, `INGEST_LAN_BIND_HOST`.
- `serve_with_shutdown` / Desktop `ingest_host` use `config.bind_host`; logs LAN vs loopback.
- Bearer auth unchanged (no anonymous LAN ingest).
- Docs: `docs/10-security.md` §1.1, `docs/12-development.md`, `docs/09-api.md` bind note, **ADR-005** in `docs/decision-log.md`.
- No SQLite schema; no Companion UI / iOS work.

## Crates / apps / files touched
- `crates/ingest/src/{config,server,lib,error}.rs`
- `crates/ingest/tests/{ingest_http,pairing_token}.rs`
- `apps/desktop/src-tauri/src/{ingest_host,lib}.rs`
- `docs/{10-security,12-development,09-api,decision-log}.md`

## How to verify (commands)
```bash
cargo test -p ingest
cargo check -p desktop
cargo test -p companion --test sample_ingest
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Default bind remains `127.0.0.1` (Simulator / same-machine)
- [ ] AC2: Explicit opt-in enables LAN bind (`BIOFOCUS_INGEST_LAN` / `BIOFOCUS_INGEST_BIND_HOST` documented)
- [ ] AC3: `POST /v1/ingest` still requires Bearer — no anonymous LAN access
- [ ] AC4: Without opt-in, behavior matches loopback-only
- [ ] AC5: Tests cover default loopback + opt-in LAN bind (ephemeral port)
- [ ] AC6: Idle-safe (tokio accept; no busy-loop)
- [ ] AC7: Docs updated (`10-security`, `12-development`; `09-api` bind note)
- [ ] AC8: ADR-005 in `decision-log.md`
- [ ] AC9: No new SQLite schema; no Companion UI / iOS
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Pairing IPC `ingestBaseUrl` still reports `http://127.0.0.1:8787` even when LAN opt-in is on → **P5-E1-T2** (advertise bind mode + base URL hints).
- `GET /v1/status` remains unauthenticated (pre-existing); when LAN is on it is LAN-reachable without Bearer — payload has no Observations (documented in `09-api.md`).
- Choosing a specific NIC IP vs `0.0.0.0` is operator-driven via `BIOFOCUS_INGEST_BIND_HOST`.

## Notes for QA
- Unit tests in `config::tests` cover resolve order without mutating process env (injectable getenv).
- Integration: `bind_lan_opt_in_accepts_on_unspecified`, `live_server_on_lan_bind_still_requires_bearer`, `config_load_lan_opt_in_sets_unspecified_bind`.
