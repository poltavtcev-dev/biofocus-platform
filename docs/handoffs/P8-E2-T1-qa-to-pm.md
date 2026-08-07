# QA → PM: P8-E2-T1

**From:** QA  
**To:** PM  
**Date:** 2026-08-07  
**Dev/UX handoff:** `docs/handoffs/P8-E2-T1-dev-to-qa.md`  
**Verdict:** Pass with notes

## What was verified

### Commands
```bash
cargo test -p feature-engine -p knowledge-engine --lib   # 60 + 21 passed
cargo test -p desktop --lib                              # 28 passed (incl. pattern_host memo)
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 `focus_vs_recent_baseline_v1` — current FocusScore vs ≤7 afternoon series; δ=10; confidence ≥0.4; Evidence Feature; calm copy | **Pass** |
| AC2 Observation load + bounded recompute; no Correlation Engine; no Feature-history migration | **Pass** — `pattern_host` + `feature_engine::baseline`; Core only |
| AC3 Idle / privacy — no always-on worker; thin → omit; local-only | **Pass** |
| AC4 Optional in-process memo (TTL/watermark), not SQLite | **Pass** — `BaselineMemoState` |
| AC5 Unit tests: thin / rich+Evidence / calm copy | **Pass** |
| AC6 Dev handoff present | **Pass** |
| Global DoD (no prod unwrap/expect; UI↛DB; glossary) | **Pass** — `expect` only under `#[cfg(test)]` |

### Extra checks
- `register_insights_v1` → 3 rules; snapshot-only `evaluate()` still omits baseline without series.
- No new SQLite migrations / Feature-history tables in touched crates.
- Clinical-word scan in knowledge-engine rule tests (incl. pattern copy).

## Defects
None blocking.

## Notes for PM (non-blocking)
- Afternoon bucket is **UTC 13:00–17:00** (v1 simplification) — document in `12-development.md` / catalog note if useful.
- Confidence gate `0.4` and δ `10` are rule constants aligned with ADR-008 sketch.
- Dashboard / Insights UX polish remains **P8-E3-T1** (Core `get_insights` path already evaluates the new rule).

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P8-E2-T1 → Done; Ready **P8-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `docs/12-development.md` (Pattern Discovery baseline note); `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` Phase 8 progress

## Suggested next Ready task
- **P8-E3-T1** — Insights IPC / UX for patterns (surface baseline Insights calmly in Dashboard)

## Notes for PM
- Branch tip: `phase/8-pattern-discovery` (code + handoffs; PR freeze until 2026-09-01).
- Epic P8-E2 closable after this Done; E3 is UX/IPC polish.
