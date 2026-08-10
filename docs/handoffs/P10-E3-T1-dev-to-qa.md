# Dev → QA: P10-E3-T1

## Meta
- **Task ID:** P10-E3-T1
- **Title:** Catalog Feature `DistractionScore`
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P10-E3-T1; brief `docs/handoffs/P10-E3-T1-pm-brief.md`
- **Branch:** `phase/10-plugin-wave-1`

## What changed
- **`DistractionScore`** catalog node: browser category mix + churn (+ optional CSR); calm fragmentation framing; omit empty / only-`unknown`.
- Pipeline normalizes `browser_category` (closed-set; strips `url`/`title`/`href`).
- Registered via `register_distraction_v1` → `register_catalog_v1` (after Focus/CSR).
- ADR-007 confidence (slots browser/CSR) + explanation factors; unit tests green.
- Catalog §1.8 + glossary / API / 12-development notes; fixed stale merge conflict markers in `12-development.md` git policy section.

## Crates / apps / files touched
- `crates/feature-engine/src/catalog/distraction_score.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`, `lib.rs`
- `crates/pipeline/src/normalize.rs`
- `docs/06-feature-catalog.md`, `16-glossary.md`, `09-api.md`, `12-development.md`
- `docs/handoffs/P10-E3-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p feature-engine distraction --lib
cargo test -p feature-engine --lib
cargo test -p pipeline browser_category
cargo check -p feature-engine -p pipeline
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `DistractionScore` in catalog §1 (goal/window/formula/provenance/confidence/DAG); calm framing
- [ ] AC2: Inputs `browser_category` required; optional CSR documented as improving signal
- [ ] AC3: Pipeline known type; strips forbidden keys; closed-set category
- [ ] AC4: Registered in `register_catalog_v1`; appears on Feature path when inputs present
- [ ] AC5: Empty / only-`unknown` → **omit**; no busy-loop; no new SQLite schema
- [ ] AC6: Unit tests: rich → emit; unknown-only → omit; confidence ADR-007
- [ ] AC7: Explanation factors present (optional AC — shipped)
- [ ] AC8: This handoff exists
- [ ] Global DoD: UL terms; calm non-clinical; UI↛DB; LLM not computing Features

## Risks / not covered
- Live OS probe often emits `unknown` only → Feature omitted until closed-set categories (scripted / future mapping) — intentional.
- No Dashboard chart / Insights / Recommendations for DistractionScore (out of scope).
- No richer URL→category mapping (out of scope).

## Notes for QA
- Formula: mix weights work10…entertainment90; churn `switches×25`; CSR `×50`; weights 0.55/0.25/0.20 renormalized.
- Stable entertainment alone ≈ 61.875 (mix/churn without CSR).
- Depend on `ContextSwitchRate` node even when CSR unused (register Focus first).
