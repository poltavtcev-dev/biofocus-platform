# Dev → QA: P21-E2-T1

## Meta
- **Task ID:** P21-E2-T1
- **Title:** Ship catalog Feature `DeepWorkScore` per ADR-022
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P21-E2-T1-pm-brief.md` · ADR-022

## What changed
- New catalog node **`DeepWorkScoreNode`** (`crates/feature-engine/src/catalog/deep_work_score.rs`):
  - Feature-level inputs: `FocusScore` (**required**) + `ContextSwitchRate` (**optional**)
  - `focus = FocusScore`; `stability = clamp(100 - CSR×50, 0, 100)`; weights 0.60 / 0.40
  - **Omit** without Focus (windows driven from FocusScore ends — CSR-only cannot emit)
  - **Renormalize** Focus-only when CSR absent
  - ADR-007: expected slots = 2; coverage × mean(upstream Feature.confidence)
  - Explanation factors: `focus` / `stability` (calm labels)
- `register_deep_work_v1` + wired into `register_catalog_v1` (after cognitive)
- Catalog §1.17 finalized (shipped); glossary + `12-development` notes updated
- Unit tests: Focus+CSR · Focus-only · omit without Focus · confidence · catalog register
- **No** leaf Feature formula changes; **no** migration; **no** UI/Dashboard

## Crates / apps / files touched
- `crates/feature-engine/src/catalog/deep_work_score.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`
- `crates/feature-engine/src/lib.rs`
- `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`
- Branch: `phase/21-deep-work-score`

## How to verify (commands)
```bash
cargo test -p feature-engine deep_work_score
cargo test -p feature-engine --lib

rg -n "DeepWorkScore|register_deep_work_v1" \
  crates/feature-engine/src/catalog/mod.rs \
  crates/feature-engine/src/lib.rs \
  docs/06-feature-catalog.md

# Leaf formulas untouched
git diff --name-only -- \
  crates/feature-engine/src/catalog/focus_score.rs \
  crates/feature-engine/src/catalog/context_switch_rate.rs
# expect: empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: DeepWorkScore in feature-engine; Feature-level; 15m/1m; 0–100
- [ ] AC2: Composition + weights; omit without Focus; renormalize without CSR; no CSR-only emit
- [ ] AC3: ADR-007 slots=2
- [ ] AC4: Factors focus/stability; calm labels (no flow/burnout/ADHD)
- [ ] AC5: Registered after focus nodes; catalog §1.17 shipped
- [ ] AC6: Unit tests Focus+CSR / Focus-only / omit / confidence / factors
- [ ] AC7: No leaf rewrite; no migration; no UI; no parallel FocusScore
- [ ] AC8: Handoff present
- [ ] Global DoD: no unwrap/expect in prod; UI↛DB; PR freeze

## Risks / not covered
- Dashboard / dogfood → **P21-E3**.
- Weight 0.60/0.40 stays per ADR-022.

## Notes for QA
- Kanban Done / canvas are PM-only after QA Pass.
