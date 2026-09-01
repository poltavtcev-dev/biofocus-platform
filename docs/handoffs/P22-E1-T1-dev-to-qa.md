# Dev → QA: P22-E1-T1

## Meta
- **Task ID:** P22-E1-T1
- **Title:** ADR-023: lock `AttentionStability` Feature scope (inputs, formula stance, omit policy)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P22-E1-T1-pm-brief.md`

## What changed
- **ADR-023** in `docs/decision-log.md` (summary table + detail): Phase 22 v1 locks
  - Primary catalog Feature **`AttentionStability`**
  - **Feature-level** inputs: `FocusScore` (**required**) + `ContextSwitchRate` (**optional**)
  - **Distinct from DeepWorkScore:** Focus **range**/consistency + CSR switch stability — **not** Focus-level intensity math
  - Formula stance for E2: window/step **15m / 1m**; `focus_stability` from in-window Focus range (`100 − (max−min)` when ≥2 samples; `100` when exactly one); `switch_stability = clamp(100 − CSR×50, 0, 100)`; weights 0.50 / 0.50; **omit** without Focus; **renormalize** when CSR absent
  - ADR-007: expected slots = 2; coverage × mean(upstream Feature.confidence); optional factors `focus_stability` / `switch_stability`
  - Calm framing: “focus stability in this window” — personal self-tracking only
  - Schema: **no** new Observation / **no** migration; do **not** rewrite FocusScore / CSR / DeepWorkScore
  - E2 ships math in `feature-engine` + catalog §1; optional E3 dogfood / Dashboard
  - Rejected: ADHD / “can’t focus”; workplace; new Observations; leaf/sibling rewrites; Focus-level intensity; IDE/weather/App Store; CircadianOffset as P22 primary; PR during freeze; migration without approve; parallel engine
- Docs planned/ADR notes: `06-feature-catalog` (§1.18 stub + removed from backlog), `12-development`, `16-glossary`
- **No** Feature math implementation; **no** DAG registration; **no** SQLite migration applied

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/22-attention-stability`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-023" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Locked inputs + omit policy + distinct from DeepWorkScore
rg -n "FocusScore|ContextSwitchRate|focus_stability|Focus range|omit without Focus|AttentionStability|DeepWorkScore|renormalize" \
  docs/decision-log.md docs/06-feature-catalog.md

# Rejected alts
rg -n "ADHD|you can't focus|workplace|CircadianOffset|IDE|weather|App Store|PR during freeze|migration|DeepWorkScore" \
  docs/decision-log.md

# No Feature code for this task
rg -n "AttentionStability" crates/feature-engine || true
# expect: no matches (math → P22-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-023 records Phase 22 primary = `AttentionStability`; Feature-level Focus+CSR; rationale vs DeepWorkScore + deferrals; personal self-tracking; calm framing
- [ ] AC2: Formula stance — 15m/1m; 0–100; Focus range (not level) + CSR stability; omit without Focus / renormalize without CSR; ADR-007 slots=2; optional factors; no parallel FocusScore; no DeepWorkScore redefine
- [ ] AC3: Rejected alternatives documented (ADHD/“can’t focus”; workplace; new Observations; leaf rewrites; IDE/weather/App Store; CircadianOffset; PR freeze; migration)
- [ ] AC4: Schema none to apply — no migration; no new Observation `data_type`
- [ ] AC5: E2 Feature + catalog; optional E3 dogfood/Dashboard sketched
- [ ] AC6: Docs touched (`06` / `12` / `16`) as planned/ADR notes
- [ ] AC7: Handoff `docs/handoffs/P22-E1-T1-dev-to-qa.md`
- [ ] Global DoD: UI↛DB unchanged; personal self-tracking; PR freeze; no Feature impl in E1

## Risks / not covered
- Exact `register_*` helper name and how E2 collects in-window Focus samples from the DAG batch deferred to **P22-E2** (must still obey ADR-023).
- Weight 0.50/0.50 may be refined in E2 only within ADR bounds (Focus still required; must stay distinct from DeepWorkScore).

## Notes for QA
- Do **not** expect `feature-engine` to know `AttentionStability` yet — math is **P22-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty PM-GATE / Phase 21 open docs may exist on the branch — out of AC unless they contradict ADR-023.
