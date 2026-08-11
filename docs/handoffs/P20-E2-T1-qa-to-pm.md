# QA → PM: P20-E2-T1

## Meta
- **Task ID:** P20-E2-T1
- **Title:** Ship catalog Feature `CognitiveLoad` per ADR-021
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P20-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine cognitive_load` — **6/6 ok**
  - Earlier full `cargo test -p feature-engine --lib` — **119/119 ok**
  - `register_cognitive_v1` + `register_catalog_v1` wiring present
  - Catalog §1.16 marked **shipped**; glossary / `12-development` updated
  - Leaf Feature files (`meeting_density` / `context_switch_rate` / `notification_pressure`) — **unchanged**
  - No `CognitiveLoad` in `bio-spec`; no migration refs
  - No `unwrap`/`expect` in production path of `cognitive_load.rs`
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Feature-level CognitiveLoad; 15m/1m; 0–100 | **Pass** |
| AC2 Normalize + equal thirds; renormalize ≥1; omit none | **Pass** |
| AC3 ADR-007 slots=3; coverage × mean(upstream.confidence) | **Pass** |
| AC4 Factors meeting/switches/notifications; calm labels | **Pass** |
| AC5 register after calendar+focus+notification; §1.16 shipped | **Pass** |
| AC6 Unit tests rich / partial / omit / confidence / factors | **Pass** |
| AC7 No leaf rewrite; no new Observation; no migration; no UI | **Pass** |
| AC8 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: windows keyed off upstream Feature ends (calendar span-safe); partial NotificationPressure-only emit confirmed by test.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P20-E2-T1** to Done; Ready **P20-E3-T1** (dogfood / optional Dashboard)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — CognitiveLoad shipped
- [ ] Write `docs/handoffs/P20-E3-T1-pm-brief.md` (optional dogfood / calm Dashboard surface)

## Suggested next Ready task
- **P20-E3-T1** — Dogfood / optional calm Dashboard surface for `CognitiveLoad` (no formula rewrite; no migration).

## Notes for PM
- Snapshot/IPC will pick up `CognitiveLoad` via existing `register_catalog_v1` host path — E3 is UX/dogfood, not math.
- PR freeze until 2026-09-01 — no PR.
