# PM Brief → Dev|UX: P27-E3-T1

**From:** PM  
**To:** Dev (+ UX if UI)  
**Status:** Ready  
**Date:** 2026-08-12  
**Closed previous:** **P27-E2-T1** (locked ADR-028 rules shipped in `knowledge-engine`; QA Pass)  
**Evidence:** `docs/handoffs/P27-E2-T1-qa-to-pm.md` · `docs/handoffs/P27-E2-T1-dev-to-qa.md`  
**Phase:** Phase 27 Pattern Discovery rule expansion — Epic P27-E3  
**Branch:** `phase/27-pattern-rules`

## Task
**P27-E3-T1 — Dogfood notes + optional calm Insights/Suggestions surface for ADR-028 rules**

## Why
E2 shipped **`cognitive_load_elevated_v1`**, **`sustained_load_elevated_v1`**, and **`combined_demand_pace_hint_v1`** via existing `register_*_v1` helpers — host evaluate-on-read already picks them up (no new IPC required for emit). Phase 27 closes with operator dogfood notes and optional calm Dashboard presentation for the new categories (`demand` / `prolonged_load` / pace Recommendation) — **not** new rule math, **not** Feature catalog work, **not** clinical / workplace framing.

## Acceptance Criteria
1. Dogfood notes in `docs/12-development.md` (or adjacent §): how to verify the three rules — fixture / path that yields elevated `CognitiveLoad` and/or `SustainedLoadIndicator` (≥ 60) → Insights; `combined_demand_pace_hint_v1` only when cognitive-load Insight present; omit when required Feature / Insight absent; evaluate-on-read idle posture.
2. Calm framing only in docs/UI chrome: “combined demand looked elevated” / “prolonged load looked elevated” / gentle pace hint — **not** overload / burnout / “you are burned out” / workplace scoring / clinical advice. Keep SustainedLoad distinct from High_Stress and CognitiveLoad Insights.
3. **Optional** calm Dashboard Insights / Suggestions surface: if existing Insights/Suggestions path already lists evaluate-on-read results, ensure the new categories are readable with calm labels when present (raw `demand` / `prolonged_load` may need light label polish); empty/omit stays quiet. Prefer reuse of existing IPC (`get_insights` / `get_recommendations` or current Dashboard poll) over redesign. Docs-only dogfood **is enough** if surface already works without gaps — state which path in handoff.
4. Smoke notes in handoff: commands / steps to see emit (unit/fixture and/or app path); confirm UI ↛ SQLite; confirm no new Feature math / migration.
5. **Do not** rewrite rule triggers / Feature formulas; **do not** add rules outside ADR-028 slate; **no** migration; **no** Insight/Recommendation SQLite store; **no** parallel Coach crate.
6. Docs touch: `12-development` / `16-glossary` reflect E3 dogfood (ADR-028 remains locked; E2 ship notes stay).
7. Handoff: `docs/handoffs/P27-E3-T1-dev-to-qa.md`.

## Out of scope
- New Insight/Recommendation rules or threshold redesign beyond calm copy/labels
- New Feature catalog math / Observation families
- OSS layers 2–3; App Store; IDE; weather; TypingRhythm; DeepFocusLikelihood
- Opening a PR (PR freeze until 2026-09-01)
- LLM-authored Insights / Recommendations / Evidence
- Claiming public launch Done

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. Pattern rules DoD)
- Obey **ADR-028** — evaluate-on-read; personal self-tracking only
- Modules: docs (+ optional Dashboard Insights/Suggestions presentation / `docs/09-api.md` if label gap)
- Branch: `phase/27-pattern-rules`
- Prefer thin E3 — Phase 27 rule math is Done
- LLM remains L5 interpret-only
- **No migration**
- **Public launch not Done**

## After QA Pass
PM → mark P27-E3-T1 Done; close Epic **P27-E3** and **Phase 27** if no further P27 tasks; next = **PM-GATE-POST-P27** (or park) **without** claiming public launch Done / without opening a PR during freeze.
