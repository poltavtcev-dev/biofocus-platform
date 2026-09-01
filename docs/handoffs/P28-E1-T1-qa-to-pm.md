# QA → PM: P28-E1-T1

## Meta
- **Task ID:** P28-E1-T1
- **Title:** Companion LAN connectivity — fail-fast probe, clear errors, operator path
- **Verdict:** **Pass with notes**
- **Date:** 2026-09-01
- **Dev handoff:** `docs/handoffs/P28-E1-T1-dev-to-qa.md`

## What was verified
```bash
cargo test -p ingest     # 54 passed (lib + integration)
cargo test -p companion  # 8 passed
cargo check -p desktop   # ok (1 pre-existing dead_code warning in series_host)
```

| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 iOS preflight | **Pass** | `fetchStatus` 5s session; loopback guard `#if !targetEnvironment(simulator)`; preflight before Send/Flush |
| AC2 iOS HTTP timeouts | **Pass** | Separate 5s/10s sessions; `IngestClientError` variants |
| AC3 Desktop Companion UX | **Pass** | Warnings, gated Copy URL, LAN checkbox |
| AC4 Persist LAN opt-in | **Pass** | `ingest_lan_enabled` file; IPC get/set; needs restart copy |
| AC5 Docs | **Pass** | `12-development.md` table + iOS README |
| AC6 Tests | **Pass** | ingest + companion green |
| AC7 Handoff | **Pass** | This file |

## Security / boundaries
- **Pass:** Bearer unchanged; default loopback; LAN explicit opt-in; UI ↛ SQLite; no biometrics in status payload.

## Gaps (non-blocking)
- Physical iPhone LAN end-to-end not exercised in this QA session (code + unit/integration tests only).
- Ingest hot-rebind on LAN toggle deferred — user must restart Desktop (documented in UI).

## PM actions
- [ ] Mark **P28-E1-T1** Done in `docs/SPRINT_ROADMAP.md` + canvas
- [ ] Queue **P28-E2-T1** (iPhone buffer hardening) as next Ready
- [ ] Fold into post-freeze ship cluster with Phase 5/15/17 companion code

## Evidence
- `docs/handoffs/P28-E1-T1-dev-to-qa.md`
- ADR-029 · `docs/decision-log.md`
