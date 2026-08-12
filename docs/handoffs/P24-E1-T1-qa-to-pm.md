# QA → PM: P24-E1-T1

## Meta
- **Task ID:** P24-E1-T1
- **Title:** ADR-025: lock `CircadianOffset` Feature scope (inputs, formula stance, omit policy)
- **Date:** 2026-08-12
- **Dev/UX handoff:** `docs/handoffs/P24-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg ADR-025` on decision-log + `06` / `12` / `16` — present (summary + detail + docs notes)
  - Inputs / omit / units / siblings — Observation-level `sleep_interval` + work/activity timing; 15m/1m + 24h lookback; 0–100; omit unless both slots; distinct from SleepDebt / DeskAwayPresence
  - Rejected alts (chronotype / night owl / workplace schedule / signed hours / new Observations / leaf rewrites / IDE/weather/App Store/TypingRhythm / GPS / PR freeze / migration / parallel engine) — present in ADR-025
  - `rg CircadianOffset crates/feature-engine` → **no matches** (math correctly deferred to E2)
  - Catalog: §1.20 stub; removed from §2 backlog; Phase 24 note updated
- AC results (pass/fail per item):

| AC | Result |
| :--- | :--- |
| AC1 ADR-025 primary = CircadianOffset; Observation-level sleep + work/activity timing; rationale vs Personal Context; personal; calm framing | **Pass** |
| AC2 Formula stance — 15m/1m + 24h lookback; 0–100 not signed hours; omit unless both slots; ADR-007 slots=2; factors; no SleepDebt/EnergyScore/FocusScore rewrite | **Pass** |
| AC3 Rejected alternatives documented | **Pass** |
| AC4 Schema none — no migration; no new Observation `data_type` | **Pass** |
| AC5 E2 Feature + catalog; optional E3 dogfood/Dashboard sketched | **Pass** |
| AC6 Docs `06` / `12` / `16` planned/ADR notes | **Pass** |
| AC7 Handoff `docs/handoffs/P24-E1-T1-dev-to-qa.md` | **Pass** |
| Global DoD: no Feature impl in E1; UI↛DB; personal; PR freeze | **Pass** |

- Extra checks: hard-gap = zero new Observation families; Feature-level magnitude proxies explicitly rejected.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P24-E1-T1** to Done; Ready **P24-E2-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — ADR-025 locked
- [x] Write `docs/handoffs/P24-E2-T1-pm-brief.md` shaped by ADR-025 (no schema approve expected)

## Suggested next Ready task
- **P24-E2-T1** — Ship catalog Feature `CircadianOffset` in `feature-engine` per ADR-025 (15m/1m + 24h lookback; Observation-level sleep + work/activity timing; omit unless both slots; 0–100; ADR-007; catalog §1.20 finalize; unit tests). No migration.

## Notes for PM
- Branch: `phase/24-circadian-offset` (PR freeze — no PR).
- Docs-only E1; unrelated dirty Phase 23 tree may still be present — fold into cluster later if needed.
- E2 must not invent Feature-level SleepDebt×ActivityBalance “alignment” or signed chronotype hours.
