# Dev → QA: P5-E1-T2

## Meta
- **Task ID:** P5-E1-T2
- **Title:** Advertise bind mode + base URL hints
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P5-E1 / P5-E1-T2; brief `docs/handoffs/P5-E1-T2-pm-brief.md`
- **Branch:** `phase/5-wearable-dogfood`

## What changed
- New `crates/ingest/src/advertise.rs`: `BindMode` (`loopback` | `lan`), `AdvertiseInfo` with `base_url_hints`; derived on read (UDP primary-LAN trick when bind is `0.0.0.0`; concrete bind host used as-is; loopback → `http://127.0.0.1:<port>`).
- `GET /v1/status` now returns `bind_mode` + `base_url_hints` (no Observations / tokens / DB paths).
- IPC `get_pairing_token`: `bindMode`, `baseUrlHints`, and `ingestBaseUrl` = primary hint (LAN when opt-in).
- Desktop host wires advertise from `IngestConfig`; `serve_with_shutdown` refreshes hints with actual bound port.
- Docs: `09-api.md`, `12-development.md`, `10-security.md` §1.1; companion README reachability note.
- Frontend `pairing.ts` types/normalize accept new fields (UI redesign still E2).

## Crates / apps / files touched
- `crates/ingest/src/{advertise.rs,status.rs,routes.rs,server.rs,lib.rs}`
- `crates/ingest/tests/ingest_http.rs`
- `apps/desktop/src-tauri/src/{lib.rs,ingest_host.rs}`
- `apps/desktop/src/pairing.ts`
- `apps/companion/README.md`
- `docs/{09-api,10-security,12-development}.md`

## How to verify (commands)
```bash
cargo test -p ingest
cargo test -p desktop --lib pairing_
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: LAN opt-in → status/IPC expose `bind_mode=lan` + usable `base_url_hints` (primary LAN IPv4 + port when discovery works; concrete `BIOFOCUS_INGEST_BIND_HOST` always)
- [ ] AC2: Payload has no Observation biometrics / absolute DB paths / pairing token in `/v1/status`
- [ ] AC3: Loopback mode reports loopback URL (`http://127.0.0.1:8787`)
- [ ] AC4: Tests cover loopback vs LAN status/IPC shape
- [ ] AC5: Idle-safe (hints on read/startup; no busy-loop)
- [ ] AC6: Docs note how to read the hint (`12-development` + `09-api`)
- [ ] AC7: This handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- OS LAN discovery can return empty `base_url_hints` on exotic/offline nets while `bind_mode` stays `lan` — pairing falls back to loopback URL for `ingestBaseUrl` only when hints empty (phone still needs a real LAN IP).
- Full Companion UI / QR redesign → **P5-E2-T1** (out of scope).
- Menubar IPC `get_status` intentionally unchanged (stays lean).

## Notes for QA
- Injected discovery covered by unit tests (`AdvertiseInfo::for_bind_with`); live discovery exercised lightly by `resolve_pairing_advertise_lan_flag_is_lan_mode` (mode must be `lan`; hints optional if CI has no route).
- `/v1/status` remains unauthenticated (pre-existing T1 note) — verify no secrets in body.
