# PM Brief → Dev: P17-E2-T1 (Companion emit — closed)

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P17-E2** ✅  
**Evidence:** `docs/handoffs/P17-E2-T1-qa-to-pm.md` · `docs/handoffs/P17-E2-T1-dev-to-qa.md`

## Task (shipped)
**P17-E2-T1 — Expand iOS Companion / HealthKit ingest per ADR-018**

- HealthKit observer → queue → ingest for `step_count` / `active_energy` / `sleep_interval`
- Soft-optional `oxygen_saturation`; keep HR/HRV
- bio-spec validators + pipeline normalize; no migration / no charts

**Next:** **P17-E3-T1** chart ranges + Features — `docs/handoffs/P17-E3-T1-pm-brief.md`  
**Note:** Physical-device dogfood still recommended.
