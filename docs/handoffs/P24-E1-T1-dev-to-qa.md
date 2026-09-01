# Dev → QA: P24-E1-T1

## Meta
- **Task ID:** P24-E1-T1
- **Title:** ADR-025: lock `CircadianOffset` Feature scope (inputs, formula stance, omit policy)
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P24-E1-T1-pm-brief.md`
- **Branch:** `phase/24-circadian-offset`

## What changed
- **ADR-025** in `docs/decision-log.md` (summary table + detail): Phase 24 v1 locks
  - Primary catalog Feature **`CircadianOffset`**
  - **Observation-level timing** inputs: `sleep_interval` (sleep midpoint) + work/activity timing from `keystrokes` / `context_window` (preferred), optional reinforcement `step_count` / `active_energy` / workout `life_event`
  - **Not** Feature-level SleepDebt / EnergyScore / ActivityBalance (magnitude ≠ timing)
  - Formula stance for E2: cadence **15m / 1m** + **24h lookback**; output **0–100** alignment (not signed chronotype hours); sleep_mid + work_mid → circular offset from `sleep_mid + 12h` → map with ~6h saturation; **omit** unless both sleep + work/activity timing present (no single-slot renormalize)
  - ADR-007: expected slots = 2; optional factors `sleep_timing` / `work_timing` (+ optional `activity_timing`)
  - Calm framing: “schedule alignment in this window” — personal self-tracking only
  - Distinct from **SleepDebt** (debt) and **DeskAwayPresence** (away-from-desk)
  - Schema: **no** new Observation / **no** migration; do **not** rewrite SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence
  - E2 ships math in `feature-engine` + catalog §1 finalize; optional E3 dogfood / Dashboard **Schedule alignment**
  - Rejected: clinical chronotype / circadian-disorder / “night owl so you fail”; workplace schedule surveillance; new Observations; leaf rewrites; Feature-level magnitude proxies; signed hours as primary units; single-slot emit; IDE / weather / App Store / TypingRhythm as P24 primary; precise GPS; PR during freeze; migration without approve; parallel engine; LLM inventing scores
- Docs planned/ADR notes: `06-feature-catalog` (§1.20 stub + removed from backlog), `12-development`, `16-glossary`
- **No** Feature math implementation; **no** DAG registration; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/24-circadian-offset`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-025" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Locked inputs + omit policy + units + distinct siblings
rg -n "sleep_interval|keystrokes|context_window|24h lookback|0–100|omit unless both|SleepDebt|DeskAwayPresence|schedule alignment" \
  docs/decision-log.md docs/06-feature-catalog.md docs/16-glossary.md

# Rejected alts
rg -n "chronotype|night owl|workplace schedule|signed|TypingRhythm|precise GPS|PR during freeze|migration|IDE|weather|App Store" \
  docs/decision-log.md

# No Feature code for this task
rg -n "CircadianOffset" crates/feature-engine || true
# expect: no matches (math → P24-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-025 records Phase 24 primary = `CircadianOffset`; Observation-level sleep + work/activity timing locked; rationale vs Personal Context + deferrals; personal self-tracking; calm framing
- [ ] AC2: Formula stance — 15m/1m + 24h lookback; 0–100 (not signed hours); omit unless both slots; ADR-007 slots=2; optional factors; no SleepDebt/EnergyScore/FocusScore rewrite
- [ ] AC3: Rejected alternatives documented (clinical chronotype; workplace; new Observations; leaf rewrites; IDE/weather/App Store/TypingRhythm; GPS; PR freeze; migration)
- [ ] AC4: Schema none to apply — no migration; no new Observation `data_type`
- [ ] AC5: E2 Feature + catalog; optional E3 dogfood/Dashboard sketched
- [ ] AC6: Docs touched (`06` / `12` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P24-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB unchanged; personal self-tracking; PR freeze; no Feature impl in E1

## Risks / not covered
- Exact `register_*` helper name, time-weighting of work centroid, and activity-reinforcement thresholds deferred to **P24-E2** (must still obey ADR-025 omit / 0–100 / no-new-Observation locks).
- 6h saturation and `sleep_mid + 12h` heuristic may be refined in E2 only within ADR bounds (still 0–100; still omit without both slots; still non-clinical).

## Notes for QA
- Do **not** expect `feature-engine` to know `CircadianOffset` yet — math is **P24-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty Phase 23 / gate docs may exist on the branch — out of AC unless they contradict ADR-025.
