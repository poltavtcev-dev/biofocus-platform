# PM Brief → Dev: P2-E2-T2

**From:** PM  
**To:** Dev (+ PM privacy check on ship)  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E2-T1 — QA Pass with notes (`docs/handoffs/P2-E2-T1-qa-to-pm.md`)

## Task
**P2-E2-T2 — Keystroke / input aggregates (privacy-safe)**

## Why
Active window (`context_window`) is live. Next context signal is **input intensity** — counts/rates only — so Features can later use activity without ever storing typed content.

## Acceptance Criteria
1. Collector emits `Observation` with `data_type` aligned to schema (`keystrokes` / agreed aggregate type in `docs/04-storage.md` + `docs/07-contracts.md`).
2. Payload is **aggregates only** (e.g. counts / rates in a time window) — **never** characters, key codes that reconstruct text, clipboard, or screenshots.
3. Accessibility (or other) permissions required for capture are **documented** in `docs/10-security.md` / `docs/12-development.md`; deny/unavailable → no panic, graceful idle.
4. User can **disable** this collector with a single setting/flag (default-off or explicit opt-in preferred for privacy).
5. Same Core path as T1: channel → persist worker; UI ↛ SQLite; stop/pause joins without busy-loop.
6. Tests cover aggregate emit + no content leakage (mock OS APIs OK); idle DoD.
7. Handoff: `docs/handoffs/P2-E2-T2-dev-to-qa.md`.

## Out of scope
- Window title capture (unless trivial and privacy-cleared — prefer defer)
- Full collector integration epic → **P2-E2-T3**
- iOS / HealthKit → **P2-E3-***
- Feature pipeline, dashboard
- Changing E1 ingest contracts or T1 `context_window` payload shape

## Constraints
- No `unwrap`/`expect` in production
- Idle footprint DoD
- **Privacy gate:** PM reviews payload schema before merge; if Accessibility is required, document prompt + off path
- Prefer extending `crates/macos-collector` / `plugin-sdk` traits (`docs/08-plugin-sdk.md`)

## Hygiene
- Git: **commit P2-E2-T1 first** (still uncommitted at QA time), then implement T2; **push + PR once per sprint**.
- Carry note from T1: one manual GUI smoke (`pnpm tauri dev` → switch apps → SQLite `context_window`) before/at sprint PR.

## After QA Pass
PM → Ready **P2-E2-T3** (collector tests + pause idle); ∥ **P2-E0-T1** still available.
