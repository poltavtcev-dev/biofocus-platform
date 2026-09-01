# PM Brief → Dev: P27-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (2026-08-12) — QA Pass · **ADR-028** locked · next **P27-E2-T1**  
**Date:** 2026-08-12  
**Closed previous:** **PM-GATE-POST-P26** (chose **Pattern Discovery rule expansion**); Phase 26 OSS hygiene complete  
**Evidence:** `docs/handoffs/P27-E1-T1-qa-to-pm.md` · `docs/handoffs/P27-E1-T1-dev-to-qa.md` · `docs/handoffs/PM-GATE-POST-P26-pm-brief.md`  
**Phase:** Phase 27 Pattern Discovery rule expansion — Epic P27-E1  
**Branch:** `phase/27-pattern-rules`

## Task
**P27-E1-T1 — ADR-028: lock Pattern Discovery / Recommendations expansion scope (which rules, Feature inputs, non-goals)**

## Why
North star is Personal Pattern Discovery. v1 (ADR-008 / ADR-009) ships only three InsightRules (`high_stress_period`, `context_switch_elevated`, `focus_vs_recent_baseline_v1`) + one RecommendationRule (`focus_dip_pace_hint_v1`). Phases 10–25 added many catalog Features that Knowledge barely uses. Feature-math backlog is empty of safe distinct nodes. Gate post–P26 chose rule expansion over park-until-freeze / IDE / weather / App Store: lock which **new** deterministic rules (Insight and/or Recommendation) E2 may add — using **shipped** Features only — before coding.

## Acceptance Criteria
1. Record **ADR-028** in `docs/decision-log.md`: Phase 27 v1 primary = **Pattern Discovery rule expansion** (InsightRule and/or RecommendationRule in `knowledge-engine`). Lock: evaluate-on-read (ADR-008/009); **no** SQLite Insight/Recommendation store; personal self-tracking framing.
2. Lock a **small explicit rule slate for E2** (prefer 1–3 new rules total — name candidate rule ids + primary Feature inputs from the shipped catalog). Prefer Features that are currently under-used by Knowledge (e.g. DistractionScore, NotificationPressure, CognitiveLoad, DeepWorkScore, AttentionStability, DeskAwayPresence, CircadianOffset, SustainedLoadIndicator, RecoveryScore, MeetingDensity — pick explicitly; justify omit of others). Each rule: trigger sketch, Evidence refs, calm copy stance, omit-when-missing policy.
3. Rejected alternatives documented (new Feature catalog math; TypingRhythm; DeepFocusLikelihood; IDE; weather; App Store; Companion polish-as-primary; precise GPS; LLM-authored Insights/Recommendations; workplace surveillance framing; PR during freeze; migration / new Observation `data_type`; claiming public launch Done / flipping visibility).
4. Schema: **none** — no migration; no new Observation; **no** new Feature formulas / DAG nodes this phase.
5. Short sketch: E2 registers rules via existing `register_insights_v1` / `register_recommendations_v1` (or additive helpers); E3 dogfood + optional calm Dashboard Insights/Suggestions surface.
6. Docs touch: `12-development` / `16-glossary` / knowledge notes as “planned / ADR” — **or** ADR explicitly defers those edits to E2 (must be stated).
7. Handoff: `docs/handoffs/P27-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing rule math / registration (→ **P27-E2**)
- New collectors / Observation families / Feature catalog math
- OSS layers 2–3 (`main` catch-up / notarized public Release / visibility flip)
- IDE plugin; weather; App Store; TypingRhythm; DeepFocusLikelihood
- Opening a PR (PR freeze until 2026-09-01)
- LLM as source of Insights / Recommendations / Evidence

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `knowledge-engine` — **no** parallel Coach / Correlation crate
- Branch: `phase/27-pattern-rules`
- Personal self-tracking only — not workplace monitoring
- Calm non-clinical copy; L5 LLM interpret-only
- Obey ADR-008 / ADR-009 evaluate-on-read idle posture
- **No migration**
- **Public launch not Done** — do not claim otherwise

## After QA Pass
PM → mark P27-E1-T1 Done; Ready **P27-E2-T1** shaped by ADR-028 (ship locked rules).
