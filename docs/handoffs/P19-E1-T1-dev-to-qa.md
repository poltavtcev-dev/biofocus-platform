# Dev → QA: P19-E1-T1

## Meta
- **Task ID:** P19-E1-T1
- **Title:** ADR-020: privacy-safe Notification Center OS mapping + live probe boundaries
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P19-E1-T1-pm-brief.md`

## What changed
- **ADR-020** in `docs/decision-log.md` (summary table + detail): Phase 19 v1 locks
  - **Hybrid** OS stance: prefer public macOS surfaces; private/undocumented OK **only** under hard **non-content field allowlist** + soft-fail when unavailable
  - Map to existing ADR-019 `notification_event` only (`count` + optional closed-set labels) — **payload unchanged**
  - Privacy reaffirmed: never capture/persist/log body, title, subtitle, message, screenshots, attachments, userInfo (even transiently for classification); logs prefer Observation `id` / counts / closed-set labels / soft-fail reason codes
  - Capability Plugin Model: extend existing `com.biofocus.macos.notifications` / `SystemNotificationEventProbe` (mirrors ADR-014 after Git soft-fail)
  - **No** migration; existing `observations` store; opt-in `BIOFOCUS_NOTIFICATION_EVENTS` unchanged
  - Existing **`NotificationPressure`** — no formula rewrite; E2 live probe → channel → persist; optional E3 dogfood/status
  - Rejected: content capture / Accessibility scrape; workplace monitoring; busy-loop; Feature rewrite; IDE/weather/App Store/`CognitiveLoad` as P19 primary; parallel store; public-only forever; unrestricted private API; payload widening; PR during freeze; migration without approve
- Docs planned/ADR notes: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`
- **No** live probe implementation; **no** Feature math change; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/19-live-nc-mapping`

## How to verify (commands)
```bash
# ADR + contracts present
rg -n "ADR-020" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/10-security.md docs/12-development.md docs/16-glossary.md

# Hybrid stance + privacy allowlist + soft-fail
rg -n "hybrid|field allowlist|soft-fail|SystemNotificationEventProbe" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md

# Rejected alts
rg -n "Content capture|Accessibility|workplace|NotificationPressure formula|CognitiveLoad|PR during freeze|migration" docs/decision-log.md

# No live probe / Feature rewrite / migration for this task
rg -n "ADR-020|system_notification_event" crates/macos-collector/src/notification_probe.rs
# expect: still soft-fail Ok(None) — no live mapping code yet
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-020 records one v1 OS/API stance (hybrid) + rationale vs ADR-019 soft-fail + Capability Model + personal self-tracking
- [ ] AC2: Privacy contract reaffirmed (no body/title/…; logs prefer id/counts/labels)
- [ ] AC3: Rejected alternatives documented (content, workplace, busy-loop, Feature rewrite, IDE/weather/App Store/CognitiveLoad, parallel store, PR freeze, migration)
- [ ] AC4: Schema none to apply — ADR-019 payload unchanged; no migration
- [ ] AC5: E2 live probe → existing channel → NotificationPressure; optional E3 dogfood/status sketched
- [ ] AC6: Docs touched (07/08/10/12/16) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P19-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB; personal self-tracking; PR freeze; no live impl in E1

## Risks / not covered
- Exact OS surface (which private DB/API) is deferred to **P19-E2** — must still obey ADR-020 field allowlist.
- Public API surface may remain insufficient; soft-fail after best-effort is explicitly OK.

## Notes for QA
- Do **not** expect `SystemNotificationEventProbe` to emit yet — live mapping is **P19-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty PM-GATE / Phase 19 open docs may exist on the branch — out of AC unless they contradict ADR-020.
