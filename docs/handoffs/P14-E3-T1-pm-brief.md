# PM Brief → Dev|UX: P14-E3-T1

**From:** PM  
**To:** Dev|UX  
**Status:** Ready  
**Date:** 2026-08-10  
**Closed previous:** P14-E2-T1 (QA Pass — live allowlist probe); Epic **P14-E2** ✅  
**Evidence:** `docs/handoffs/P14-E2-T1-qa-to-pm.md`

## Task
**P14-E3-T1 — Dogfood gate + calm Git allowlist Settings/IPC**

## Why
ADR-014 + P14-E2 shipped config allowlist (`~/.biofocus/git-watched-roots.toml`) and live `SystemGitActivityProbe` under user roots. Phase 14 still needs an operator **dogfood path** and a calm way to edit the allowlist without hand-editing TOML / without UI→SQLite — then close the Phase.

## Acceptance Criteria
1. Dogfood runbook section in `docs/12-development.md` (and short cross-links in `08-plugin-sdk` / `10-security` as needed): enable `BIOFOCUS_GIT_ACTIVITY=1`, non-empty `git-watched-roots.toml` (or documented CI env override when file absent), expect coarse `git_activity` → existing `GitActivityRate`; soft-fail when allowlist empty; privacy reminder (no paths in Observations).
2. Desktop IPC (host) to **get** and **set** watched roots for the ADR-014 config file only — validate absolute roots / soft-fail empty; **never** write roots into Observation payloads or default logs; UI ↛ SQLite.
3. Minimal calm Settings/Menubar surface (or existing settings-adjacent panel) to list/add/remove roots — personal self-tracking framing only; no workplace / surveillance copy; no mandatory new Dashboard chart for `GitActivityRate`.
4. Tests / mocks where practical: IPC round-trip to temp `BIOFOCUS_HOME` (or equivalent) config file; empty set → empty file / soft-fail semantics documented; QA browser mock flag if UI needs it.
5. **No** `GitActivityRate` formula rewrite; **no** SQLite allowlist table / migration; ADR-013 payload contract unchanged.
6. Docs mark Phase 14 companion shipped (glossary / status notes as needed).
7. Handoff: `docs/handoffs/P14-E3-T1-dev-to-qa.md` (Role: Dev|UX).

## Out of scope
- IDE collector; weather/light; App Store packaging product; NotificationPressure
- Workplace / manager dashboards; whole-disk scans
- Reworking Feature DAG math; new Insights / Recommendations for Git
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- ADR-014 + ADR-013 contracts
- Branch: `phase/14-git-allowlist`
- Prefer extend existing Tauri IPC patterns — no parallel settings store
- Personal self-tracking only — not workplace monitoring
- LLM remains L5 interpret-only

## After QA Pass
PM → mark P14-E3-T1 Done; close Epic **P14-E3** and **Phase 14** Kanban if no further P14 tasks; next horizon via separate PM gate — **without** opening a PR during freeze.
