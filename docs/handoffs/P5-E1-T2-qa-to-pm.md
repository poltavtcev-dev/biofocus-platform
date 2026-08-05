# QA → PM: P5-E1-T2

## Meta
- **Task ID:** P5-E1-T2
- **Title:** Advertise bind mode + base URL hints
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P5-E1-T2-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p ingest` — **48 passed** (lib 24 + ingest_http 18 + persist 2 + pairing_token 4)
  - `cargo test -p desktop --lib pairing_` — **6 passed** (loopback/LAN advertise + path-free JSON)
  - `cargo test -p desktop --lib status_` — **4 passed** (Menubar `get_status` lean path unchanged)
- AC results:
  - **AC1 Pass** — LAN opt-in exposes `bind_mode` / `bindMode` + `base_url_hints` / `baseUrlHints` via HTTP status + pairing IPC; concrete bind host and injected discovery covered in unit/integration tests
  - **AC2 Pass** — status/pairing JSON assertions: no Observation fields, no `.biofocus` / `/Users` paths; `/v1/status` has no token field
  - **AC3 Pass** — loopback advertise stable `http://127.0.0.1:8787` (ingest + desktop tests)
  - **AC4 Pass** — `get_status_lan_advertise_shape`, advertise unit tests, `pairing_info_lan_advertise_uses_primary_hint`, `resolve_pairing_advertise_*`
  - **AC5 Pass** — hints derived in `AdvertiseInfo::for_bind` / status build / pairing resolve; no spin loops added
  - **AC6 Pass** — `docs/09-api.md` + `docs/12-development.md` (curl/jq note); security §1.1 pointer
  - **AC7 Pass** — `docs/handoffs/P5-E1-T2-dev-to-qa.md` present
  - **Global DoD Pass** — `unwrap`/`expect` only in `#[cfg(test)]`; UI↛DB; no schema change
- Extra checks:
  - Bearer still required on LAN bind (`live_server_on_lan_bind_still_requires_bearer` still green)
  - Menubar IPC `get_status` not bloated with advertise fields (intentional)

## Defects (if any)
- None blocking.

## Notes (non-blocking)
1. If OS LAN discovery fails under `BIOFOCUS_INGEST_LAN=1` (no outbound route), `base_url_hints` may be empty while `bind_mode` stays `lan`; pairing `ingestBaseUrl` then falls back to loopback — dogfood should prefer explicit `BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>` or check hints before phone pairing.
2. Full Companion LAN UI / QR copy polish remains **P5-E2-T1** (out of scope; types already forward-compatible).

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P5-E1-T2 → Done; Ready **P5-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: status line in `ARCHITECTURE_STATUS` / `12-development` footer (Dev already updated API + how-to-read-hint)
- [x] Epic P5-E1 can be marked closed when T2 Done (T1 already Done)

## Suggested next Ready task
- **P5-E2-T1** — Companion UI: LAN base URL + token/QR (consume `bindMode` / `baseUrlHints` / `ingestBaseUrl`)

## Notes for PM
- Branch: `phase/5-wearable-dogfood`. No PR requested in this chat; code cluster continues on same branch.
- Evidence: this file + Dev handoff.
