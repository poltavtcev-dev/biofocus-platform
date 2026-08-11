# Dev → QA: P16-E2-T1

## Meta
- **Task ID:** P16-E2-T1
- **Title:** Catalog Feature `AmbientLightShare`
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P16-E2-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P16-E2
- **Branch:** `phase/16-ambient-light`

## What changed
- Catalog Feature **`AmbientLightShare`**: share of window samples with `light_kind ∈ {dark, dim, moderate, bright}` → 0–100; window 15m / step 1m.
- **Omit policy (documented + tested):** empty / only-`unknown` → **omit** Feature.
- Optional `level` accepted on Observations but **not** used in v1 value/factors (coarse band-share only).
- **Confidence (ADR-007):** single family; `mean(Observation.confidence)`. Band **explanation factors** when ≥1 closed-set sample (shares sum 1.0).
- Registered via `register_ambient_light_v1` → wired into `register_catalog_v1` (Feature Worker / snapshot path).
- Docs: catalog §1.11 finalized (no planned/sketch); glossary / `12-development` / `08-plugin-sdk` / `07-contracts` / `10-security` mark Feature shipped.
- **No** SQLite migration; **no** new ADR; **no** Phase 17 / Dashboard light chart.

### Crates / files touched
- `crates/feature-engine/src/catalog/ambient_light_share.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`
- `crates/feature-engine/src/lib.rs`
- `docs/06-feature-catalog.md`
- `docs/08-plugin-sdk.md`
- `docs/07-contracts.md`
- `docs/10-security.md`
- `docs/12-development.md`
- `docs/16-glossary.md`

## How to verify (commands)
```bash
cargo test -p feature-engine ambient_light
cargo test -p feature-engine

# Docs / registration sanity
rg -n "AmbientLightShare|register_ambient_light_v1" \
  docs/06-feature-catalog.md docs/12-development.md docs/08-plugin-sdk.md docs/16-glossary.md \
  crates/feature-engine/src
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `AmbientLightShare` in catalog §1.11 with goal / 15m·1m / units / inputs / formula / omit / provenance / ADR-007 / DAG; calm framing; no planned/sketch
- [ ] AC2: Inputs `ambient_light`; closed-set share → 0–100; omit empty/unknown-only; level unused in v1 value
- [ ] AC3: `register_ambient_light_v1` → `register_catalog_v1`; no mandatory new Dashboard UI
- [ ] AC4 (optional): ExplanationFactors for closed-set bands (sum 1.0)
- [ ] AC5: Unit tests — rich emit; empty/unknown omit; confidence ADR-007
- [ ] AC6: Docs Feature shipped (catalog / glossary / `12-development` as needed)
- [ ] AC7: No migration / new ADR / Phase 17
- [ ] AC8: Handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; no PR during freeze

## Risks / not covered
- Live OS ambient-light mapping still soft-fails (OOS) — Feature verified on scripted/fixture Observations.
- No E2E `pipeline_e2e` fixture with `ambient_light` (unit coverage sufficient per AC).
- No Insights / Recommendations rules for AmbientLightShare (OOS).

## Notes for QA
- Prefer focused tests via `AmbientLightShareNode` / `register_ambient_light_v1`; production path = `register_catalog_v1`.
- Policy: **omit** (not low-confidence emit) for unknown-only windows — same shape as AmbientMediaShare / DistractionScore.
- Do **not** start Phase 17 in this chat; PM closes Done.
