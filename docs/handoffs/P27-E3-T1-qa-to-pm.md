# QA → PM: P27-E3-T1

## Meta
- **Task ID:** P27-E3-T1
- **Title:** Dogfood notes + optional calm Insights/Suggestions surface for ADR-028 rules
- **Date:** 2026-10-10
- **Dev/UX handoff:** `docs/handoffs/P27-E3-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
### Commands run + results
| Check | Result |
| :--- | :--- |
| `cargo test -p knowledge-engine -- cognitive_load_elevated sustained_load_elevated combined_demand adr028_rules_fire_together` | **Pass** — 18/18 (38 other tests filtered out). Emit ≥ 60, omit absent / below / non-scalar, pace hint only after demand Insight, rules stay distinct |
| `apps/desktop` `./node_modules/.bin/tsc --noEmit` | **Pass** — Vite started only after tsc exited 0 |
| Compiled `formatInsightCategory` / `formatRecommendationCategory` / mock parsers (20 assertions) | **Pass** — `demand` → Общая нагрузка; `prolonged_load` → Длительная нагрузка; `stress` stays Напряжение; `pace` → Темп; empty mocks have 0 rows; legacy `mockRecommendations=pace` title unchanged; new copy has no burnout / overload / workplace wording |
| `rg sqlite` on `apps/desktop/src` `*.ts` / `*.tsx` | **Pass** — no matches |
| `git diff -- crates/knowledge-engine` | **Pass** — empty (no rule / Feature / migration edit) |
| Dogfood § + glossary + `09-api` | **Pass** — § Pattern rules dogfood present; E2 slate text kept; ADR-028 not rewritten |

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 Dogfood notes for the three rules, omit, idle | **Pass** | `docs/12-development.md` § Pattern rules dogfood |
| AC2 Calm framing; SustainedLoad ≠ High_Stress ≠ CognitiveLoad Insight | **Pass** | Labels and mock copy; `stress` label unchanged |
| AC3 Existing Insights/Suggestions path; calm labels; empty quiet | **Pass** | Same `get_insights` / `get_recommendations` poll. Label map only |
| AC4 Smoke; UI ↛ SQLite; no Feature math / migration | **Pass** | Commands above. No new schema |
| AC5 No trigger rewrite, extra rules, store, or Coach crate | **Pass** | knowledge-engine diff empty |
| AC6 `12-development` / `16-glossary` E3 dogfood; ADR-028 and E2 notes stay | **Pass** | Also `09-api` category chrome + mock query names |
| Global DoD / Pattern rules DoD | **Pass** | Evaluate-on-read unchanged; no new production `unwrap` |

### Extra checks
- Dashboard `InsightsSlot` / `RecommendationsSlot` still call `formatInsightCategory` / `formatRecommendationCategory` and hide an empty label.
- Live painted Dashboard in a browser was **not** driven: Chrome DevTools was not connected, and starting a remote-debug Chrome was not run. Label behavior was checked by executing the same functions the slots call.
- Working tree also contains an unrelated dashboard-frame fix (`dashboard_window.rs`, `lib.rs`, roadmap bug note). Out of this AC.

## Defects (if any)
- None blocking.
- Note: checkout is `main` @ `6209d0c`, while the brief names `phase/27-pattern-rules` (branch exists, not checked out). No PR opened.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P27-E3-T1 → Done; close Epic **P27-E3** and **Phase 27** (no further P27 tasks). Do **not** claim public launch Done
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Status docs if needed: `ARCHITECTURE_STATUS` / `14-roadmap` / `PROJECT_CANVAS` / `00-vision` — Phase 27 closed on dogfood; public launch still not Done
- [ ] Next Ready: **PM-GATE-POST-P27** (or park). Calendar date of this report is **2026-10-10** (after the 2026-09-01 PR-freeze date in the brief). This chat did **not** open a PR
- [ ] Leave the unrelated dashboard-frame dirty files out of the P27 close unless that bug is already accepted

## Suggested next Ready task
- **PM-GATE-POST-P27** — choose the next phase. Do not mark public launch Done.

## Notes for PM
- Surface path reused: Dashboard Insights / Suggestions via `get_insights` / `get_recommendations`. Docs-only would have left raw `demand` / `prolonged_load`; those now have calm labels.
- This chat did **not** mark Kanban Done and did **not** edit the execution canvas.
