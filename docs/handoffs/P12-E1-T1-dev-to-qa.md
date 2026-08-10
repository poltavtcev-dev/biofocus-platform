# Dev → QA: P12-E1-T1

## Meta
- **Task ID:** P12-E1-T1
- **Title:** ADR-012: Phase 12 scope (ambient sources + commercial packaging)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P12-E1-T1; brief `docs/handoffs/P12-E1-T1-pm-brief.md`
- **Branch:** `phase/12-ambient-packaging`

## What changed
- **ADR-012** in `docs/decision-log.md`:
  - **Primary:** ambient wave-1 = **Now Playing / music** (`data_type: "now_playing"`; opt-in `BIOFOCUS_NOW_PLAYING` default off; coarse `media_kind` + `is_playing` only).
  - **Secondary (same phase):** commercial packaging = signed-build / notarization / update **runbook + process**; optional sync **stance** off by default (no sync product in P12); AGPLv3 Core stays open.
  - Weather / light deferred; IDE/Git stays deferred (ADR-010).
- Observation contract sketch + Capability Plugin Model; existing `observations` store; **no migration**.
- E2/E3 names locked: **P12-E2** = Now Playing plugin; **P12-E3** = `AmbientMediaShare` Feature + packaging runbook.
- Rejected: all three ambient sources in one wave; always-on capture; cloud sync by default; secret formulas; Features without Observations; packaging that forces UI→DB / cloud LLM; IDE/Git into P12; PR during freeze; mic/lyrics capture.
- Planned notes (not deferred): `08-plugin-sdk`, `07-contracts`, `10-security`, `12-development`, `16-glossary`, `06-feature-catalog`, `04-storage`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-012 summary + detail)
- `docs/08-plugin-sdk.md` §6 Now Playing planned
- `docs/07-contracts.md` (`now_playing` payload sketch)
- `docs/10-security.md` (ambient + packaging note)
- `docs/12-development.md` (ADR-012 bullet)
- `docs/16-glossary.md` (Now Playing + AmbientMediaShare)
- `docs/06-feature-catalog.md` (`AmbientMediaShare` planned)
- `docs/04-storage.md` (no migration / no ambient sync store)
- `docs/handoffs/P12-E1-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-012 present and no migration SQL applied
rg -n "ADR-012" docs/decision-log.md docs/08-plugin-sdk.md docs/07-contracts.md docs/10-security.md docs/12-development.md docs/16-glossary.md docs/06-feature-catalog.md
rg -n "now_playing|AmbientMediaShare|BIOFOCUS_NOW_PLAYING" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/16-glossary.md docs/06-feature-catalog.md
rg -n "Always-on ambient|Cloud sync by default|secret / proprietary|music \\+ weather \\+ light|PR during freeze" docs/decision-log.md
rg -n "CREATE TABLE.*(now_playing|ambient|sync)" docs/decision-log.md || true
# No crate / app code expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-012 in `docs/decision-log.md` — v1 primary track + sequencing (ambient one-of + packaging stance); rationale (vision ladder / macOS dogfood / privacy / idle)
- [ ] AC2: Ambient contract sketch — `data_type`, privacy-safe payload, `provider_id`, opt-in default off, poll/idle; existing `observations` store
- [ ] AC3: Packaging boundaries — P12 v1 vs deferred; optional sync off by default; AGPLv3 open; commercial ≠ closed math
- [ ] AC4: Rejected alternatives documented (all three ambient; always-on; sync by default; secret formulas; Features without Observations; UI→DB/cloud LLM packaging; PR during freeze; …)
- [ ] AC5: Schema/sync sketch only — **no** migration applied
- [ ] AC6: E2 → E3 sketch names locked (plugin → `AmbientMediaShare` + packaging runbook)
- [ ] AC7: Docs notes in `08` / `07` / `10` / `12` / `16` (and catalog) — not deferred
- [ ] AC8: Handoff present
- [ ] Global DoD (docs): Local-First / opt-in / Capability Model / no UI→DB change / PR freeze noted

## Risks / not covered
- Exact macOS Now Playing probe API chosen in E2 (MediaRemote / AppleScript / other) — ADR leaves probe to E2.
- Update-channel tool (Sparkle vs manual) chosen in E3 runbook — ADR only locks stance.
- No collector / Feature / installer code in this task (correct OOS).

## Notes for QA
- Docs-only task — green path is `rg` evidence + confirm `crates/` / `apps/` untouched by this ADR.
- PM open files (`SPRINT_ROADMAP`, canvas docs) may already be dirty from Phase 12 open; Dev did not mark Done.
