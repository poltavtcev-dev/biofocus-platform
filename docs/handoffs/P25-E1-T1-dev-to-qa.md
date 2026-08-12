# Dev → QA: P25-E1-T1

## Meta
- **Task ID:** P25-E1-T1
- **Title:** ADR-026: lock `SustainedLoadIndicator` Feature scope (inputs, formula stance, omit policy)
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P25-E1-T1-pm-brief.md`
- **Branch:** `phase/25-sustained-load`

## What changed
- **ADR-026** in `docs/decision-log.md` (summary table + detail): Phase 25 v1 locks
  - Primary catalog Feature **`SustainedLoadIndicator`**
  - **Feature-level** inputs: **`StressIndex` + `FatigueIndex` + `MeetingDensity`** (schedule proxy)
  - **Not** CognitiveLoad as input; **not** raw Observation mix
  - Formula stance for E2: cadence **15m / 1m** + **4h lookback**; output **0–100**; mean Stress / Fatigue / (MeetingDensity→0–100) → weighted renormalize (sketch 0.40 / 0.40 / 0.20); **omit** when both Stress and Fatigue absent; **renormalize** when MeetingDensity absent
  - ADR-007: expected slots = 3; optional factors `stress` / `fatigue` / `meeting`
  - Calm framing: “prolonged load in this window” — personal self-tracking only
  - Distinct from **CognitiveLoad** (current combined demand)
  - Schema: **no** new Observation / **no** migration; do **not** rewrite Stress / Fatigue / MeetingDensity / CognitiveLoad
  - E2 ships math in `feature-engine` + catalog §1 finalize; optional E3 dogfood / Dashboard **Prolonged load**
  - Rejected: clinical burnout / “you are burned out”; workplace surveillance; new Observations; leaf / CognitiveLoad rewrites; CognitiveLoad-as-input; meetings-alone emit; multi-day burnout profiling; IDE / weather / App Store / TypingRhythm / DeepFocusLikelihood as P25 primary; PR during freeze; migration without approve; parallel engine; LLM inventing scores
- Docs planned/ADR notes: `06-feature-catalog` (§1.21 stub + removed from backlog table), `12-development`, `16-glossary`
- **No** Feature math implementation; **no** DAG registration; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/25-sustained-load`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-026" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Locked inputs + omit + lookback + distinct sibling
rg -n "StressIndex|FatigueIndex|MeetingDensity|4h lookback|0–100|omit when both|CognitiveLoad|prolonged load" \
  docs/decision-log.md docs/06-feature-catalog.md docs/16-glossary.md

# Rejected alts
rg -n "burned out|workplace|TypingRhythm|DeepFocusLikelihood|PR during freeze|migration|IDE|weather|App Store|CognitiveLoad as" \
  docs/decision-log.md

# No Feature code for this task
rg -n "SustainedLoadIndicator" crates/feature-engine || true
# expect: no matches (math → P25-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-026 records Phase 25 primary = `SustainedLoadIndicator`; Feature-level Stress+Fatigue+MeetingDensity locked; rationale vs CognitiveLoad + deferrals; personal self-tracking; calm framing
- [ ] AC2: Formula stance — 15m/1m + 4h lookback; 0–100; omit when both Stress&Fatigue absent; renormalize Meeting optional; ADR-007 slots=3; optional factors; no Stress/Fatigue/MeetingDensity/CognitiveLoad rewrite
- [ ] AC3: Rejected alternatives documented (clinical burnout; workplace; new Observations; leaf rewrites; IDE/weather/App Store/TypingRhythm/DeepFocusLikelihood; PR freeze; migration)
- [ ] AC4: Schema none to apply — no migration; no new Observation `data_type`
- [ ] AC5: E2 Feature + catalog; optional E3 dogfood/Dashboard sketched
- [ ] AC6: Docs touched (`06` / `12` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P25-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB unchanged; personal self-tracking; PR freeze; no Feature impl in E1

## Risks / not covered
- Exact `register_*` helper name, MeetingDensity unit normalization, and weight tweaks deferred to **P25-E2** (must still obey ADR-026 omit / 0–100 / no-new-Observation / no-CognitiveLoad-input locks).
- 4h lookback may be refined in E2 only within ADR bounds (still not multi-day clinical profiling; still omit without Stress&Fatigue).

## Notes for QA
- Do **not** expect `feature-engine` to know `SustainedLoadIndicator` yet — math is **P25-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty Phase 24 / gate docs may exist on the branch — out of AC unless they contradict ADR-026.
