# Dev → QA: P23-E1-T1

## Meta
- **Task ID:** P23-E1-T1
- **Title:** ADR-024: lock Personal Context Layer (Variant B + health context + desk-away)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P23-E1-T1-pm-brief.md`
- **Branch:** `phase/23-personal-context`

## What changed
- **ADR-024** in `docs/decision-log.md` (summary table + detail): Phase 23 v1 locks
  - Primary = **Personal Context Layer** with three pillars:
    1. **Reference bands Variant B** — personal baseline primary; literature secondary (cited calm orienting ranges); reject “нельзя / diagnosis”
    2. **Health context** — opt-in user-declared curated conditions; v1 consumer = **prompt packs / report** (L5); must not invent disease from biometrics; must not rewrite Feature formulas from disease tags
    3. **Desk-away** — secondary signals only; **reject precise GPS / continuous geo**; calm “away from desk in this window”
  - Layer relationships: Observations / Features / Insights / Recommendations / Prompt packs / local `~/.biofocus/` config
  - Stance sketches: case-catalog Insights; `~/.biofocus/health-context.toml`; Feature candidate **`DeskAwayPresence`**
  - **E2 order locked:** (1) DeskAwayPresence primary → (2) health→prompt secondary → (3) large literature library deferred
  - Provider universality note (contracts, not Apple-only)
  - Rejected: GPS; clinical diagnosis engine; LLM inventing conditions; cloud health sync; workplace presence; disease-tagged Feature rewrites; CircadianOffset/IDE/weather/App Store as P23 primary; PR freeze; migration; mega-rule; literature-as-E2-primary
- Docs stubs: `06-feature-catalog` §1.19 + Phase 23 note; `12-development`; `16-glossary`
- **No** Feature/Insight/UI implementation; **no** SQLite migration; **no** new Observation `data_type`

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` code for this task).
- Branch: `phase/23-personal-context`

## How to verify (commands)
```bash
# ADR + docs present
rg -n "ADR-024" docs/decision-log.md docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# Three pillars + Variant B + desk-away + health
rg -n "Variant B|Personal Context Layer|DeskAwayPresence|health-context|precise GPS|away from desk" \
  docs/decision-log.md docs/06-feature-catalog.md docs/16-glossary.md

# E2 order
rg -n "DeskAwayPresence first|PRIMARY: DeskAwayPresence|literature library" docs/decision-log.md

# Rejected alts
rg -n "precise GPS|diagnosis engine|LLM inventing|cloud health|workplace presence|CircadianOffset|IDE|weather|App Store|PR during freeze|migration" \
  docs/decision-log.md

# Universality
rg -n "provider-agnostic|Apple-only|Observation contracts" docs/decision-log.md docs/12-development.md

# No Feature code for this task
rg -n "DeskAwayPresence|PersonalContext" crates/feature-engine crates/knowledge-engine crates/report-engine || true
# expect: no matches (impl → P23-E2)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-024 records Phase 23 primary = Personal Context Layer; three pillars locked (Variant B / health / desk-away)
- [ ] AC2: Layer relationships documented (Obs / Feature / Insight / Rec / packs / local config)
- [ ] AC3: Stance sketches — case catalog; health-context.toml; DeskAwayPresence + omit + calm copy
- [ ] AC4: Rejected alternatives documented
- [ ] AC5: E2 order locked (desk-away first and/or health→prompt; literature deferred); E3 dogfood sketched
- [ ] AC6: Catalog / glossary stubs (`DeskAwayPresence` §1.19; Personal Context Layer / health context terms)
- [ ] AC7: Provider universality noted
- [ ] AC8: Handoff present
- [ ] Global DoD: no Feature impl in E1; no migration; UI↛DB; personal self-tracking; PR freeze

## Risks / not covered
- Exact DeskAwayPresence formula weights / ADR-007 slot list deferred to **P23-E2** (must still obey ADR-024 omit / no-GPS locks).
- Closed-set health condition id list is illustrative — finalize in E2 within ADR bounds.
- Optional screen-lock signal deferred (not required for E2).

## Notes for QA
- Do **not** expect `feature-engine` / prompt packs to implement DeskAwayPresence or health injection yet — that is **P23-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty Phase 22 / gate docs may exist on the branch — out of AC unless they contradict ADR-024.
