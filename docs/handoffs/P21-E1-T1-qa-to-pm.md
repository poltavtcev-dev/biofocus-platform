# QA → PM: P21-E1-T1

## Meta
- **Task ID:** P21-E1-T1
- **Title:** ADR-022: lock `DeepWorkScore` Feature scope (inputs, formula stance, omit policy)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P21-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg ADR-022` on decision-log + `06` / `12` / `16` — present (summary + detail + docs notes)
  - Feature-level FocusScore (required) + CSR (optional); idle dropped; 15m/1m; omit-without-Focus / renormalize-without-CSR — present in ADR + catalog §1.17
  - Rejected alts (flow/burnout/ADHD, workplace, AttentionStability/CircadianOffset, IDE/weather/App Store, PR freeze, migration, parallel FocusScore) — present in ADR-022
  - `rg DeepWorkScore crates/feature-engine` — no matches (math correctly deferred to E2)
  - Catalog backlog row for `DeepWorkScore` removed; §1.17 stub + Phase 21 note present
  - Handoff `docs/handoffs/P21-E1-T1-dev-to-qa.md` on disk; branch `phase/21-deep-work-score`
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-022 primary = DeepWorkScore; Focus+CSR; idle dropped; personal; calm framing | **Pass** |
| AC2 Formula stance: 15m/1m; 0–100; omit without Focus / renormalize without CSR; ADR-007 slots=2; no parallel FocusScore | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema none — no migration; no new Observation `data_type` | **Pass** |
| AC5 E2 Feature + catalog; optional E3 dogfood/Dashboard sketched | **Pass** |
| AC6 Docs `06` / `12` / `16` planned/ADR notes | **Pass** |
| AC7 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: weights Focus 0.60 / CSR-stability 0.40 locked in ADR sketch for E2.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P21-E1-T1** to Done; Ready **P21-E2-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — ADR-022 locked
- [ ] Write `docs/handoffs/P21-E2-T1-pm-brief.md` shaped by ADR-022 (no schema approve expected)

## Suggested next Ready task
- **P21-E2-T1** — Ship catalog Feature `DeepWorkScore` in `feature-engine` per ADR-022 (15m/1m; Focus required + optional CSR; omit/renormalize; ADR-007; catalog §1.17 finalize; unit tests). No migration.

## Notes for PM
- Locked omit policy: **omit without FocusScore**; CSR optional with renormalize — important for E2 AC.
- Idle stays out of v1 until a future ADR finds a real idle leaf.
- PR freeze until 2026-09-01 — no PR.
