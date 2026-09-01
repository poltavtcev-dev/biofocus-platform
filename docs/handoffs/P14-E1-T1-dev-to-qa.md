# Dev → QA: P14-E1-T1

## Meta
- **Task ID:** P14-E1-T1
- **Title:** ADR-014: Git watched-roots / path-allowlist + live probe boundaries
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P14-E1-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P14-E1
- **Branch:** `phase/14-git-allowlist`

## What changed
- **ADR-014** in `docs/decision-log.md`:
  - v1 durable allowlist store = **local config file** `~/.biofocus/git-watched-roots.toml` (`version` + absolute `roots`).
  - Rationale vs ADR-013 soft-fail; Capability Plugin Model fit; personal self-tracking only.
  - Rejected env-only-as-sole-store and SQLite table for Phase 14 v1 (optional CI env override may exist in E2 — not the durable SoT).
- Privacy contract: roots in config only; Observation payloads stay ADR-013 (`activity_kind` + optional `event_count`); logs prefer Observation `id` / counts — no default path dumps.
- Rejected: whole-disk scan; workplace dashboards; cloud git sync; IDE / weather-light / App Store as Phase 14 primary; NotificationPressure; PR during freeze; migration without approve; payload widening.
- Schema: **none to apply** — no SQLite allowlist; no migration. Future table sketched only as deferred (needs new ADR + user approve).
- E2/E3 sketch: E2 = file load + live `SystemGitActivityProbe` → existing channel → persist → existing `GitActivityRate` (no formula rewrite); optional E3 = Settings/IPC or dogfood gate.
- Planned notes landed (not deferred): `08-plugin-sdk`, `07-contracts`, `10-security`, `12-development`, `04-storage`, `16-glossary`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-014 summary + detail; ADR-013 deferral cross-link)
- `docs/04-storage.md`
- `docs/07-contracts.md`
- `docs/08-plugin-sdk.md` (§7.1 allowlist planned)
- `docs/10-security.md`
- `docs/12-development.md`
- `docs/16-glossary.md` (Git watched roots)
- `docs/handoffs/P14-E1-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-014 present and no migration SQL applied
rg -n "ADR-014" docs/decision-log.md docs/08-plugin-sdk.md docs/07-contracts.md docs/10-security.md docs/12-development.md docs/04-storage.md docs/16-glossary.md
rg -n "git-watched-roots\\.toml|watched.?roots|Git watched roots" docs/decision-log.md docs/08-plugin-sdk.md docs/07-contracts.md docs/10-security.md docs/04-storage.md docs/16-glossary.md
rg -n "Always-on whole-disk|SQLite allowlist|NotificationPressure|PR during freeze|migration without user approve|workplace|Cloud git|IDE wave as Phase 14" docs/decision-log.md
rg -n "CREATE TABLE|ALTER TABLE|git_watched_roots" docs/decision-log.md docs/04-storage.md || true
# No crate / app code expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-014 chooses **one** v1 storage stance (config file); rationale vs ADR-013; Capability fit; personal self-tracking
- [ ] AC2: Privacy — allowlist = user-chosen roots as config only; never paths/remotes/branch/SHA/message/diff/author in Observation payloads; logs prefer id/counts
- [ ] AC3: Rejected alternatives documented (whole-disk; workplace; cloud sync; IDE/weather/App Store primary; NotificationPressure; PR freeze; migration without approve; …)
- [ ] AC4: Schema — none to apply; non-SQLite preferred; no migration
- [ ] AC5: E2 allowlist + live probe → channel → persist → existing `GitActivityRate`; optional E3 Settings/IPC or dogfood
- [ ] AC6: Docs touch `08` / `07` / `10` / `12` / `04` / `16` — not deferred
- [ ] AC7: This handoff exists
- [ ] Global DoD: personal self-tracking; UI↛DB; LLM interpret-only; no PR during freeze; ADR-013 payload unchanged

## Risks / not covered
- Implementing allowlist load / live probe → **P14-E2-T1** (out of scope).
- Settings / IPC / dogfood gate → **P14-E3-T1** (out of scope).
- Exact FSEvents / git-dir heuristics left to E2 — ADR locks store + privacy bar only.
- Optional CI env override precedence is E2 detail if implemented.

## Notes for QA
- Decision is explicitly **config file**, not env-only and not SQLite.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- PM open files (`SPRINT_ROADMAP`, vision, canvas docs) may already be dirty from Phase 14 open; Dev did not mark Done.
