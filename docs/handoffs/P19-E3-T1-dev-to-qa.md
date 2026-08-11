# Dev → QA: P19-E3-T1

## Meta
- **Task ID:** P19-E3-T1
- **Title:** Dogfood runbook + verify `NotificationPressure` on live emits
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P19-E3-T1-pm-brief.md`

## What changed
- Dogfood runbook in `docs/12-development.md` § **Notification events dogfood**: enable `BIOFOCUS_NOTIFICATION_EVENTS=1`; Full Disk Access note for `group.com.apple.usernoted`; soft-fail idle without it; privacy bar; live → `NotificationPressure`; verify via `get_feature_snapshot` / series IPC, optional sqlite count (operator), fixture/CI path; calm soft-fail framing.
- Cross-links: `08-plugin-sdk`, `10-security`, `07-contracts`, `16-glossary`; ADR-020 E3 sketch marked shipped.
- Phase 19 companion marked shipped in glossary / development status bullet.
- **No** Menubar IPC / Dashboard chart (optional AC3 deferred as docs calm-status note — docs-only OK per brief).
- **No** Feature rewrite; **no** migration; contracts unchanged; no content blob SELECT.

## Crates / apps / files touched
- Docs only: `12-development`, `08-plugin-sdk`, `10-security`, `07-contracts`, `16-glossary`, `decision-log` (E3 note)
- Branch: `phase/19-live-nc-mapping`

## How to verify (commands)
```bash
rg -n "Notification events dogfood|BIOFOCUS_NOTIFICATION_EVENTS|Full Disk Access|NotificationPressure" \
  docs/12-development.md docs/08-plugin-sdk.md docs/10-security.md docs/07-contracts.md docs/16-glossary.md

# Privacy / no content
rg -n "never.*record.data|no body/title|workplace" docs/12-development.md docs/10-security.md

# No Feature / IPC code in this task
git diff --name-only -- 'crates/feature-engine/' 'apps/desktop/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood runbook in `12-development` + cross-links; enable env; FDA; soft-fail; Feature expect; privacy
- [ ] AC2: Verify Feature path documented (snapshot IPC + fixture + optional operator sqlite count); UI↛SQLite for product
- [ ] AC3: Calm soft-fail note (docs) — no clinical / workplace copy; no mandatory chart
- [ ] AC4: Docs-only OK (no new IPC) — no extra tests required
- [ ] AC5: No Feature rewrite; no migration; contracts unchanged
- [ ] AC6: Phase 19 companion marked shipped in glossary / status notes
- [ ] AC7: Handoff `docs/handoffs/P19-E3-T1-dev-to-qa.md`
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB stance

## Risks / not covered
- No new Menubar status IPC — operators use runbook + env + FDA checklist.
- Physical-device FDA click-through not automated.

## Notes for QA
- Kanban Done / canvas / Phase close are PM-only after QA Pass.
- Unrelated dirty PM roadmap docs may exist on the branch — out of AC.
