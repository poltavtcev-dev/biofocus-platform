# PM Brief → Dev: P7-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-06  
**Closed previous:** P7-E1-T1 (QA Pass — ADR-007 Feature confidence + domain/IPC wire); Epic P7-E1 ✅  
**Evidence:** `docs/handoffs/P7-E1-T1-qa-to-pm.md`

## Task
**P7-E2-T1 — Explanation factors on Features**

## Why
Confidence says *how much* to trust a Feature; users and Insights still need a calm *why this value* breakdown (weights / contributions) grounded in provenance — not LLM prose and not clinical diagnosis tone.

## Acceptance Criteria
1. Document factor shape (stable id or label + contribution weight/share; calm naming) in domain and/or contracts + catalog note. Use an ADR only if the domain/IPC boundary grows beyond an additive optional field (follow ADR-007 style).
2. At least one catalog Feature (prefer `FocusScore` or `StressIndex`) **emits** factors with value + confidence + provenance.
3. `get_feature_snapshot` (or Feature JSON) exposes factors, or documents an explicit omit-until-present policy; UI ↛ SQLite; no new Feature SQLite schema without ADR + approve.
4. Unit tests cover factor emission / documented sum-or-share policy; empty/thin windows stay idle-safe (omit Feature or empty factors per policy).
5. Handoff: `docs/handoffs/P7-E2-T1-dev-to-qa.md`.

## Out of scope
- LLM-generated explanations
- Pattern Discovery / baselines (Phase 8)
- New bio Features (→ **P7-E3-T1**)
- Full Dashboard “Why?” UX redesign (optional read-only display OK, not required)
- Action / automation framework

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Build on ADR-007 confidence + existing `FeatureNode` path — **no** parallel Explainability Engine crate
- Branch: `phase/7-trust-layer`
- Calm copy: factors describe inputs/weights, not burnout/clinical claims

## After QA Pass
PM → mark P7-E2-T1 Done; Ready **P7-E3-T1** (first bio-backed Trust Features) unless sequencing note says otherwise.
