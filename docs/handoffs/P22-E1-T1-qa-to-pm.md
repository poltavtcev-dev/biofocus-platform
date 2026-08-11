# QA → PM: P22-E1-T1

## Meta
- **Task ID:** P22-E1-T1
- **Title:** ADR-023: lock `AttentionStability` Feature scope (inputs, formula stance, omit policy)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P22-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg ADR-023` on decision-log + `06` / `12` / `16` — present (summary + detail + docs notes)
  - Feature-level FocusScore (required) + CSR (optional); Focus **range** (not level); distinct from DeepWorkScore; 15m/1m; omit-without-Focus / renormalize-without-CSR — present in ADR + catalog §1.18
  - Rejected alts (ADHD / “can’t focus”, workplace, leaf/DeepWorkScore rewrites, Focus-level intensity, IDE/weather/App Store, CircadianOffset, PR freeze, migration) — present in ADR-023
  - `rg AttentionStability crates/feature-engine` — no matches (math correctly deferred to E2)
  - Catalog backlog row for `AttentionStability` removed; §1.18 stub + Phase 22 note present
  - Handoff `docs/handoffs/P22-E1-T1-dev-to-qa.md` on disk; branch `phase/22-attention-stability`
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-023 primary = AttentionStability; Focus+CSR; vs DeepWorkScore; personal; calm framing | **Pass** |
| AC2 Formula stance: 15m/1m; Focus range + CSR stability; omit/renormalize; ADR-007 slots=2; no parallel Focus / no DeepWork redefine | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema none — no migration; no new Observation `data_type` | **Pass** |
| AC5 E2 Feature + catalog; optional E3 dogfood/Dashboard sketched | **Pass** |
| AC6 Docs `06` / `12` / `16` planned/ADR notes | **Pass** |
| AC7 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: weights Focus-stability 0.50 / switch-stability 0.50 locked in ADR sketch for E2; single-Focus sample → focus_stability = 100 (no swing observed).

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P22-E1-T1** to Done; Ready **P22-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — ADR-023 locked
- [x] Write `docs/handoffs/P22-E2-T1-pm-brief.md` shaped by ADR-023 (no schema approve expected)

## Suggested next Ready task
- **P22-E2-T1** — Ship catalog Feature `AttentionStability` in `feature-engine` per ADR-023 (15m/1m; Focus range required + optional CSR; omit/renormalize; ADR-007; catalog §1.18 finalize; unit tests; distinct from DeepWorkScore). No migration.

## Notes for PM
- Locked omit policy: **omit without FocusScore**; CSR optional with renormalize.
- Must stay **distinct** from DeepWorkScore — Focus **range**/consistency, not Focus-level intensity.
- PR freeze until 2026-09-01 — no PR.
