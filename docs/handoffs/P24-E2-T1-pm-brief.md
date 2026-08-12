# PM Brief → Dev: P24-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · Feature shipped · next **P24-E3-T1**  
**Date:** 2026-08-12  
**Closed previous:** P24-E1-T1 (**ADR-025** locked; QA Pass)  
**Evidence:** `docs/handoffs/P24-E1-T1-qa-to-pm.md` · `docs/handoffs/P24-E1-T1-dev-to-qa.md`  
**Phase:** Phase 24 CircadianOffset — Epic P24-E2  
**Branch:** `phase/24-circadian-offset`

## Task
**P24-E2-T1 — Ship catalog Feature `CircadianOffset` per ADR-025**

## Why
ADR-025 locked **Observation-level timing** alignment from **sleep timing** (`sleep_interval`) vs **work/activity timing** (`keystrokes` / `context_window`, optional steps / active energy / workout) — distinct from SleepDebt magnitude and DeskAwayPresence. E1 docs/catalog stub §1.20 are in place; E2 implements math + DAG registration so Snapshot/IPC can emit the Feature. No new Observation family; no migration; leaf Feature formulas stay untouched.

## Acceptance Criteria
1. Implement **`CircadianOffset`** in `feature-engine` per ADR-025: Observation-level timing only; Feature cadence **15m / 1m**; timing math **24h lookback** ending at window end; output **0–100**.
2. Composition (ADR-025):  
   - Sleep timing from qualifying `sleep_interval` (same rest stages as SleepDebt: `asleep` / `in_bed` / missing stage; `awake` / `unknown` do not contribute) → sleep midpoint.  
   - Work/activity timing from `keystrokes` / `context_window` (preferred) with optional reinforcement `step_count` / `active_energy` / workout `life_event` → work/activity centroid.  
   - Map circular hours from expected mid-wake (`sleep_mid + 12h`) to alignment: prefer `100 × (1 − offset_hours / 6)` clamped to 0–100 (exact helper / weighting OK if ADR-025 intent preserved).  
   - **Omit** unless **both** sleep timing and work/activity timing present — **do not** renormalize a single slot; **do not** emit signed chronotype hours as primary units.  
   - Do **not** invent Feature-level SleepDebt×ActivityBalance “alignment”; do **not** use SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence magnitudes as timing proxies.
3. **ADR-007** confidence: `expected_slots = 2` (sleep_timing / work_activity_timing); emit only when both present; follow existing confidence patterns.
4. Optional explanation factors for present components (`sleep_timing` / `work_timing` / optional `activity_timing`) with calm labels — **not** chronotype / “night owl” / sleep-disorder copy.
5. Register in catalog / DAG (follow existing Observation-timing Feature patterns; exact `register_*` helper name chosen in E2). Finalize `docs/06-feature-catalog.md` §1.20 (stub → shipped formula).
6. Unit tests: both slots emit · omit without sleep · omit without work/activity · confidence · factors when emitted · no Feature-level magnitude proxy path.
7. Docs: `12-development` / `16-glossary` ship notes for CircadianOffset.
8. **Do not** rewrite SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence formulas; **no** new Observation `data_type`; **no** migration; **no** UI/Dashboard surface (→ E3 optional); **no** parallel engine crate.
9. Handoff: `docs/handoffs/P24-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard calm surface (→ **P24-E3**)
- IDE plugin; weather ambient; App Store packaging; TypingRhythm
- Workplace / manager dashboards; clinical chronotype / sleep-disorder diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)
- New collectors / Observation families

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. circadian DoD)
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/24-circadian-offset`
- Personal self-tracking only — not workplace monitoring
- Copy: “schedule alignment in this window” — **not** chronotype / circadian-disorder / “night owl so you fail”
- Distinct from SleepDebt (“sleep debt”) and DeskAwayPresence (“away from desk”)
- LLM remains L5 interpret-only
- **No migration**
- Important: omit-unless-both-slots is locked — single-slot must not emit CircadianOffset

## After QA Pass
PM → mark P24-E2-T1 Done; Ready **P24-E3-T1** (dogfood / optional Dashboard).
