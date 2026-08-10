# PM Brief → Dev: P12-E3-T1

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P12-E2-T1 (QA Pass with notes — Now Playing plugin); Epic **P12-E2** ✅  
**Evidence:** `docs/handoffs/P12-E2-T1-qa-to-pm.md`

## Task
**P12-E3-T1 — Catalog Feature `AmbientMediaShare` + packaging runbook**

## Why
ADR-012 + P12-E2 shipped opt-in `now_playing` Observations (`NowPlayingPlugin`, `BIOFOCUS_NOW_PLAYING`, coarse `media_kind` + `is_playing`). Vision rule: Features only with real inputs. Catalog already names **`AmbientMediaShare`**. Close Phase 12 with a calm, confidence-aware catalog node **and** the secondary packaging companion (signed-build / notarization / update-channel **runbook** — no sync product).

## Acceptance Criteria
1. Move **`AmbientMediaShare`** from `docs/06-feature-catalog.md` § Planned → §1 with: goal, window/step (**15m / 1m**, align Focus/CSR / DistractionScore), units, inputs, formula strategy (v1), provenance, ADR-007 confidence, DAG registration note. Calm framing only (“media present during this window” — **not** “you listen too much” / clinical diagnosis).
2. Inputs: `now_playing` Observations required (`media_kind` + `is_playing`). Formula per ADR-012: share of window minutes/samples with `is_playing && media_kind ∈ {music, podcast, other}` → **0–100**. Empty / only-`none` / only-`unknown` thin windows → **omit** Feature **or** emit with clearly lower confidence (pick one policy; document + test).
3. Pipeline: ensure `now_playing` is a known normalized type; strip forbidden content keys (`title` / `artist` / `album` / `lyrics` / playlist ids / etc.) if present — never persist them via normalize path.
4. Register in `feature_engine::register_catalog_v1` (or helper wired into it); Feature appears on existing snapshot / Feature Worker path when inputs present — **no** mandatory new Dashboard UI.
5. Packaging companion (same task, docs): signed macOS `.app`/`.dmg` + notarization **runbook** + update-channel **stance**; optional sync remains **off by default / stance only** (no sync product, no account system). Reaffirm AGPLv3 Core stays open; commercial packaging ≠ closed Feature math. Touch `docs/12-development.md` (and/or dedicated runbook under `docs/`) as appropriate.
6. Unit tests: rich playing closed-set media → emit; empty / none-unknown-only → omit or low confidence per policy; confidence behaves per ADR-007.
7. Optional: `ExplanationFactor`s if weighted components are clear (P7-E2 shape).
8. Handoff: `docs/handoffs/P12-E3-T1-dev-to-qa.md`.

## Out of scope
- Live MediaRemote / AppleScript content mapping (production probe may stay soft-fail `None`; Feature must work on scripted / persisted `now_playing`)
- Weather / light collectors; IDE/Git; App Store listing product; cloud sync product
- New Insights / Recommendations rules for AmbientMediaShare
- Dashboard redesign / dedicated chart (snapshot path enough)
- New SQLite schema / sync store / ambient allowlist table
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-012 + E2 contract in `docs/decision-log.md` / `docs/07-contracts.md` / `docs/08-plugin-sdk.md`
- Modules: `crates/feature-engine`, `crates/pipeline` (normalize), `docs/06-feature-catalog.md`, packaging docs
- Branch: `phase/12-ambient-packaging`
- Prefer extend existing catalog registration — no parallel Feature registry
- LLM remains L5 interpret-only — must not invent score or media kinds
- Personal self-tracking only — not workplace / environmental surveillance

## After QA Pass
PM → mark P12-E3-T1 Done; close Epic **P12-E3** and **Phase 12** Kanban if no further P12 tasks; next horizon via separate PM gate — **without** opening a PR during freeze.
