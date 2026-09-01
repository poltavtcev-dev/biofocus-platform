# Dev → QA: P22-E2-T1

## Meta
- **Task ID:** P22-E2-T1
- **Title:** Ship catalog Feature `AttentionStability` per ADR-023
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P22-E2-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` Phase 22 E2
- **Branch:** `phase/22-attention-stability`

## What changed
- New catalog node **`AttentionStabilityNode`** (`crates/feature-engine/src/catalog/attention_stability.rs`):
  - Feature-level inputs: FocusScore (**required**) + ContextSwitchRate (**optional**)
  - Window driven from FocusScore ends (CSR-only cannot emit)
  - `focus_stability` from in-window Focus **range** (`100 − (max−min)` when ≥2 samples; `100` when exactly one) — **not** DeepWorkScore Focus-level intensity
  - `switch_stability = clamp(100 − CSR × 50, 0, 100)`
  - Weights 0.50 / 0.50 with renormalize when CSR absent
  - ADR-007 `expected_slots = 2`; calm factors `focus_stability` / `switch_stability`
- `register_attention_stability_v1` + wired into `register_catalog_v1` (after DeepWork)
- Catalog §1.18 stub → **shipped** formula; `docs/12-development.md` Phase 22 note updated
- Unit tests cover Focus+CSR, Focus-only, single vs multi Focus range, omit without Focus, confidence, factors, sibling distinctness vs DeepWorkScore

## Crates / files touched
- `crates/feature-engine/src/catalog/attention_stability.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`
- `crates/feature-engine/src/lib.rs`
- `docs/06-feature-catalog.md` (§1.18)
- `docs/12-development.md` (Phase 22 shipped note)

## How to verify (commands)
```bash
cargo test -p feature-engine attention_stability
cargo test -p feature-engine deep_work_score   # sibling untouched
rg -n "AttentionStability|register_attention_stability_v1" \
  crates/feature-engine/src/catalog docs/06-feature-catalog.md
rg -n "ADHD|can't focus|burnout" \
  crates/feature-engine/src/catalog/attention_stability.rs || true
# no new Observation data_type / no migration
rg -n "AttentionStability" crates/bio-spec || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: AttentionStability in feature-engine; Feature-level; 15m/1m; 0–100
- [ ] AC2: Composition + weights (range/single-sample/CSR map); omit without Focus; renormalize without CSR; no CSR-only; no DeepWork Focus-level reuse
- [ ] AC3: ADR-007 expected_slots=2; coverage × mean upstream conf
- [ ] AC4: Calm factors `focus_stability` / `switch_stability` — not ADHD / “can’t focus” / burnout
- [ ] AC5: `register_attention_stability_v1` + `register_catalog_v1`; catalog §1.18 shipped
- [ ] AC6: Unit tests cover required cases
- [ ] AC7: No FocusScore / CSR / DeepWorkScore rewrite; no new data_type; no migration; no UI
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- No Dashboard / dogfood surface (→ **P22-E3**)
- Snapshot/IPC picks up Feature via existing `register_catalog_v1` — no dedicated UI wiring this task
- Focus-slot confidence uses **mean** of in-window Focus samples (documented in catalog); tip-only would differ on thin early windows

## Notes for QA
- Key sibling check: single Focus sample → AttentionStability **100**, DeepWorkScore **== Focus level** (`distinct_from_deep_work_focus_level_intensity`)
- Multi-sample fixture: low→high typing across span → `focus_stability < 100`
- Do **not** mark Done / touch canvas (PM)
