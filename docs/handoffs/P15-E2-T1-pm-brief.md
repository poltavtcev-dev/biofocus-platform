# PM Brief → Dev: P15-E2-T1 (companion — closed)

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P15-E2** ✅  
**Evidence:** `docs/handoffs/P15-E2-T1-qa-to-pm.md` · `docs/handoffs/P15-E2-T1-dev-to-qa.md`

## Task (shipped)
**P15-E2-T1 — Core SDNN-or-RMSSD + iOS Auto-sync (ADR-016)**

- `pipeline::normalize_hrv` accepts `rmssd_ms` OR `sdnn_ms`
- Focus/Recovery prefer rmssd then sdnn
- iOS HRV + ObservationQueue + HKObserver background delivery + Auto-sync wiring

**Next:** **P15-E3-T1** dogfood + Companion UI — `docs/handoffs/P15-E3-T1-pm-brief.md`
