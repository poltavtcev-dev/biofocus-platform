# PM Brief → Dev: P9-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-08  
**Closed previous:** P9-E1-T1 (QA Pass — ADR-009); Epic P9-E1 ✅  
**Evidence:** `docs/handoffs/P9-E1-T1-qa-to-pm.md`

## Task
**P9-E2-T1 — Recommendations v1 engine path**

## Why
ADR-009 locked L4 to a first-class `Recommendation` + `RecommendationRule` in `knowledge-engine` (evaluate-on-read; no Recommendation SQLite; no parallel Coach Engine). Next: ship types + ≥1 deterministic rule with Evidence so Core can emit calm pace hints from existing Pattern Insights — IPC/Dashboard stays **P9-E3**.

## Acceptance Criteria
1. Domain types in `crates/bio-spec` per ADR-009: `Recommendation` at minimum `{ id, title, suggestion, category, evidence_list }`; extend `EvidenceRef` with **`Insight(InsightId)`** (or equivalent) so Recommendations can cite Knowledge.
2. In `crates/knowledge-engine`: pluggable `RecommendationRule` + registry; evaluate **after** Insights (Features + Signals + just-evaluated Insights → `Result<Vec<Recommendation>>`); empty / no-match → `Ok([])`.
3. Implement ≥1 product rule id **`focus_dip_pace_hint_v1`** per ADR-009 / `docs/09-api.md` sketch:
   - emit when pattern Insight for Focus-below-recent-baseline is present (e.g. from `focus_vs_recent_baseline_v1` / category `pattern` indicating lower Focus) **and** live `FocusScore` passes confidence gate (ADR-007)
   - Evidence includes Feature `FocusScore` + Insight id; calm non-clinical title/suggestion (pace/pause optional hint)
4. Idle / privacy: no busy-loop / always-on worker; no Recommendation SQLite; local-only; thin/low-confidence → omit.
5. Unit tests: emit + Evidence on rich synthetic inputs; omit when no matching Insight / low confidence; calm copy (no clinical claims).
6. Handoff: `docs/handoffs/P9-E2-T1-dev-to-qa.md`.

## Out of scope
- `get_recommendations` IPC / Dashboard UX (→ **P9-E3-T1** — Core evaluate API + registration is enough)
- Persisting Recommendation history / dismiss store
- LLM-authored or LLM-computed recommendations
- Plugin wave-1 (P10); AI coaching polish (P11)
- CircadianOffset / SleepDebt Features
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-009 + contracts sketch in `docs/decision-log.md` / `docs/09-api.md`
- Modules: `crates/bio-spec`, `crates/knowledge-engine` (+ host registration helpers if needed)
- Prefer evolve existing Knowledge path — **no** new Coach Engine crate
- Branch: `phase/9-recommendations`
- Calm copy only — optional personal hints, not medical advice
- LLM remains L5 interpret-only

## After QA Pass
PM → mark P9-E2-T1 Done; Ready **P9-E3-T1** (Recommendations IPC / UX).
