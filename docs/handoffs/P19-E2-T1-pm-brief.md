# PM Brief → Dev: P19-E2-T1 (live probe — closed)

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P19-E2** ✅  
**Evidence:** `docs/handoffs/P19-E2-T1-qa-to-pm.md` · `docs/handoffs/P19-E2-T1-dev-to-qa.md`

## Task (shipped)
**P19-E2-T1 — Implement live `SystemNotificationEventProbe` mapping (ADR-020)**

- usernoted NC SQLite allowlist (`delivered_date` + `app.identifier` only)
- Soft-fail when missing / TCC denied; scripted + fixture tests
- No Feature rewrite; no migration; no content / `bundle_id` fields

**Next:** **P19-E3-T1** dogfood — `docs/handoffs/P19-E3-T1-pm-brief.md`
