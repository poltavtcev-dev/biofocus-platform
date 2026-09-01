# PM Brief → Dev: P27-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · three ADR-028 rules shipped · next **P27-E3-T1**  
**Date:** 2026-08-12  
**Closed previous:** **P27-E1-T1** (**ADR-028** locked; QA Pass)  
**Evidence:** `docs/handoffs/P27-E2-T1-qa-to-pm.md` · `docs/handoffs/P27-E2-T1-dev-to-qa.md`  
**Phase:** Phase 27 Pattern Discovery rule expansion — Epic P27-E2  
**Branch:** `phase/27-pattern-rules`

## Task
**P27-E2-T1 — Ship locked Insight/Recommendation rules in `knowledge-engine` per ADR-028**

## Why
**ADR-028** locked a small evaluate-on-read expansion: two InsightRules + one RecommendationRule on **already shipped** Features (`CognitiveLoad`, `SustainedLoadIndicator`). v1 Knowledge still only uses High_Stress / CSR / Focus baseline + focus pace hint. E2 implements and registers **exactly** the locked slate so Snapshot evaluate-on-read can emit calm demand / prolonged-load Insights and a matching pace Recommendation — without Feature math, migration, or a parallel Coach crate.

## Acceptance Criteria
1. Implement + register **exactly** these three rules via existing `register_insights_v1` / `register_recommendations_v1` (or thin additive helpers called from those) in `knowledge-engine`:
   - InsightRule **`cognitive_load_elevated_v1`** — required Feature **`CognitiveLoad`**; trigger latest scalar ≥ **60** (tune ±10 within calm non-spam bounds OK); omit when absent / non-scalar / below threshold; optional Evidence `MeetingDensity` / `ContextSwitchRate` / `NotificationPressure` when present; calm copy: “Combined demand looked elevated in this window” — **not** overload / burnout / workplace scoring.
   - InsightRule **`sustained_load_elevated_v1`** — required Feature **`SustainedLoadIndicator`**; trigger ≥ **60** (±10 tune OK); omit when absent / non-scalar / below threshold; optional Evidence `StressIndex` / `FatigueIndex` / `MeetingDensity`; calm copy: “Prolonged load looked elevated in this window” — **not** “you are burned out”; keep distinct from `high_stress_period_v1` and `cognitive_load_elevated_v1`.
   - RecommendationRule **`combined_demand_pace_hint_v1`** — required Insight **`cognitive_load_elevated_v1`** (ADR-009 style, same pattern as `focus_dip_pace_hint_v1`) + optional latest `CognitiveLoad`; emit when that Insight is present; omit when absent; gentle pace / pause hint — **not** clinical / employer coaching; Evidence = Insight id (+ CognitiveLoad Feature id when present).
2. **Do not** add any rule outside this locked slate without ADR amend + user approve.
3. **Do not** rewrite existing v1 rules (`high_stress_period_v1`, `context_switch_elevated_v1`, `focus_vs_recent_baseline_v1`, `focus_dip_pace_hint_v1`) beyond necessary shared helper wiring.
4. Unit tests cover: emit / omit / Evidence / calm copy stance / threshold band; Recommendation emits only when cognitive-load Insight present; no Feature formula rewrite path.
5. Schema / product: **none** — no migration; no new Observation `data_type`; **no** new Feature catalog math / DAG nodes; **no** Insight/Recommendation SQLite store.
6. Docs: update `12-development` / `16-glossary` (and knowledge notes if present) from “planned / ADR” → E2 ship notes for the three rule ids.
7. Handoff: `docs/handoffs/P27-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard Insights·Suggestions surface polish (→ **P27-E3**)
- New Feature catalog math / Observation families
- Rules on DistractionScore / DeepWorkScore / AttentionStability / DeskAwayPresence / CircadianOffset / RecoveryScore (omitted by ADR-028)
- OSS layers 2–3; App Store; IDE; weather; TypingRhythm; DeepFocusLikelihood
- Opening a PR (PR freeze until 2026-09-01)
- LLM-authored Insights / Recommendations / Evidence
- Parallel Coach / Correlation Engine crate

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. Pattern rules DoD)
- Obey **ADR-028** locks — do not reopen rejected alts
- Prefer extend `knowledge-engine` — evaluate-on-read idle posture (ADR-008 / ADR-009)
- Branch: `phase/27-pattern-rules`
- Personal self-tracking only — not workplace monitoring
- Calm non-clinical copy; L5 LLM interpret-only
- **No migration**
- **Public launch not Done** — do not claim otherwise

## After QA Pass
PM → mark P27-E2-T1 Done; Ready **P27-E3-T1** (dogfood + optional calm Insights/Suggestions surface).
