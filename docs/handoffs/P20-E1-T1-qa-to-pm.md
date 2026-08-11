# QA → PM: P20-E1-T1

## Meta
- **Task ID:** P20-E1-T1
- **Title:** ADR-021: lock `CognitiveLoad` Feature scope (inputs, formula stance, omit policy)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P20-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg ADR-021` on decision-log + `06` / `12` / `16` — present (summary + detail + docs notes)
  - Locked Feature-level inputs (`MeetingDensity` + `ContextSwitchRate` + `NotificationPressure`), 15m/1m, renormalize-partial / omit-none — present in ADR + catalog §1.16
  - Rejected alts (clinical overload/burnout, workplace, Observation-level mix, leaf rewrites, IDE/weather/App Store, PR freeze, migration) — present in ADR-021 rejected list
  - `rg CognitiveLoad crates/feature-engine` — no matches (math correctly deferred to E2)
  - Catalog backlog row for `CognitiveLoad` removed; §1.16 stub + Phase 20 note present
  - Handoff `docs/handoffs/P20-E1-T1-dev-to-qa.md` on disk
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-021 primary = CognitiveLoad; Feature-level inputs; rationale; personal self-tracking; calm framing | **Pass** |
| AC2 Formula stance: 15m/1m; 0–100; renormalize (≥1) / omit (none); ADR-007 slots=3; optional factors | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema none — no migration; no new Observation `data_type` | **Pass** |
| AC5 E2 Feature + catalog; optional E3 dogfood/Dashboard sketched | **Pass** |
| AC6 Docs `06` / `12` / `16` planned/ADR notes | **Pass** |
| AC7 Dev handoff present | **Pass** |
| Global DoD: UI↛DB; personal self-tracking; PR freeze; no Feature impl in E1 | **Pass** |

- Extra checks: branch `phase/20-cognitive-load`; no `feature-engine` CognitiveLoad code; omit-unless-all-three explicitly rejected in ADR.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P20-E1-T1** to Done; Ready **P20-E2-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: confirm `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` reflect ADR-021 locked (may already say Ready — flip to E1 Done / E2 Ready)
- [ ] Write `docs/handoffs/P20-E2-T1-pm-brief.md` shaped by ADR-021 (no schema approve expected)

## Suggested next Ready task
- **P20-E2-T1** — Ship catalog Feature `CognitiveLoad` in `feature-engine` per ADR-021 (15m/1m; Feature-level inputs; renormalize-partial; ADR-007; catalog §1.16 finalize; unit tests). No migration.

## Notes for PM
- Locked omit policy is **renormalize when ≥1 input Feature present** (not all-three-required) — important for E2 AC so NotificationPressure opt-in-off still yields partial CognitiveLoad.
- Leaf Feature formulas must stay untouched in E2.
- PR freeze until 2026-09-01 — no PR.
