# PM Brief → Dev: P8-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-07  
**Closed previous:** P8-E1-T1 (QA Pass — ADR-008 recompute-on-read); Epic P8-E1 ✅  
**Evidence:** `docs/handoffs/P8-E1-T1-qa-to-pm.md`

## Task
**P8-E2-T1 — Pattern Discovery v1 Insight path**

## Why
ADR-008 locked Pattern Discovery v1 to **recompute-on-read** (no Feature-history SQLite table, no always-on worker). Next: ship ≥1 calm baseline Insight in `knowledge-engine` that compares a live Feature to a bounded recomputed series from local Observations.

## Acceptance Criteria
1. Implement ≥1 baseline / multi-window Insight rule in `crates/knowledge-engine` per ADR-008 — prefer rule id **`focus_vs_recent_baseline_v1`**:
   - `current` = `FocusScore` from live Feature snapshot (confidence ≥ gate, ADR-007)
   - `series` = recompute `FocusScore` for N prior comparable windows (N ≤ 7; same time-of-day bucket or daily afternoon mean)
   - emit when `|current - baseline| ≥ δ` and both sides pass confidence; Evidence refs Feature/Signal ids
2. Wire Observation load + bounded Feature recompute for the rule path (evolve existing Core / `feature-engine` helpers as needed — **no** new parallel Correlation Engine crate; **no** Feature-history migration).
3. Idle / privacy: no busy-loop / always-on recompute worker; thin/empty history → omit Insight (`Ok([])`); all local-only.
4. Optional in-process memo (TTL / invalidate on Observation watermark) allowed — not SQLite.
5. Unit tests: thin history (no emit); rich synthetic series (emit + Evidence); calm non-clinical copy.
6. Handoff: `docs/handoffs/P8-E2-T1-dev-to-qa.md`.

## Out of scope
- Feature-history / baseline SQLite tables or migrations
- Always-on recompute worker
- LLM pattern generation
- CircadianOffset / SleepDebt Features
- Recommendations (Phase 9)
- Dashboard / Insights IPC UX polish (→ **P8-E3-T1** — Core rule path is enough if existing `get_insights` already evaluates registered rules)
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-008 + contracts sketch in `docs/09-api.md` / `docs/decision-log.md`
- Modules: `crates/knowledge-engine` (+ Observation load / Feature recompute as required)
- Branch: `phase/8-pattern-discovery` (or tip user is on)
- Calm copy only — personal observation, not clinical diagnosis

## After QA Pass
PM → mark P8-E2-T1 Done; Ready **P8-E3-T1** (Insights IPC / UX for patterns).
