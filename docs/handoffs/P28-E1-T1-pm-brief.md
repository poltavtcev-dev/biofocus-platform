# PM Brief → Dev|UX: P28-E1-T1

**From:** PM  
**To:** Dev (+ UX)  
**Status:** Ready  
**Date:** 2026-08-23  
**Priority:** **User blocker** — physical iPhone Companion times out; must be stable before post–freeze ship cluster  
**Closed previous:** — (interrupts Phase 27 E3 docs-only; parallel fix)  
**Phase:** Phase 28 — Local device reliability **(A+B)** — **ADR-029** ✅  
**Branch:** `phase/28-local-reliability` (E1 may share `phase/28-companion-connectivity`)

## Task
**P28-E1-T1 — Companion LAN connectivity: fail-fast probe, clear errors, operator path**

## Why (diagnosis)
Physical iPhone **cannot** reach Desktop on `http://127.0.0.1:8787` — that is the phone’s own loopback. Default Desktop bind is loopback-only (ADR-005). iOS `IngestClient` uses `URLSession.shared` with **no short timeout** and **no preflight** — user waits ~60s then sees generic “timed out”.

Common failure modes (all look like “timeout” today):
1. Desktop started **without** `BIOFOCUS_INGEST_LAN=1` → ingest listens on Mac loopback only.
2. User pasted **loopback** Base URL into phone (Desktop Companion still shows `127.0.0.1` when LAN off).
3. LAN on but **discovery empty** → UI fallback still loopback; user didn’t set `BIOFOCUS_INGEST_BIND_HOST=<mac-lan-ip>`.
4. **macOS Firewall** / guest Wi‑Fi AP isolation — TCP never reaches `:8787`.
5. iOS **Local Network** permission denied (ATS/local-network keys exist; no in-app guidance on deny).
6. Packaged `.app` from Finder — **no env** → LAN never enabled without launch wrapper.

Code path itself (Bearer POST `/v1/ingest`, queue + flush) is shipped and tested on loopback/LAN in CI — gap is **reachability UX + operator docs**, not ingest contract.

## Acceptance Criteria
1. **iOS preflight:** “Test connection” (or automatic before Send/Flush) → `GET {baseURL}/v1/status` with **≤5s** timeout; calm errors:
   - loopback URL on physical device → block with “use LAN Base URL from Desktop, not 127.0.0.1”;
   - timeout / unreachable → “Desktop not reachable — same Wi‑Fi, LAN bind on Mac, firewall”;
   - `401` N/A on status (unauthenticated); show `bind_mode` + `db_status` on success.
2. **iOS HTTP client:** explicit `URLSessionConfiguration` timeout (connect + request ≤10s for ingest POST); distinguish timeout vs refused vs unauthorized.
3. **Desktop Companion UX:** when `bindMode=loopback`, prominent calm warning “Physical iPhone needs LAN — enable below” + documented steps; when `bindMode=lan`, Base URL must **not** be loopback unless `needsLanHintFallback` (existing helper — surface as error state, not copyable primary).
4. **Optional thin Dev (if ≤1 screen):** persist LAN opt-in preference (UserDefaults/plist) so packaged `.app` can enable LAN bind on next launch **without** shell env — **only if** fits ADR-005 (still opt-in, Bearer unchanged). If ADR touch needed → note in handoff, don’t ship silent always-on LAN.
5. **Docs:** `docs/12-development.md` § Companion — troubleshooting table (timeout → checklist: LAN env, bind_host, curl from phone network, firewall, Local Network permission); `apps/companion/ios/README.md` mirror.
6. **Tests:** Rust companion client timeout/error mapping if shared; Swift unit test for loopback-on-device guard if testable; existing `cargo test -p ingest` / companion green.
7. Handoff: `docs/handoffs/P28-E1-T1-dev-to-qa.md`.

## Out of scope
- mDNS/Bonjour auto-discovery (ADR-005 rejected for Phase 5)
- TLS / cloud relay / pairing redesign
- QR for Base URL (nice follow-up, not required)
- Changing ingest auth or SQLite schema
- PR during freeze

## Constraints
- ADR-005: default loopback; LAN remains **explicit opt-in**
- Bearer required on `POST /v1/ingest`
- UI ↛ SQLite; no clinical copy
- Global DoD from `SPRINT_ROADMAP.md`

## Operator smoke (QA / dogfood)
1. Desktop **without** LAN → iOS Test connection → fail-fast (&lt;5s) with loopback/LAN message (not 60s hang).
2. `export BIOFOCUS_INGEST_LAN=1` + restart → Companion shows `http://192.168.x.x:8787` → curl from Mac OK → iOS Test → success → Send HR → SQLite row.
3. Wrong token → POST 401 → “check pairing token” (not timeout).
4. Firewall block (optional manual) → Test → unreachable message with firewall hint.

## After QA Pass
PM → mark P28-E1-T1 Done; continue **P28-E2-T1** (buffer) → **P28-E3-T1** (always-on) → **P28-E4-T1** (replay). Optional **P28-E5-T1** in parallel after E1.

## Next chat (скопируй в новый чат — **user blocker, можно до P27-E3**)
```
как агент: режим build-qa для P28-E1-T1.
Brief: docs/handoffs/P28-E1-T1-pm-brief.md
ADR: docs/decision-log.md ADR-029
1) Как Dev|UX — собери по AC, создай docs/handoffs/P28-E1-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P28-E1-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
