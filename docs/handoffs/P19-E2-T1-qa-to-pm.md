# QA → PM: P19-E2-T1

## Meta
- **Task ID:** P19-E2-T1
- **Title:** Implement live `SystemNotificationEventProbe` mapping (ADR-020)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P19-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
cargo test -p macos-collector notification
# 8 passed (unit: fixture allowlist, soft-fail missing DB, live delta emit)

cargo test -p macos-collector --test collector_integration notification_event
# 4 passed (scripted→storage, stop freeze, soft-fail no emit, live fixture→storage)
```

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 Live mapping / no content | **Pass** | usernoted SQLite; SELECT `delivered_date` + `app.identifier` only; payload has no title/body/`bundle_id` |
| AC2 Soft-fail + idle cadence | **Pass** | Missing DB → None; stream ≥5s default; stop_stream freeze test |
| AC3 ingest_host env gate | **Pass** | Unchanged `BIOFOCUS_NOTIFICATION_EVENTS`; channel path intact |
| AC4 Tests | **Pass** | Soft-fail / scripted / fixture live / stop / no content keys |
| AC5 Docs name OS surface | **Pass** | `07` / `08` / `10` / `12` / `16` mark usernoted path shipped |
| AC6 No Feature/migration/widen | **Pass** | No NotificationPressure rewrite; no migration; no bundle_id field |
| AC7 Handoff | **Pass** | `docs/handoffs/P19-E2-T1-dev-to-qa.md` |
| Global DoD | **Pass** | UI↛DB; personal self-tracking; PR freeze |

### Extra checks
- SQL string in `query_deliveries_after` does not reference `data` / title / body.
- Real-device dogfood may need Full Disk Access; soft-fail without it is intentional (E3 runbook).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P19-E2-T1 → Done; Ready **P19-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE / todos / stats / callout / DAG
- [ ] Optional: fold remaining Phase 19 open docs on branch when clustering

## Suggested next Ready task
- **P19-E3-T1** — Dogfood runbook / optional calm status when NC mapping unavailable (FDA note)

## Notes for PM
- Branch: `phase/19-live-nc-mapping`
- PR freeze until 2026-09-01 — do not open PR.
- Live `interruption_level` not filled from OS in v1 (would need content blobs) — count + category/app_kind sufficient for `NotificationPressure`.
