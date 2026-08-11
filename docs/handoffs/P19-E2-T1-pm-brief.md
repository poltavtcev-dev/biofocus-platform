# PM Brief → Dev: P19-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** P19-E1-T1 (QA Pass — ADR-020); Epic **P19-E1** ✅  
**Evidence:** `docs/handoffs/P19-E1-T1-qa-to-pm.md`

## Task
**P19-E2-T1 — Implement live `SystemNotificationEventProbe` mapping (ADR-020)**

## Why
ADR-020 locked Phase 19 v1 stance to **hybrid** privacy-safe OS mapping (prefer public surfaces; private/undocumented OK only under a hard **non-content field allowlist**) with honest soft-fail when mapping unavailable. ADR-019 `notification_event` payload stays unchanged. Next: wire live `SystemNotificationEventProbe` so opt-in `BIOFOCUS_NOTIFICATION_EVENTS` can emit real Observations → existing **`NotificationPressure`** (no Feature rewrite).

## Acceptance Criteria
1. Implement live mapping behind existing `NotificationPlugin` / `SystemNotificationEventProbe` per ADR-020: emit ADR-019 Observations (`count` + optional closed-set `category` / `interruption_level` / `app_kind` only). **Never** read, log, or persist body / title / subtitle / message / screenshots / attachments / userInfo dumps — even transiently for classification.
2. Soft-fail idle (`Ok(None)` / no emit) when OS mapping unavailable or would require forbidden fields; remain idle-safe (on delivery / coalesced change or rare ≥5s poll; **no** busy-loop); `stop_stream` joins background work.
3. Desktop `ingest_host` still starts the plugin **only** when `BIOFOCUS_NOTIFICATION_EVENTS=1`; same bounded Observation channel → validate/normalize → persist; pipeline continues to strip content keys if present.
4. Keep `ScriptedNotificationEventProbe` (or equivalent) for tests; cover: soft-fail / unavailable path → no emit; scripted/fixture emit → channel → persist where practical; after `stop_stream`, emission freezes; forbidden content keys still rejected/stripped.
5. Docs finalized as shipped (not “planned only”) for the live probe path: `07-contracts` / `08-plugin-sdk` / `10-security` / `12-development` as needed. Exact OS surface chosen in E2 must be named and still obey ADR-020 field allowlist.
6. **No** Feature formula rewrite for `NotificationPressure`; **no** SQLite migration / parallel notification store; **no** payload widening (`bundle_id` / display names as Observation fields).
7. Handoff: `docs/handoffs/P19-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood runbook / calm mapping-unavailable status UX (→ **P19-E3-T1**)
- Rewriting `NotificationPressure` math
- IDE collector; weather ambient; App Store packaging; `CognitiveLoad`
- Workplace / manager dashboards; Accessibility banner scrape
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-020 + ADR-019 contracts in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Prefer extend `macos-collector` + host — no parallel marketplace crate
- Branch: `phase/19-live-nc-mapping`
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only
- **No schema approve expected**

## After QA Pass
PM → mark P19-E2-T1 Done; Ready **P19-E3-T1** (dogfood runbook / optional calm status).
