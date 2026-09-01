# Dev → QA: P12-E3-T1

## Meta
- **Task ID:** P12-E3-T1
- **Title:** Catalog Feature `AmbientMediaShare` + packaging runbook
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P12-E3-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P12-E3
- **Branch:** `phase/12-ambient-packaging`

## What changed
- Catalog Feature **`AmbientMediaShare`**: share of window samples with `is_playing && media_kind ∈ {music, podcast, other}` → 0–100; window 15m / step 1m.
- **Omit policy (documented + tested):** empty / only-`none` / only-`unknown` → **omit** Feature. Paused closed-set (`music`/`podcast`/`other` with `is_playing: false`) → emit **0**.
- **Confidence (ADR-007):** single family; `mean(Observation.confidence)`. Kind **explanation factors** when ≥1 playing sample.
- Registered via `register_ambient_v1` → wired into `register_catalog_v1` (Feature Worker / snapshot path).
- Pipeline: `now_playing` is a **known** normalized type; strips `title` / `artist` / `album` / `lyrics` / playlist / track ids.
- Docs: catalog §1.9; packaging companion `docs/18-packaging-runbook.md` (signed `.app`/`.dmg`, notarization, update-channel stance; sync off by default; AGPLv3 Core open); links from `12-development`, `08-plugin-sdk`, `10-security`, `16-glossary`.

### Crates / files touched
- `crates/feature-engine/src/catalog/ambient_media_share.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`
- `crates/feature-engine/src/lib.rs`
- `crates/pipeline/src/normalize.rs` (+ `DATA_TYPE_NOW_PLAYING` export)
- `crates/pipeline/src/lib.rs`
- `docs/06-feature-catalog.md`
- `docs/18-packaging-runbook.md` (new)
- `docs/12-development.md`
- `docs/08-plugin-sdk.md`
- `docs/10-security.md`
- `docs/16-glossary.md`

## How to verify (commands)
```bash
cargo test -p feature-engine ambient_media
cargo test -p pipeline now_playing
cargo test -p feature-engine
cargo test -p pipeline

# Docs / strip sanity
rg -n "AmbientMediaShare|register_ambient_v1|18-packaging-runbook" docs/06-feature-catalog.md docs/12-development.md docs/18-packaging-runbook.md docs/08-plugin-sdk.md
rg -n "title|artist|lyrics|playlist" crates/pipeline/src/normalize.rs | head
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `AmbientMediaShare` in catalog §1 with goal / 15m·1m / units / inputs / formula / provenance / ADR-007 / DAG; calm framing; removed from Planned
- [ ] AC2: Inputs `now_playing`; formula share → 0–100; omit empty/none/unknown-only (policy documented + tested)
- [ ] AC3: `now_playing` known normalize type; forbidden content keys stripped
- [ ] AC4: `register_catalog_v1` includes node; no mandatory new Dashboard UI
- [ ] AC5: Packaging runbook (signed build / notarization / update stance; sync off; AGPLv3 Core open) + touch `12-development` and/or dedicated doc
- [ ] AC6: Unit tests — rich playing emit; empty/none/unknown omit; confidence per ADR-007
- [ ] AC7 (optional): ExplanationFactors for playing kinds
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; no PR during freeze

## Risks / not covered
- Live MediaRemote content mapping still soft-fail (out of scope) — Feature verified on scripted/persisted Observations.
- No E2E fixture with `now_playing` in `pipeline_e2e` (unit coverage sufficient per AC).
- Notarization / codesign not executed in CI (runbook is docs/process only).

## Notes for QA
- Prefer focused tests via `AmbientMediaShareNode` / `register_ambient_v1`; production path = `register_catalog_v1`.
- Policy choice: **omit** (not low-confidence emit) for thin none/unknown windows — same shape as DistractionScore only-unknown.
