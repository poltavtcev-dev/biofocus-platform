# PM Brief → Dev: P11-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P11-E1-T1 (QA Pass — ADR-011); Epic **P11-E1** ✅  
**Evidence:** `docs/handoffs/P11-E1-T1-qa-to-pm.md`

## Task
**P11-E2-T1 — Versioned prompt packs in `report-engine`**

## Why
ADR-011 locked L5 polish to **named/versioned prompt packs** in `report-engine` (templates over already-computed Features / Insights / Recommendations) plus later provider UX. Next: ship the in-process pack API + default calm pack so E3 can wire Dashboard without inventing prompt strings in UI. LLM stays interpret-only; packs must not compute Features.

## Acceptance Criteria
1. Public Core API in `crates/report-engine` per ADR-011: select/build a pack by **`id` + `version`** that produces offline `markdown` + `llm_prompt` from already-computed Features / Insights / Recommendations — return `Result`; **no** Feature / Recommendation math inside the pack builder.
2. Ship ≥1 default pack (e.g. `biofocus.default` / `1`): calm / non-clinical; `llm_prompt` instructions explicitly forbid inventing metrics, Evidence, Insights, Recommendations, or actions.
3. Empty / partial Evidence → soft empty sections, `Ok`, no panic; unit tests: pack selection + empty/partial inputs + default pack present.
4. No SQLite schema; no network from pack builder; no Desktop UI in this task (host may keep using existing `build_report` until E3 — wire pack into Core path as needed without UI).
5. Docs: finalize pack API in `docs/09-api.md`; update glossary / security notes if “planned” → shipped.
6. Handoff: `docs/handoffs/P11-E2-T1-dev-to-qa.md`.

## Out of scope
- Dashboard provider status / pack picker UX (→ **P11-E3-T1**)
- Cloud LLM providers / marketplace; auto-invoke on open
- On-disk user-editable pack overrides (future ADR + approve)
- Chat history / coaching transcript SQLite
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-011 + Phase 4 `build_report` / `interpret_report` / `generate_report` baseline
- Prefer evolve `report-engine` — **no** parallel Coach Engine crate
- Branch: `phase/11-ai-coaching-polish`
- Calm copy only — optional personal interpretation, not medical advice
- LLM remains L5 interpret-only

## After QA Pass
PM → mark P11-E2-T1 Done; Ready **P11-E3-T1** (provider UX + pack-aware Report flow).
