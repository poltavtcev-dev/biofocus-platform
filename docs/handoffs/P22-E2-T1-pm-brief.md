# PM Brief → Dev: P22-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P22-E2** ✅ · `AttentionStability` shipped  
**Evidence:** `docs/handoffs/P22-E2-T1-qa-to-pm.md` · `docs/handoffs/P22-E2-T1-dev-to-qa.md`  
**Next:** **P22-E3-T1** — `docs/handoffs/P22-E3-T1-pm-brief.md`  
**Closed previous:** P22-E1-T1 (ADR-023 locked; QA Pass)  
**Phase:** Phase 22 AttentionStability — Epic P22-E2

## Task
**P22-E2-T1 — Ship catalog Feature `AttentionStability` per ADR-023**

## Why
ADR-023 locked Feature-level **variance/stability** composite from **FocusScore (required) + ContextSwitchRate (optional)** — distinct from DeepWorkScore intensity (Focus **range**/consistency, not Focus level). E1 docs/catalog stub §1.18 are in place; E2 implements math + DAG registration so Snapshot/IPC can emit the Feature. No new Observation family; no migration; leaf / DeepWorkScore formulas stay untouched.

## Acceptance Criteria
1. Implement **`AttentionStability`** in `feature-engine` per ADR-023: Feature-level inputs only; window **15m / 1m**; output **0–100**.
2. Composition + weights (ADR-023):  
   - `focus_samples` = FocusScore values with ends in window  
   - if ≥2 samples: `focus_stability = clamp(100 - (max−min), 0, 100)`  
   - if exactly one Focus sample: `focus_stability = 100` (no swing observed — **not** a DeepWorkScore Focus-level term)  
   - `switch_stability = clamp(100 - ContextSwitchRate × 50, 0, 100)`  
   - Catalog weights **w_focus_stab = 0.50**, **w_switch_stab = 0.50**  
   - **Omit** when FocusScore absent; **renormalize** (Focus-stability only) when CSR absent; both present → catalog weights.  
   - Do **not** emit CSR-only; do **not** invent Focus from Observations; do **not** reuse DeepWorkScore Focus-level intensity math.
3. **ADR-007** confidence: `expected_slots = 2`; coverage = present/2; `confidence = clamp(coverage × mean(upstream.confidence), 0, 1)`.
4. Optional explanation factors for present components (`focus_stability` / `switch_stability`) with calm labels — **not** ADHD / “can’t focus” / burnout copy.
5. Register in catalog / DAG after focus / DeepWork nodes (follow existing composite patterns, e.g. `register_deep_work_v1`). Finalize `docs/06-feature-catalog.md` §1.18 (stub → shipped formula).
6. Unit tests: Focus+CSR emit · Focus-only renormalize · single-vs-multi Focus range · omit without Focus · confidence · factors when emitted.
7. **Do not** rewrite FocusScore / CSR / DeepWorkScore formulas; **no** new Observation `data_type`; **no** migration; **no** UI/Dashboard surface (→ E3 optional); **no** parallel FocusScore.
8. Handoff: `docs/handoffs/P22-E2-T1-dev-to-qa.md`.

## Out of scope
- Dogfood notes / Dashboard calm surface (→ **P22-E3**)
- IDE plugin; weather ambient; App Store packaging
- Workplace / manager dashboards; clinical diagnosis copy
- Opening a PR (PR freeze until 2026-09-01)
- New collectors / Observation families

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Prefer extend `feature-engine` catalog — no parallel engine crate
- Branch: `phase/22-attention-stability`
- Personal self-tracking only — not workplace monitoring
- Copy: “focus stability in this window” — **not** ADHD / “you can’t focus” / burnout; distinct from DeepWorkScore “sustained focus”
- LLM remains L5 interpret-only
- **No migration**
- Important: omit-without-Focus is locked — CSR alone must not emit AttentionStability

## After QA Pass
PM → mark P22-E2-T1 Done; Ready **P22-E3-T1** (dogfood / optional Dashboard).
