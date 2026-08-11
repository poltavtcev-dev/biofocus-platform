# QA → PM: P18-E2-T1

## Meta
- **Task ID:** P18-E2-T1
- **Title:** Implement notification collector plugin per ADR-019
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P18-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
cargo test -p bio-spec          # ok (incl. contracts + unit validators)
cargo test -p pipeline          # ok (normalize strip / reject tests)
cargo test -p ingest            # ok
cargo test -p macos-collector --test collector_integration notification_event
# 3 passed: emit→storage (no content keys), stop freezes polls, system soft-fail idle
```

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 Plugin id + capability | **Pass** | `com.biofocus.macos.notifications` / `notification_events` → `notification_event` |
| AC2 ingest_host env gate | **Pass** | Arms only when `notification_events_enabled()`; default off |
| AC3 ADR payload / no content | **Pass** | Payload builder + normalize strip; integration asserts no title/body/… |
| AC4 Soft-fail + scripted + ≥5s + stop | **Pass** | System → `Ok(None)`; DEFAULT ≥5s; stop poll freeze test |
| AC5 bio-spec + pipeline + ingest | **Pass** | Validator + `invalid_notification_event` + normalize |
| AC6 Docs as shipped | **Pass** | `07` / `08` / `10` / `12` mark E2 collector shipped; Feature → E3 |
| AC7 Tests | **Pass** | See commands above |
| AC8 No Feature / migration / content | **Pass** | No Feature formula in feature crates; no migration; soft-fail idle |
| AC9 Handoff | **Pass** | `docs/handoffs/P18-E2-T1-dev-to-qa.md` |
| Global DoD | **Pass** | UI↛DB; PR freeze; personal self-tracking framing in docs |

### Extra checks
- Privacy: pipeline strips accidental content keys; collector payload never inserts them.
- System probe intentionally idle until a privacy-safe OS mapping exists (documented risk for E3).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P18-E2-T1 → Done; Ready **P18-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE / todos / stats / callout / DAG
- [ ] Optional: `ARCHITECTURE_STATUS` / decision-log “shipped” note if PM tracks E2 there (contracts already updated in Dev pass)

## Suggested next Ready task
- **P18-E3-T1** — catalog Feature `NotificationPressure` (HRV-style omit-when-empty + confidence per ADR-019 / ADR-007)

## Notes for PM
- Branch: `phase/18-notification-pressure` (dirty tree may still include E1 docs + this E2 code — commit when clustering).
- PR freeze until 2026-09-01 — do not open PR.
- Real Notification Center OS mapping remains soft-fail; Feature can still ship against scripted / future probe evidence.
