# Dev → QA: P13-E3-T1

## Meta
- **Task ID:** P13-E3-T1
- **Title:** Catalog Feature `GitActivityRate`
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P13-E3-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` Epic P13-E3
- **Branch:** `phase/13-plugin-wave-2`

## What changed
- Catalog Feature **`GitActivityRate`**: sum of `event_count` (default 1) for `activity_kind ∈ {commit, checkout, sync, other}` → **events per 15m window**; window 15m / step 1m.
- **Omit** empty / only-`idle` / only-`unknown` (same policy shape as AmbientMediaShare).
- ADR-007 single-family confidence + optional kind `ExplanationFactor`s (shares sum to 1.0).
- Registered via `register_git_v1` → wired into `register_catalog_v1` (Feature Worker / snapshot path). **Does not** touch `DistractionScore`.
- Pipeline: reused E2 `git_activity` normalize (strip forbidden keys) — no formula change needed.
- Docs: catalog §1.10; glossary / 12-development / 08-plugin-sdk / 10-security updated.

## Crates / apps / files
- `crates/feature-engine/src/catalog/git_activity_rate.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`, `crates/feature-engine/src/lib.rs`
- Docs: `06-feature-catalog.md`, `12-development.md`, `16-glossary.md`, `08-plugin-sdk.md`, `10-security.md`

## How to verify (commands)
```bash
cargo test -p feature-engine git_activity
cargo test -p pipeline normalize::tests::git_activity
rg -n "GitActivityRate|register_git_v1" docs/06-feature-catalog.md docs/12-development.md crates/feature-engine
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `GitActivityRate` in catalog §1 with goal / 15m·1m / units / inputs / formula / provenance / ADR-007 / DAG; calm framing; removed from Planned
- [ ] AC2: Inputs `git_activity`; sum event_count (default 1) for countable kinds → events/15m; omit idle/unknown-only
- [ ] AC3: Pipeline still strips forbidden keys (reuse E2 coverage)
- [ ] AC4: `register_catalog_v1` includes node; no DistractionScore merge
- [ ] AC5: Unit tests — rich emit; idle/unknown omit; confidence; event_count batches
- [ ] AC6: ExplanationFactors present for countable kinds
- [ ] AC7: This handoff
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live OS git probe remains soft-fail idle (E2) — Feature dogfoods via scripted / HTTP-ingest Observations.
- No Dashboard UI / Insights / Recommendations for this Feature (OOS).

## Notes for QA
- Policy choice: **omit** (not low-confidence emit) for thin idle/unknown windows — same shape as AmbientMediaShare / DistractionScore only-unknown.
- Units choice: **events per 15-minute window** (not per minute).
