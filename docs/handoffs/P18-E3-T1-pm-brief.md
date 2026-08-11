# PM Brief → Dev: P18-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-11  
**Closed previous:** **P18-E2-T1** — notification collector plugin (QA Pass); Epic **P18-E2** ✅  
**Evidence:** `docs/handoffs/P18-E2-T1-qa-to-pm.md` · `docs/decision-log.md` ADR-019

## Task
**P18-E3-T1 — Ship catalog Feature `NotificationPressure`**

## Why
P18-E2 shipped opt-in `notification_event` Observations (`NotificationPlugin`, `BIOFOCUS_NOTIFICATION_EVENTS`, coarse `count` + optional closed-set labels; no body/title). Vision rule: Features only with real inputs. ADR-019 / catalog name **`NotificationPressure`**. Close Phase 18 with a calm, confidence-aware catalog node. Live Notification Center OS mapping may still soft-fail idle — Feature must work on scripted / fixture / HTTP-ingest Observations (same pattern as AmbientLightShare / GitActivityRate).

## Acceptance Criteria
1. Finalize **`NotificationPressure`** in `docs/06-feature-catalog.md` (remove backlog-only framing): goal, window/step (**15m / 1m**, align Focus catalog; series may coarsen), units **0–100**, inputs, formula (v1), omit policy, provenance, ADR-007 confidence, DAG registration. Calm framing only (“interruption intensity in this window” — **not** “you are overloaded” / clinical ADHD / workplace productivity scoring).
2. Inputs: `notification_event` Observations in-window. Formula v1 per ADR-019 sketch: sum `count` → intensity map **0–100**; **empty / no usable events → omit**. Optional factors from `category` / `interruption_level` / `app_kind` shares when present (P7-E2 shape; sum ≈1.0 when present).
3. Register via `feature_engine::register_notification_v1` (or helper) wired into `register_catalog_v1`; Feature appears on existing Feature Worker / `get_feature_snapshot` (+ series path when applicable) when inputs present — **no** mandatory new Dashboard chart/UI.
4. Unit tests: rich events → emit; empty → omit; confidence per ADR-007; no content keys required.
5. Docs: catalog + glossary / `12-development` Feature note as needed; mark Phase 18 Feature shipped.
6. **No** SQLite migration; **no** new ADR unless formula needs contract change (stop + propose ADR + approve). **No** live NC OS probe expansion required for Done (soft-fail idle OK).
7. Handoff: `docs/handoffs/P18-E3-T1-dev-to-qa.md`.

## Out of scope
- Live Notification Center OS mapping (may stay soft-fail `None`; Feature works on fixtures)
- `CognitiveLoad` wiring (later — needs more inputs)
- IDE / weather / App Store
- New Insights / Recommendations rules for NotificationPressure
- Dashboard redesign / dedicated notification chart
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-019 + P18-E2 contract in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Modules: `crates/feature-engine` (+ docs catalog/glossary as needed)
- Branch: `phase/18-notification-pressure`
- Prefer extend existing catalog registration — no parallel Feature registry
- LLM remains L5 interpret-only — must not invent notification scores or content
- Personal self-tracking only — not workplace / employee surveillance

## After QA Pass
PM → mark P18-E3-T1 Done; close Epic **P18-E3** and **Phase 18**; open next gate / phase per roadmap (no PR during freeze).
