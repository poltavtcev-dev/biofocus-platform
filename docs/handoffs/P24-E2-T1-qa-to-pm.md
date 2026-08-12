# QA → PM: P24-E2-T1

**From:** QA  
**To:** PM  
**Date:** 2026-08-12  
**Dev/UX handoff:** `docs/handoffs/P24-E2-T1-dev-to-qa.md`  
**Verdict:** **Pass**

## What was verified

### Commands + results
- `cargo test -p feature-engine circadian` → **11 passed**
- `cargo check -p feature-engine` → **ok**
- `rg CircadianOffset|register_circadian_v1` on crate + catalog/glossary/dev docs → present (shipped)
- Sibling Feature node names in `circadian_offset.rs` → **only** negative proxy-test assertions (no deps)
- Sibling files (`sleep_debt` / `energy_score` / `activity_balance` / `focus_score` / `desk_away_presence`) → **no diff**
- UI/apps scan for CircadianOffset → **no matches** (E3 deferred)
- Prod path: `unwrap`/`expect` only under `#[cfg(test)]`

### AC results

| AC | Result |
| :--- | :--- |
| AC1 CircadianOffset Observation-level; 15m/1m; 24h lookback; 0–100 | **Pass** |
| AC2 Composition — sleep mid + work/activity centroid; circular map; omit-unless-both; no magnitude proxies | **Pass** |
| AC3 ADR-007 slots=2; emit only when both present | **Pass** |
| AC4 Calm factors (`sleep_timing` / `work_timing` / optional `activity_timing`) | **Pass** |
| AC5 `register_circadian_v1` + `register_catalog_v1`; §1.20 finalized | **Pass** |
| AC6 Unit tests (emit / omit sleep / omit work / confidence / factors / no proxy) | **Pass** |
| AC7 Docs `12-development` / `16-glossary` ship notes | **Pass** |
| AC8 No sibling rewrite; no new data_type; no migration; no UI; no parallel crate | **Pass** |
| Global DoD | **Pass** |

### Extra checks
- Align near mid-wake → ~100; ~12h circular offset → ~0
- Activity-only reinforcement emits `activity_timing` (no desk factor)
- Awake-only sleep intervals omit (rest-stage filter)
- Catalog registration smoke via `register_catalog_v1`

## Defects
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P24-E2-T1** Done; Ready **P24-E3-T1** (dogfood / optional Dashboard)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` — CircadianOffset **shipped** (math+DAG); E3 next
- [ ] Write `docs/handoffs/P24-E3-T1-pm-brief.md` when Ready

## Suggested next Ready task
- **P24-E3-T1** — Dogfood / optional calm Dashboard surface for CircadianOffset (chart label **Schedule alignment**); still non-clinical copy; no PR during freeze.

## Notes for PM
- Branch `phase/24-circadian-offset` holds E1 docs + E2 Feature code; PR freeze until 2026-09-01.
- v1 prefers desk evidence exclusively when present; activity reinforcement only when desk thin (ADR-aligned).
