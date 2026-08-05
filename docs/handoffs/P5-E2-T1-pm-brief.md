# PM Brief → UX + Dev: P5-E2-T1

**From:** PM  
**To:** UX + Dev  
**Status:** Done (QA Pass with notes, 2026-08-05)  
**Date:** 2026-08-05  
**Closed previous:** P5-E1-T2 (QA Pass with notes — `bind_mode` / `base_url_hints` on `/v1/status` + pairing IPC; Epic **P5-E1** ✅)  
**Closed this:** QA Pass with notes → `docs/handoffs/P5-E2-T1-qa-to-pm.md`; Epic **P5-E2** ✅; Ready **P5-E3-T1**  
**Evidence:** `docs/handoffs/P5-E1-T2-qa-to-pm.md`

## Task
**P5-E2-T1 — Companion UI: LAN base URL + token/QR**

## Why
Advertise hints ship on IPC (`ingestBaseUrl`, `bindMode`, `baseUrlHints`), but Companion still needs calm, copyable LAN (or loopback) pairing UX so a physical phone can reach Desktop without guessing the IP.

## Acceptance Criteria
1. Companion UI shows a copyable base URL — prefer primary LAN hint when available; otherwise loopback (Simulator / same-machine).
2. Existing token Show / Copy / QR still works (`get_pairing_token`).
3. Calm, non-evaluative copy that LAN is **opt-in** / local network only (no cloud account language).
4. Data only via IPC; UI ↛ SQLite.
5. Idle-safe: no busy-loop poll of pairing/status.
6. Manual smoke steps in handoff (loopback default + LAN opt-in path).
7. Handoff: `docs/handoffs/P5-E2-T1-dev-to-qa.md`.

## Out of scope
- iOS app / HealthKit runnable (→ **P5-E3-T1**)
- Changing ingest auth / Bearer scheme
- TLS, mDNS discovery, new SQLite schema
- Menubar `get_status` advertise bloat (keep lean; pairing IPC is the surface)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `apps/desktop` (Companion section + IPC glue); types already in `pairing.ts`
- Reuse `bindMode` / `baseUrlHints` / `ingestBaseUrl` from P5-E1-T2 — do not re-derive LAN IP in the UI
- QA note: if `baseUrlHints` empty under LAN mode, surface fallback clearly; dogfood may need `BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>`
- Branch: `phase/5-wearable-dogfood`

## IPC reference (from E1-T2)
- `invoke("get_pairing_token")` → `token`, `ingestBaseUrl`, `bindMode`, `baseUrlHints`, `qrSvg`
- Contract: `docs/09-api.md` · how-to-read: `docs/12-development.md` (Base URL hint)

## After QA Pass
~~PM → Ready **P5-E3-T1**~~ — done; see `docs/handoffs/P5-E3-T1-pm-brief.md`.
