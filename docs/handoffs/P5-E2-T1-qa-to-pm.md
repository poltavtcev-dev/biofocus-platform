# QA → PM: P5-E2-T1

## Meta
- **Task ID:** P5-E2-T1
- **Title:** Companion UI: LAN base URL + token/QR
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P5-E2-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
### Commands
```bash
cd apps/desktop && pnpm exec tsc --noEmit   # exit 0
cargo test -p desktop --lib pairing_       # 6 passed
```
- Boundary: no `rusqlite` / `biofocus_main.db` / sqlite usage under `apps/desktop/src`.
- Idle: pairing fetched once on mount + manual Reload; `setInterval` remains Menubar status ~5s only (no pairing poll).

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Copyable base URL (LAN primary / else loopback) | **Pass** — UI shows `primaryBaseUrl` = IPC `ingestBaseUrl`; no client-side IP discovery |
| AC2 Token Show / Copy / QR | **Pass** — controls preserved under Token block |
| AC3 Calm LAN opt-in / local-only copy | **Pass** — intro + `networkModeDetail`; no cloud-account CTA |
| AC4 UI ↛ SQLite / IPC only | **Pass** — `invoke("get_pairing_token")` only |
| AC5 Idle-safe | **Pass** — no pairing busy-loop |
| AC6 Manual smoke steps in handoff | **Pass (doc)** — steps A/B in `dev-to-qa`; live Menubar dogfood not run in this QA session |
| AC7 Handoff present | **Pass** |
| Global DoD | **Pass** — TS/UI only; no new unwrap paths; glossary-neutral copy |

### Extra checks
- LAN empty-hints path: UI `needsLanHintFallback` triggers when `bindMode=lan` and primary URL is loopback (covers host fallback when discovery empty). Note: `normalize()` may coerce empty `baseUrlHints` to `[ingestBaseUrl]` — fallback still surfaces via loopback check.
- Out of scope respected: no ingest auth/TLS/mDNS/schema; Menubar `get_status` untouched.

## Defects (if any)
- None blocking. Live smoke A/B left for operator on `phase/5-wearable-dogfood` Desktop run.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P5-E2-T1 → Done; Ready **P5-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs: `docs/12-development.md` — mark Companion LAN UI done (retire “→ P5-E2-T1”); optionally `docs/14-roadmap.md` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` Phase 5 next pointer

## Suggested next Ready task
- **P5-E3-T1** — Runnable iOS companion + HealthKit one-shot (per brief / roadmap)

## Notes for PM
- Branch remains `phase/5-wearable-dogfood`; no PR requested in this chat.
- Desktop README already mentions Base URL surface; fold roadmap/status docs in pm-close.
