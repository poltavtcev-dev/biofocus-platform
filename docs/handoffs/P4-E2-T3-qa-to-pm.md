# QA → PM: P4-E2-T3

## Meta
- **Task ID:** P4-E2-T3
- **Title:** Insights IPC + Dashboard list
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P4-E2-T3-dev-to-qa.md`
- **Verdict:** Pass

## What was verified

### Commands run + results
```bash
cargo test -p desktop insights
# 3 passed (empty idle, unregistered→[], registered≥2 + evidenceList)

cargo test -p knowledge-engine
# 15 passed

cd apps/desktop && pnpm exec tsc --noEmit
# exit 0
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 IPC `get_insights` from Features/Signals (in-memory / last snapshot) | **Pass** — evaluate-on-read over `SnapshotState`; documented in handoff + `docs/09-api.md` |
| AC2 Host `new()` + `register_insights_v1` then `evaluate`; empty alone → `[]` | **Pass** — `InsightsEngineState::new()` at setup; unit test unregistered → `[]` |
| AC3 Dashboard list / calm empty + evidence refs; non-clinical copy | **Pass** — `InsightsSlot` + `insights.ts`; mocks `?mockInsights=`; rule copy reused |
| AC4 UI ↛ SQLite; no new persistence / ADR | **Pass** — UI only `invoke("get_insights")`; no schema change |
| AC5 Idle-safe refresh; Menubar / charts intact | **Pass** — parallel fetch on open + 30s; ChartSlot / Menubar paths unchanged |
| AC6 Handoff + smoke + tests | **Pass** |
| Global DoD | **Pass** — no prod `unwrap`/`expect` on new path; soft-fail → `[]` |

### Extra checks
- Wire JSON uses camelCase `evidenceList` with `kind` ∈ {`feature`,`signal`}; no Observation payloads / paths in DTO tests.
- `get_feature_snapshot` / `get_status` still registered; `get_status` stays lean.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P4-E2-T3 → Done; Ready → **P4-E3-T1** (unless re-order)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `14-roadmap.md` Ready pointer; `12-development.md` status line (T3 already documented as implemented)

## Suggested next Ready task
- **P4-E3-T1** — report-engine prompt / markdown builder (Dev), per brief.

## Notes for PM
- Branch: `phase/4-dashboard-ai` (code cluster continues; no PR required from QA).
- Host contract for consumers unchanged from T2; IPC name is `get_insights`.
