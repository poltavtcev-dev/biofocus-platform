# Dev|UX → QA: P8-E3-T1

## Meta
- **Task ID:** P8-E3-T1
- **Title:** Insights IPC / UX for patterns
- **Role that built:** Dev + UX
- **Date:** 2026-08-08
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P8-E3-T1; brief `docs/handoffs/P8-E3-T1-pm-brief.md`
- **Branch:** `phase/8-pattern-discovery`

## What changed
- Pattern / baseline Insights already evaluated in Core on `get_insights` (P8-E2) — no IPC re-plumb.
- Dashboard Insights rows show a calm category affordance from Core `category` (`Pattern` / `Focus` / `Stress`); title + description rendered as returned (no clinical UI chrome).
- QA mocks: `?mockInsights=pattern` (single baseline-shaped Insight) and `ready` now includes the pattern sample first.
- Docs: `docs/09-api.md` (`pattern` category + evaluate_with_pattern note), `apps/desktop/README.md` (mock + dogfood smoke), `docs/12-development.md` (P8-E3 UX note).
- Empty / thin-history still maps to calm empty list (`No Insights yet` — no error when rule omits).

### Crates / apps / files touched
| Path | Change |
| :--- | :--- |
| `apps/desktop/src/insights.ts` | `formatInsightCategory`; mock `pattern` + pattern sample in `ready` |
| `apps/desktop/src/Dashboard.tsx` | Category label on Insight rows |
| `apps/desktop/src/App.css` | `.insight-category` |
| `apps/desktop/README.md` | Pattern mock / dogfood smoke |
| `docs/09-api.md` | `get_insights` Pattern Discovery UX notes |
| `docs/12-development.md` | P8-E3-T1 entry |

## How to verify (commands)
```bash
cargo test -p desktop insights
cargo test -p desktop pattern
cargo test -p knowledge-engine baseline
cd apps/desktop && pnpm exec tsc --noEmit
```

### Smoke (manual)
1. **Mock (no Core history):** open Dashboard with  
   `?view=dashboard&mockInsights=pattern`  
   → one Insight, category **Pattern**, title “Focus relative to your recent average”, Evidence `Feature FocusScore`.  
   Also: `mockInsights=empty` → calm empty; `mockInsights=ready` → pattern + stress + focus rows.
2. **Dogfood (live):** multi-day afternoon FocusScore evidence (UTC 13:00–17:00 windows, confidence ≥ 0.4, `|Δ| ≥ 10` vs baseline mean) → `get_insights` may emit `focus_vs_recent_baseline_v1`; thin history → empty list (no error).
3. Confirm UI path is only `invoke("get_insights")` — frontend does not open SQLite / `rusqlite` / `.db`.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Pattern / baseline Insights appear via existing `get_insights` → Dashboard list; calm non-clinical copy (title/description as returned; no clinical framing in UI chrome)
- [ ] AC2: Empty / thin-history stays calm (omit or empty list — no error noise)
- [ ] AC3: Optional UX polish — category/`pattern` affordance fits existing list (not a redesign)
- [ ] AC4: Smoke notes in handoff (mock + dogfood); UI ↛ SQLite
- [ ] AC5: Handoff `docs/handoffs/P8-E3-T1-dev-to-qa.md`
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live dogfood with real multi-day FocusScore depends on local Observation history; unit tests + mocks cover layout/copy.
- No new product window / Dashboard redesign (out of scope).
- ADR-008 δ / confidence / UTC afternoon bucket unchanged.

## Notes for QA
- Core rule path was Done in P8-E2; this task is surface + calm category label.
- Do **not** mark Done / touch canvas (PM after your report).
