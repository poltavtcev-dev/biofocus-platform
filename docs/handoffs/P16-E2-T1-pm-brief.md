# PM Brief → Dev: P16-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** P16-E1-T1 (QA Pass — ambient light plugin); Epic **P16-E1** ✅  
**Evidence:** `docs/handoffs/P16-E1-T1-qa-to-pm.md`

## Task
**P16-E2-T1 — Catalog Feature `AmbientLightShare`**

## Why
P16-E1 shipped opt-in `ambient_light` Observations (`AmbientLightPlugin`, `BIOFOCUS_AMBIENT_LIGHT`, coarse `light_kind` + optional `level` 0–100). Vision rule: Features only with real inputs. Catalog §1.11 sketches **`AmbientLightShare`**. Close Phase 16 with a calm, confidence-aware catalog node. Live OS brightness may still soft-fail idle — Feature must work on scripted / fixture / HTTP-ingest Observations (same pattern as AmbientMediaShare / GitActivityRate).

## Acceptance Criteria
1. Finalize **`AmbientLightShare`** in `docs/06-feature-catalog.md` §1.11 (remove “planned/sketch”): goal, window/step (**15m / 1m**, align Focus/CSR / AmbientMediaShare), units, inputs, formula (v1), omit policy, provenance, ADR-007 confidence, DAG registration. Calm framing only (“light context during this window” — **not** clinical lighting advice / “bad light harms you”).
2. Inputs: `ambient_light` Observations (`light_kind` + optional `level`) with ≥1 **closed-set** kind: `dark` / `dim` / `moderate` / `bright`. Formula v1 (mirror AmbientMediaShare sample-share stance unless catalog already locks better): among samples in-window, value = **0–100** share of closed-set samples (or documented band-share variant); **only-`unknown` / empty** → **omit**. Optional `level` may refine factors/confidence only if still coarse (no lux / camera).
3. Register via `feature_engine::register_ambient_light_v1` (or helper) wired into `register_catalog_v1`; Feature appears on existing Feature Worker / `get_feature_snapshot` path when inputs present — **no** mandatory new Dashboard chart/UI.
4. Optional: `ExplanationFactor`s for closed-set band shares (P7-E2 shape; sum 1.0 when present).
5. Unit tests: rich closed-set light → emit; empty / only-`unknown` → omit; confidence per ADR-007.
6. Docs: catalog + glossary / `12-development` Feature note as needed; mark Phase 16 Feature shipped.
7. **No** SQLite migration; **no** new ADR unless formula needs contract change (stop + propose ADR + approve). **No** Phase 17 wearable/chart work (ADR-017 parked).
8. Handoff: `docs/handoffs/P16-E2-T1-dev-to-qa.md`.

## Out of scope
- Live OS brightness mapping (may stay soft-fail `None`; Feature works on fixtures)
- Weather ambient; IDE; App Store; NotificationPressure
- Phase 17 Mi/HealthKit depth or chart range picker (ADR-017)
- New Insights / Recommendations rules for AmbientLightShare
- Dashboard redesign / dedicated light chart
- New SQLite schema / ambient registry
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-015 + P16-E1 contract in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Modules: `crates/feature-engine` (+ docs catalog/glossary as needed)
- Branch: `phase/16-ambient-light`
- Prefer extend existing catalog registration — no parallel Feature registry
- LLM remains L5 interpret-only — must not invent light scores or kinds
- Personal self-tracking only — not workplace / environmental surveillance

## After QA Pass
PM → mark P16-E2-T1 Done; close Epic **P16-E2** and **Phase 16** if no further P16 tasks; next = Phase 17 shaping (ADR-017 parked intent → contract ADR) **without** opening a PR during freeze.
