# QA → PM: P21-E2-T1

## Meta
- **Task ID:** P21-E2-T1
- **Title:** Ship catalog Feature `DeepWorkScore` per ADR-022
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P21-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine deep_work_score` — **6/6 ok**
  - Full `cargo test -p feature-engine --lib` — **125/125 ok** (earlier in Dev)
  - `register_deep_work_v1` + `register_catalog_v1` wiring present
  - Catalog §1.17 marked **shipped**; glossary / `12-development` updated
  - Leaf Feature files (`focus_score` / `context_switch_rate`) — **unchanged**
  - No `DeepWorkScore` in `bio-spec`; no migration; no prod `unwrap`/`expect`
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Feature-level DeepWorkScore; 15m/1m; 0–100 | **Pass** |
| AC2 Weights 0.60/0.40; omit without Focus; renormalize without CSR; no CSR-only | **Pass** |
| AC3 ADR-007 slots=2 | **Pass** |
| AC4 Factors focus/stability; calm labels | **Pass** |
| AC5 register after focus; §1.17 shipped | **Pass** |
| AC6 Unit tests Focus+CSR / Focus-only / omit / confidence / factors | **Pass** |
| AC7 No leaf rewrite; no migration; no UI; no parallel FocusScore | **Pass** |
| AC8 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: windows keyed from FocusScore ends → CSR-only structurally cannot emit.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P21-E2-T1** to Done; Ready **P21-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — DeepWorkScore shipped
- [ ] Write `docs/handoffs/P21-E3-T1-pm-brief.md` (dogfood / optional Dashboard)

## Suggested next Ready task
- **P21-E3-T1** — Dogfood / optional calm Dashboard surface for `DeepWorkScore` (no formula rewrite; no migration).

## Notes for PM
- Snapshot/IPC picks up `DeepWorkScore` via existing `register_catalog_v1` — E3 is UX/dogfood.
- PR freeze until 2026-09-01 — no PR.
