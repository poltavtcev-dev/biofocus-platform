# Dev|UX → QA: P5-E2-T1

## Meta
- **Task ID:** P5-E2-T1
- **Title:** Companion UI: LAN base URL + token/QR
- **Role that built:** UX + Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P5-E2 / P5-E2-T1; brief `docs/handoffs/P5-E2-T1-pm-brief.md`
- **Branch:** `phase/5-wearable-dogfood`

## What changed
- Companion section shows a dedicated **Base URL** block: primary IPC `ingestBaseUrl` (LAN hint when available, else loopback), **Copy URL**, **Reload**.
- Calm copy: local pairing / no cloud; LAN labelled as opt-in / same Wi‑Fi only.
- When `bindMode=lan` but hints empty / primary still loopback → clear fallback note suggesting `BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>`.
- Token Show / Copy / QR unchanged; copy feedback distinguishes URL vs token.
- Helpers in `pairing.ts` (`primaryBaseUrl`, `networkModeLabel` / `Detail`, `needsLanHintFallback`) — UI does not re-derive LAN IP.
- Pairing still one-shot on mount (+ manual Reload); no pairing busy-loop. Status poll (~5s) unchanged.
- `apps/desktop/README.md` notes P5-E2-T1 Base URL surface.

## Crates / apps / files touched
- `apps/desktop/src/pairing.ts`
- `apps/desktop/src/App.tsx`
- `apps/desktop/src/App.css`
- `apps/desktop/README.md`

## How to verify (commands)
```bash
cd apps/desktop && pnpm exec tsc --noEmit
cargo test -p desktop --lib pairing_
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Companion UI shows copyable base URL — primary LAN hint when available; otherwise loopback
- [ ] AC2: Token Show / Copy / QR still works (`get_pairing_token`)
- [ ] AC3: Calm copy that LAN is opt-in / local network only (no cloud account language)
- [ ] AC4: Data only via IPC; UI ↛ SQLite
- [ ] AC5: Idle-safe — no busy-loop poll of pairing/status beyond existing Menubar status ~5s
- [ ] AC6: Manual smoke steps below (loopback default + LAN opt-in path)
- [ ] AC7: This handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Manual smoke (AC6)
### A — Loopback default
1. Start Desktop **without** `BIOFOCUS_INGEST_LAN` / bind override.
2. Open Menubar shell → Companion.
3. Expect Base URL like `http://127.0.0.1:8787`, label **This Mac**, Copy URL works.
4. Token Show / Copy / Show QR still work.

### B — LAN opt-in
1. Restart Desktop with `export BIOFOCUS_INGEST_LAN=1` (optional: `BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>` if discovery empty).
2. Companion → Reload (or relaunch).
3. Expect Base URL = primary LAN hint when discovery/bind host works; label **Local network**; copy mentions opt-in / same Wi‑Fi.
4. If LAN on but address unavailable: label **Local network (address unavailable)** + fallback copy with `BIOFOCUS_INGEST_BIND_HOST`.

## Risks / not covered
- No automated frontend unit tests (no vitest in desktop package); logic is pure helpers + visual smoke.
- Live LAN URL depends on host discovery / env — CI may only prove loopback IPC shape (E1-T2 tests).
- iOS runnable / HealthKit → **P5-E3-T1** (out of scope).

## Notes for QA
- Do not expect Menubar `get_status` to grow advertise fields (lean by design).
- Grep boundary: frontend must not import rusqlite / open DB.
