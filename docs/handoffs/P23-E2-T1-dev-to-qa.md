# Dev → QA: P23-E2-T1

## Meta
- **Task ID:** P23-E2-T1
- **Title:** Ship first slice per ADR-024: `DeskAwayPresence` (+ health→prompt)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P23-E2-T1-pm-brief.md` · ADR-024
- **Branch:** `phase/23-personal-context`

## What changed
### PRIMARY — `DeskAwayPresence` (shipped)
- New catalog node `DeskAwayPresenceNode` (`crates/feature-engine/src/catalog/desk_away_presence.rs`):
  - 15m / 1m; output 0–100 or omit
  - Inputs: keystrokes / `context_window` / `step_count` / `life_event` walk (existing only)
  - **Omit** without positive away evidence (walk **or** steps ≥ 40); quiet alone insufficient; active typing + weak steps without walk → omit
  - **No** GPS / new Observation `data_type`
  - ADR-007 expected slots = 3; factors `input_quiet` / `steps` / `walk_event`
  - `register_desk_away_v1` + `register_catalog_v1`
- Catalog §1.19 stub → **shipped**

### SECONDARY — health→prompt (shipped this task)
- `report-engine` module `health_context`: load `~/.biofocus/health-context.toml` (or `BIOFOCUS_HOME`)
- Closed-set ids: `sleep_sensitive` / `migraine_prone` / `caffeine_sensitive` + optional note
- Missing/empty → no injection
- `build_report_with_pack` loads file; `build_report_with_pack_and_health` for explicit override
- Markdown section “User-declared context” + L5 prompt instructions — **not** diagnosis; Feature math untouched

### Deferred
- Large literature-band Insight library (Variant B) — **not** this task

## Crates / apps / files touched
- `crates/feature-engine/src/catalog/desk_away_presence.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`, `lib.rs`
- `crates/report-engine/src/health_context.rs` (new)
- `crates/report-engine/src/packs.rs`, `lib.rs`, `Cargo.toml` (+ `toml`)
- `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`
- **Not touched:** UI/Dashboard; Focus/Stress formulas; migrations; knowledge-engine literature rules

## How to verify (commands)
```bash
cargo test -p feature-engine desk_away
cargo test -p report-engine

rg -n "DeskAwayPresence|register_desk_away_v1|MIN_STEPS_AWAY" \
  crates/feature-engine docs/06-feature-catalog.md

rg -n "health-context|User-declared context|build_report_with_pack_and_health" \
  crates/report-engine docs/12-development.md docs/16-glossary.md

# no GPS / migration
rg -n "latitude|longitude|geolocation" crates/feature-engine/src/catalog/desk_away_presence.rs || true
git diff --name-only | rg -i migrat || echo "(no migration)"

# Feature math does not import health context
rg -n "health_context|HealthContext" crates/feature-engine || echo "(none — good)"
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: DeskAwayPresence shipped — 15m/1m; omit policy; no GPS; ADR-007; factors; register; catalog §1.19; unit tests
- [ ] AC2: Health→prompt shipped (or leftover noted) — local toml; closed-set; pack injection; no Feature branching; no migration
- [ ] AC3: Literature library deferred
- [ ] AC4: Docs `12` / `16` ship notes
- [ ] AC5: No diagnosis invent; no leaf rewrites; no UI; no CircadianOffset; no PR
- [ ] AC6: Handoff states secondary landed
- [ ] Global DoD

## Risks / not covered
- Dashboard / dogfood UI → **P23-E3**
- Hosts that call `build_report` (Phase 4 path) without packs do not auto-inject health — pack path does
- Exact step threshold (40) / saturation (250) may be tuned later within ADR omit locks

## Notes for QA
- Secondary **did land** in this task (not leftover).
- Kanban Done / canvas are PM-only.
