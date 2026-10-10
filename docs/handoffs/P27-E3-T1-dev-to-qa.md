# Dev|UX → QA: P27-E3-T1

## Meta
- **Task ID:** P27-E3-T1
- **Title:** Dogfood notes + optional calm Insights/Suggestions surface for ADR-028 rules
- **Role that built:** Dev|UX
- **Date:** 2026-10-10
- **AC source:** `docs/handoffs/P27-E3-T1-pm-brief.md` · `/docs/SPRINT_ROADMAP.md` Phase 27 · ADR-028

## What changed
- Dogfood runbook **§ Pattern rules dogfood** in `docs/12-development.md`: three locked rules, ≥ 60 fixtures, omit / idle empty posture, unit commands, live IPC path, browser mocks. E2 ship notes stay; ADR-028 not reopened.
- Glossary (`docs/16-glossary.md`) and `docs/09-api.md` name the same dogfood path and calm category chrome.
- Dashboard label polish only (existing `get_insights` / `get_recommendations` poll unchanged):
  - `demand` → **Общая нагрузка**
  - `prolonged_load` → **Длительная нагрузка**
  - `pace` stays **Темп**
  - empty category still renders nothing; empty lists stay the calm empty states.
- Browser QA mocks (no Core): `mockInsights=demand|prolonged`, `mockRecommendations=demand`. Copy mirrors the shipped rule strings. No rule math, thresholds, registration, migration, or Insight/Recommendation store.

## Crates / apps / files touched
- `apps/desktop/src/insights.ts`
- `apps/desktop/src/recommendations.ts`
- `apps/desktop/README.md`
- `docs/12-development.md`
- `docs/16-glossary.md`
- `docs/09-api.md`

Not touched: `crates/knowledge-engine` rule bodies, Feature catalog, SQLite, IPC command list, `docs/SPRINT_ROADMAP.md` Kanban, execution canvas.

## How to verify (commands)
```bash
cargo test -p knowledge-engine -- cognitive_load_elevated sustained_load_elevated combined_demand adr028_rules_fire_together
cd apps/desktop && ./node_modules/.bin/tsc --noEmit
```

Browser (Vite, no Core):
- `?view=dashboard&mockInsights=demand` → category **Общая нагрузка**
- `?view=dashboard&mockInsights=prolonged` → **Длительная нагрузка**
- `?view=dashboard&mockInsights=empty` → «Наблюдений пока нет»
- `?view=dashboard&mockRecommendations=demand` → **Темп**, title «Можно ненадолго сбавить темп»
- `?view=dashboard&mockRecommendations=empty` → «Подсказок пока нет»

UI ↛ SQLite: Insights/Suggestions still `invoke("get_insights")` / `invoke("get_recommendations")` only. `rg sqlite apps/desktop/src` (ts/tsx) is empty.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: dogfood notes cover elevated CognitiveLoad / SustainedLoadIndicator ≥ 60, pace hint only after cognitive-load Insight, omit, idle empty
- [ ] AC2: calm framing; SustainedLoad distinct from High_Stress and CognitiveLoad Insights; no burnout / clinical / workplace scoring copy in the new chrome
- [ ] AC3: existing Insights/Suggestions path; new categories readable; empty stays quiet; no IPC redesign
- [ ] AC4: smoke commands above; UI ↛ SQLite; no new Feature math / migration
- [ ] AC5: rule triggers / formulas unchanged; no extra rules; no Insight/Recommendation store; no Coach crate
- [ ] AC6: `12-development` + `16-glossary` reflect E3 dogfood; ADR-028 and E2 notes remain
- [ ] Global DoD / Pattern rules DoD: evaluate-on-read; personal calm copy; no unwrap added

## Risks / not covered
- Live Desktop with a real Feature snapshot ≥ 60 was not clicked in this build (fixture tests + mock path).
- Checkout at build time is **`main`** (`6209d0c`), not `phase/27-pattern-rules`. That branch exists; this chat did not switch it because the tree already has the E2 rules plus unrelated uncommitted dashboard-frame work.
- Unrelated dirty files (do not treat as this task): `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/src/dashboard_window.rs`, the dashboard-frame bullet in `docs/12-development.md`, `docs/SPRINT_ROADMAP.md` bug note, `docs/handoffs/BUG-dashboard-fullscreen-frame-*.md`.

## Notes for QA
- Raw category ids `demand` / `prolonged_load` were the E2 gap. Polish is label-only in `formatInsightCategory`.
- `mockRecommendations=pace|ready` (focus-dip sample) is unchanged.
