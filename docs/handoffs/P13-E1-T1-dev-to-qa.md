# Dev → QA: P13-E1-T1

## Meta
- **Task ID:** P13-E1-T1
- **Title:** ADR-013: Plugin wave-2 scope (IDE or Git) + Observation contract
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P13-E1-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P13-E1
- **Branch:** `phase/13-plugin-wave-2`

## What changed
- **ADR-013** in `docs/decision-log.md`:
  - Wave-2 v1 primary = **Git activity aggregates** (exactly one source; **not** IDE in this wave).
  - Rationale vs ADR-010: IDE already in `context_window` / CSR; privacy-safe IDE session kinds without paths/titles are weak dogfood; Git ops are additive; counts/cadence only.
  - Capability Plugin Model via `plugin-sdk` + `macos-collector` (or thin adapter).
- Observation contract sketch: `data_type: "git_activity"`, provider `com.biofocus.macos.git`, payload `activity_kind` (+ optional `event_count`), opt-in `BIOFOCUS_GIT_ACTIVITY` default **off**, emit on change / ≥5s, no busy-loop.
- Explicit forbid: paths / remotes / branch names / diffs / commit messages / buffer content / workplace surveillance framing.
- Rejected: IDE+Git same wave; IDE without additive signal; always-on; cloud git sync; marketplace crate; parallel plugin SQLite; weather/light or App Store as P13 primary; NotificationPressure as this Feature; redefining `DistractionScore`; PR during freeze; path/remote persistence.
- Schema: **none to apply** — existing `observations` only; no migration.
- E2/E3 names locked: **P13-E2** = Git activity plugin; **P13-E3** = `GitActivityRate` (calm cadence framing; ADR-007).
- Planned notes (not deferred): `08-plugin-sdk`, `07-contracts`, `10-security`, `12-development`, `16-glossary`, `06-feature-catalog`, `04-storage`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-013 summary + detail)
- `docs/08-plugin-sdk.md` (§7 Git activity planned + ADR-010 note update)
- `docs/07-contracts.md` (`git_activity` payload sketch)
- `docs/10-security.md` (wave-2 Git note)
- `docs/12-development.md` (ADR-013 bullet)
- `docs/16-glossary.md` (Git Activity + GitActivityRate)
- `docs/06-feature-catalog.md` (`GitActivityRate` planned → P13-E3)
- `docs/04-storage.md` (no migration / no git registry)
- `docs/handoffs/P13-E1-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-013 present and no migration SQL applied
rg -n "ADR-013" docs/decision-log.md docs/08-plugin-sdk.md docs/07-contracts.md docs/10-security.md docs/12-development.md docs/16-glossary.md docs/06-feature-catalog.md docs/04-storage.md
rg -n "git_activity|BIOFOCUS_GIT_ACTIVITY|GitActivityRate|com.biofocus.macos.git" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/16-glossary.md docs/06-feature-catalog.md
rg -n "IDE \\+ Git|Always-on|Cloud git|NotificationPressure|DistractionScore|PR during freeze|weather|App Store" docs/decision-log.md | head -40
rg -n "CREATE TABLE.*(git|plugin|repo|sync)" docs/decision-log.md docs/04-storage.md || true
# No crate / app code expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-013 chooses **exactly one** of IDE|Git; rationale vs ADR-010; Capability Plugin Model
- [ ] AC2: Observation contract sketch — data_type, privacy-safe payload, provider_id, opt-in default off, poll/idle; forbids paths/remotes/diffs/… 
- [ ] AC3: Rejected alternatives documented (IDE+Git; always-on; cloud sync; marketplace; parallel SQLite; weather/App Store primary; NotificationPressure; PR freeze; …)
- [ ] AC4: Schema — none to apply; prefer existing `observations`; no migration
- [ ] AC5: E2 plugin → channel → persist; E3 `GitActivityRate` locked + calm framing + ADR-007; not redefining DistractionScore
- [ ] AC6: Docs touch `08` / `07` / `10` / `12` / `16` / `06` (and storage) — not deferred
- [ ] AC7: This handoff exists
- [ ] Global DoD: personal self-tracking; UI↛DB; LLM interpret-only; no PR during freeze

## Risks / not covered
- Implementing the collector plugin → **P13-E2-T1** (out of scope).
- Catalog Feature / normalize beyond sketch → **P13-E3-T1** (out of scope).
- Exact probe (FSEvents / git hooks / CLI heuristics) left to E2 — ADR locks contract + privacy bar only.
- Watched-roots allowlist SQLite config needs a **new** ADR + approve — not silently added.
- Activity kind labels may be refined slightly in E2 contracts if still coarse and closed-set.

## Notes for QA
- Decision is explicitly **Git**, not IDE, for wave-2 v1.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- PM open files (`SPRINT_ROADMAP`, vision, canvas docs) may already be dirty from Phase 13 open; Dev did not mark Done.
