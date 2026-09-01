# Dev → QA: P20-E1-T1

## Meta
- **Task ID:** P20-E1-T1
- **Title:** ADR-021: lock `CognitiveLoad` Feature scope (inputs, formula stance, omit policy)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P20-E1-T1-pm-brief.md`

## What changed
- **ADR-021** in `docs/decision-log.md` (summary table + detail): Phase 20 v1 locks
  - Primary catalog Feature **`CognitiveLoad`**
  - **Feature-level** inputs: `MeetingDensity` + `ContextSwitchRate` + `NotificationPressure` (not Observation-level mix)
  - Formula stance for E2: window/step **15m / 1m**; normalize to 0–100; equal thirds; **renormalize** when ≥1 present / **omit** when none (locked — not omit-unless-all-three)
  - ADR-007: expected slots = 3; coverage × mean(upstream Feature.confidence); optional explanation factors
  - Calm framing: “combined demand in this window” — personal self-tracking only
  - Schema: **no** new Observation / **no** migration; do **not** rewrite leaf Feature formulas
  - E2 ships math in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard
  - Rejected: clinical overload/burnout; workplace surveillance; new Observation families; rewriting leaf Features; Observation-mix as v1; IDE/weather/App Store as P20 primary; PR during freeze; migration without approve; parallel Load Engine; LLM inventing scores
- Docs planned/ADR notes: `06-feature-catalog` (§1.16 stub + removed from backlog), `12-development`, `16-glossary`
- **No** Feature math implementation; **no** DAG registration; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/20-cognitive-load`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-021" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Locked inputs + omit policy + window
rg -n "MeetingDensity|ContextSwitchRate|NotificationPressure|renormalize|15m|CognitiveLoad" docs/decision-log.md docs/06-feature-catalog.md

# Rejected alts
rg -n "cognitive overload|workplace|Observation-level|IDE|weather|App Store|PR during freeze|migration" docs/decision-log.md

# No Feature code / migration for this task
rg -n "CognitiveLoad" crates/feature-engine || true
# expect: no matches (math → P20-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-021 records Phase 20 primary = `CognitiveLoad`; Feature-level inputs locked; rationale vs deferrals; personal self-tracking; calm non-clinical framing
- [ ] AC2: Formula stance sketch — 15m/1m; 0–100; missing-input policy = renormalize (≥1) / omit (none); ADR-007 slots=3; optional factors
- [ ] AC3: Rejected alternatives documented (clinical overload/burnout; workplace; new Observations; leaf rewrites; IDE/weather/App Store; PR freeze; migration)
- [ ] AC4: Schema none to apply — no migration; no new Observation `data_type`
- [ ] AC5: E2 ships Feature + catalog; optional E3 dogfood/Dashboard sketched
- [ ] AC6: Docs touched (`06` / `12` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P20-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB unchanged; personal self-tracking; PR freeze; no Feature impl in E1

## Risks / not covered
- Exact `register_*` helper name and provenance wiring details deferred to **P20-E2** (must still obey ADR-021).
- Weight tuning (⅓ each) may be refined in E2 only within ADR bounds (still Feature-level + renormalize-partial).

## Notes for QA
- Do **not** expect `feature-engine` to know `CognitiveLoad` yet — math is **P20-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty Phase 19 / PM-GATE docs may exist on the branch — out of AC unless they contradict ADR-021.
