# QA → PM: P19-E1-T1

## Meta
- **Task ID:** P19-E1-T1
- **Title:** ADR-020: privacy-safe Notification Center OS mapping + live probe boundaries
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P19-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
rg -n "ADR-020" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md \
  docs/10-security.md docs/12-development.md docs/16-glossary.md
# hits in all listed docs

rg -n "hybrid|field allowlist|SystemNotificationEventProbe" docs/decision-log.md
# stance + probe sketch present

# Probe still soft-fail (no live impl in E1)
rg -n "Ok\(None\)|system_notification_event" crates/macos-collector/src/notification_probe.rs
# system_notification_event → Ok(None)
```

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 One OS stance + rationale | **Pass** | Hybrid + soft-fail; vs ADR-019 idle; Capability Model; personal self-tracking |
| AC2 Privacy reaffirmed | **Pass** | Forbid body/title/… even transiently; logs prefer id/counts/labels |
| AC3 Rejected alternatives | **Pass** | Content, Accessibility, workplace, busy-loop, Feature rewrite, IDE/weather/App Store/CognitiveLoad, parallel store, public-only forever, unrestricted private, payload widen, PR freeze, migration |
| AC4 Schema none | **Pass** | ADR-019 payload unchanged; no migration |
| AC5 E2/E3 sketch | **Pass** | Live probe → channel → NotificationPressure; optional E3 dogfood/status |
| AC6 Docs planned/ADR | **Pass** | 07 / 08 / 10 / 12 / 16 updated |
| AC7 Handoff | **Pass** | `docs/handoffs/P19-E1-T1-dev-to-qa.md` |
| Global DoD | **Pass** | Docs-only E1; probe still soft-fail; PR freeze; UI↛DB |

### Extra checks
- No crate code changes for live mapping in this task (correct for E1).
- Branch: `phase/19-live-nc-mapping`.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P19-E1-T1 → Done; Ready **P19-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE / todos / stats / callout / DAG
- [x] Status docs + `P19-E2-T1-pm-brief.md`

## Suggested next Ready task
- **P19-E2-T1** — Implement live `SystemNotificationEventProbe` mapping per ADR-020 · `docs/handoffs/P19-E2-T1-pm-brief.md`

## Notes for PM
- Exact OS surface chosen in E2 must still obey ADR-020 non-content field allowlist.
- PR freeze until 2026-09-01 — do not open PR.
- No schema approve expected for E2.
