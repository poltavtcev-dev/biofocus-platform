# Dev → QA: P21-E1-T1

## Meta
- **Task ID:** P21-E1-T1
- **Title:** ADR-022: lock `DeepWorkScore` Feature scope (inputs, formula stance, omit policy)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P21-E1-T1-pm-brief.md`

## What changed
- **ADR-022** in `docs/decision-log.md` (summary table + detail): Phase 21 v1 locks
  - Primary catalog Feature **`DeepWorkScore`**
  - **Feature-level** inputs: `FocusScore` (**required**) + `ContextSwitchRate` (**optional**); backlog **idle dropped** for v1 (no shipped idle Feature)
  - Formula stance for E2: window/step **15m / 1m**; `focus = FocusScore`; `stability = clamp(100 - CSR×50, 0, 100)`; weights 0.60 / 0.40; **omit** without Focus; **renormalize** when CSR absent; do **not** invent parallel FocusScore
  - ADR-007: expected slots = 2; coverage × mean(upstream Feature.confidence); optional factors `focus` / `stability`
  - Calm framing: “sustained focus in this window” — personal self-tracking only
  - Schema: **no** new Observation / **no** migration; do **not** rewrite FocusScore / CSR
  - E2 ships math in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard
  - Rejected: clinical flow/burnout/ADHD; workplace; new Observations; idle invent; leaf rewrites; IDE/weather/App Store; AttentionStability/CircadianOffset as P21 primary; PR during freeze; migration without approve; parallel engine
- Docs planned/ADR notes: `06-feature-catalog` (§1.17 stub + removed from backlog), `12-development`, `16-glossary`
- **No** Feature math implementation; **no** DAG registration; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/21-deep-work-score`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-022" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Locked inputs + omit policy + idle dropped
rg -n "FocusScore|ContextSwitchRate|idle dropped|omit without Focus|DeepWorkScore|renormalize" \
  docs/decision-log.md docs/06-feature-catalog.md

# Rejected alts
rg -n "flow state|workplace|AttentionStability|CircadianOffset|IDE|weather|App Store|PR during freeze|migration" \
  docs/decision-log.md

# No Feature code for this task
rg -n "DeepWorkScore" crates/feature-engine || true
# expect: no matches (math → P21-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-022 records Phase 21 primary = `DeepWorkScore`; Feature-level Focus+CSR; idle dropped with rationale; personal self-tracking; calm framing
- [ ] AC2: Formula stance — 15m/1m; 0–100; omit without Focus / renormalize without CSR; ADR-007 slots=2; optional factors; no parallel FocusScore
- [ ] AC3: Rejected alternatives documented (flow/burnout/ADHD; workplace; new Observations; leaf rewrites; IDE/weather/App Store; AttentionStability/CircadianOffset; PR freeze; migration)
- [ ] AC4: Schema none to apply — no migration; no new Observation `data_type`
- [ ] AC5: E2 Feature + catalog; optional E3 dogfood/Dashboard sketched
- [ ] AC6: Docs touched (`06` / `12` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P21-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB unchanged; personal self-tracking; PR freeze; no Feature impl in E1

## Risks / not covered
- Exact `register_*` helper name deferred to **P21-E2** (must still obey ADR-022).
- Weight 0.60/0.40 may be refined in E2 only within ADR bounds (Focus still required).

## Notes for QA
- Do **not** expect `feature-engine` to know `DeepWorkScore` yet — math is **P21-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty PM-GATE / Phase 20–21 open docs may exist on the branch — out of AC unless they contradict ADR-022.
