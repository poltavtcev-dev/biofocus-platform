# PM Brief → Dev: P5-E1-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P5-E1-T1 (QA Pass with notes — opt-in LAN bind `BIOFOCUS_INGEST_LAN` / `BIOFOCUS_INGEST_BIND_HOST`, ADR-005; pairing `ingestBaseUrl` still hardcodes loopback)

## Task
**P5-E1-T2 — Advertise bind mode + base URL hints**

## Why
LAN opt-in binds the server, but Companion / pairing still advertise `http://127.0.0.1:…`. Physical-phone dogfood needs a usable base URL hint (and bind mode) from status/IPC without exposing Observations or absolute DB paths.

## Acceptance Criteria
1. When LAN opt-in is on, host exposes bind mode + usable base URL hint(s) (e.g. primary LAN IPv4 + port) via documented status and/or IPC.
2. Payload must **not** include Observation biometrics or absolute DB paths.
3. Loopback mode still reports a loopback URL (Simulator / same-machine unchanged).
4. Tests cover status/IPC shape for loopback vs LAN opt-in.
5. Idle-safe (no busy-loop; derive hints on read / startup, not spin).
6. Short docs note (`12-development` and/or `09-api`) on how to read the hint.
7. Handoff: `docs/handoffs/P5-E1-T2-dev-to-qa.md`.

## Out of scope
- Full Companion pairing QR / UI redesign (→ **P5-E2-T1**)
- iOS app work (→ **P5-E3**)
- Changing auth scheme, TLS, mDNS discovery
- New SQLite schema

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `crates/ingest`, desktop host (`apps/desktop/src-tauri`); optionally `/v1/status` and/or pairing IPC (`ingestBaseUrl`)
- Reuse P5-E1-T1 knobs; do not weaken Bearer on `POST /v1/ingest`
- Branch: `phase/5-wearable-dogfood`
- Note from QA T1: `GET /v1/status` is unauthenticated (pre-existing) — if extending it for LAN hints, keep payload free of Observations / secrets

## After QA Pass
PM closes → Ready **P5-E2-T1** (Companion UI: LAN base URL + token/QR).
