# QA → PM: P19-E3-T1

## Meta
- **Task ID:** P19-E3-T1
- **Title:** Dogfood runbook + verify `NotificationPressure` on live emits
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P19-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
rg -n "Notification events dogfood|BIOFOCUS_NOTIFICATION_EVENTS|Full Disk Access|get_feature_snapshot" \
  docs/12-development.md docs/08-plugin-sdk.md docs/10-security.md docs/07-contracts.md docs/16-glossary.md
# runbook + cross-links present

git diff --name-only -- crates/feature-engine apps/desktop
# empty for this task scope (docs-only companion)
```

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 Dogfood runbook + cross-links | **Pass** | § Notification events dogfood; FDA; soft-fail; privacy; Feature expect |
| AC2 Verify Feature path | **Pass** | `get_feature_snapshot` / series; fixture tests; optional operator sqlite count; UI↛SQLite |
| AC3 Calm soft-fail note | **Pass** | Docs calm note; no Menubar IPC (optional; docs-only OK); no chart |
| AC4 Tests | **Pass** | N/A — docs-only, no new IPC |
| AC5 No rewrite / migration | **Pass** | Contracts unchanged; no Feature/desktop code |
| AC6 Phase 19 companion shipped | **Pass** | glossary + `12-development` status bullet; ADR-020 E3 shipped |
| AC7 Handoff | **Pass** | `docs/handoffs/P19-E3-T1-dev-to-qa.md` |
| Global DoD | **Pass** | Personal self-tracking; PR freeze |

### Extra checks
- Runbook explicitly forbids Accessibility scrape and workplace framing.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P19-E3-T1 → Done; close Epic **P19-E3** and **Phase 19**; open **PM-GATE-POST-P19**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx`
- [x] Status docs + `PM-GATE-POST-P19-pm-brief.md`

## Suggested next Ready task
- **PM-GATE-POST-P19** — choose Phase 20+ · `docs/handoffs/PM-GATE-POST-P19-pm-brief.md`

## Notes for PM
- Branch: `phase/19-live-nc-mapping`
- PR freeze until 2026-09-01 — do not open PR.
- Optional Menubar mapping-status IPC was not shipped (docs calm note sufficient per brief).
