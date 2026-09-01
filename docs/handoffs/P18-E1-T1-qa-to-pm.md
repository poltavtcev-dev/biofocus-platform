# QA → PM: P18-E1-T1

## Meta
- **Task ID:** P18-E1-T1
- **Title:** Lock notification Observation + `NotificationPressure` scope as ADR-019 + docs
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P18-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg ADR-019` in decision-log + contracts docs → summary row + detail section present
  - Locked `notification_event` + `BIOFOCUS_NOTIFICATION_EVENTS` + `NotificationPressure` across ADR / `07-contracts` / `08-plugin-sdk` / catalog
  - Rejected alts include IDE / weather / App Store / content capture / workplace surveillance / `CognitiveLoad` as E3 / separate `notification_burst` family / PR freeze / no migration
  - Phase 18 epic text aligned (E1 ADR · E2 collector · E3 Feature) with locked `data_type` + env
  - `rg` over `crates/` + `apps/` → **no** collector / validator / Feature node code (expected)
  - No new bio-spec / feature-engine notification modules; storage notes say **no migration**
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-019 locks family + privacy + opt-in/plugin + no migration + Feature sketch + rejected | **Pass** |
| AC2 Contracts / catalog / security / development / glossary (+ storage) updated; E2/E3 marked | **Pass** |
| AC3 Aligned with Phase 18 epic split in SPRINT_ROADMAP | **Pass** |
| AC4 No collector emit; no Feature formula beyond stub; no migration applied | **Pass** |
| AC5 Handoff `P18-E1-T1-dev-to-qa.md` | **Pass** |
| Global DoD / PR freeze | **Pass** |

- Extra checks: one Observation family (`notification_event`) with coalescing via `count` (not a second `notification_burst` type); personal self-tracking framing explicit.

## Defects (if any)
- None blocking.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P18-E1-T1** Done; Ready **P18-E2-T1** (collector)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE / todos / stats / callout / DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` (ADR-019 locked); brief for E2

## Suggested next Ready task
- **P18-E2-T1** — Implement notification collector plugin per ADR-019 (`notification_event`, `BIOFOCUS_NOTIFICATION_EVENTS`, soft-fail idle).

## Notes for PM
- Branch: `phase/18-notification-pressure`
- PR freeze until 2026-09-01 — do not open PR.
- Working tree may also include prior PM-GATE-POST-P17 / Phase 17 close docs — fold into cluster commits as appropriate.
