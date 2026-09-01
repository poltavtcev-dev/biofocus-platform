# Dev|UX → QA: P9-E3-T1

## Meta
- **Task ID:** P9-E3-T1
- **Title:** Recommendations IPC / UX
- **Role that built:** Dev (+ UX)
- **Date:** 2026-08-08
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P9-E3-T1; brief `docs/handoffs/P9-E3-T1-pm-brief.md`
- **Branch:** `phase/9-recommendations`

## What changed
- **IPC `get_recommendations`:** Host registers `register_recommendations_v1` with Insights at startup; evaluate-on-read after Insights on Feature snapshot + pattern baseline inputs; DTO `{ recommendations: [{ id, title, suggestion, category, evidenceList }] }` with Feature/Signal/Insight kinds. Soft-fail / empty → `[]`.
- **Dashboard:** Insights-adjacent **Suggestions** section — Core title/suggestion as returned; calm category (`Pace`); empty/loading/error states; no Coach window.
- **Mocks:** `?view=dashboard&mockRecommendations=empty|ready|pace|error` (`ready`/`pace` = `focus_dip_pace_hint_v1`-shaped sample). Documented in `apps/desktop/README.md`.
- Contracts: `docs/09-api.md`, `docs/12-development.md`.
- Unit test: lower-baseline Focus → pace Recommendation with Feature + Insight evidence on wire.

## Crates / apps / files touched
- `apps/desktop/src-tauri/src/lib.rs`, `alert_state.rs`
- `apps/desktop/src/{recommendations.ts,Dashboard.tsx,insights.ts,App.css}`
- `apps/desktop/README.md`
- `docs/09-api.md`, `docs/12-development.md`
- `docs/handoffs/P9-E3-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p desktop registered_engine_emits_pace_recommendation
cd apps/desktop && pnpm exec tsc --noEmit
```

### Smoke (browser / Tauri)
1. **Mock pace:** open Dashboard with  
   `?view=dashboard&mockRecommendations=pace`  
   → Suggestions shows “A gentler pace may help” + suggestion text + Evidence Feature FocusScore · Insight …
2. **Mock empty:** `?mockRecommendations=empty` → calm “No suggestions yet” (no error noise).
3. **Mock error:** `?mockRecommendations=error` → calm load-failure copy.
4. **Dogfood (optional):** multi-day afternoon FocusScore history where **current Focus is lower** than baseline (|Δ| ≥ 10, confidence ≥ 0.4) → live `get_recommendations` may emit pace hint alongside pattern Insight. Thin history → empty Suggestions, not an error.
5. Confirm UI only uses `invoke("get_recommendations")` — no SQLite / no Recommendation persistence.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `get_recommendations` IPC + host `register_recommendations_v1`; DTO with evidence kinds; empty → calm `[]`
- [ ] AC2: Dashboard Suggestions section (Insights-adjacent) shows title/suggestion; calm non-clinical chrome
- [ ] AC3: Mock path `empty|ready|pace|error` documented (README + handoff)
- [ ] AC4: Smoke notes (mock + dogfood path); UI ↛ SQLite; no Recommendation persistence
- [ ] AC5: This handoff exists
- [ ] Global DoD: UI↛DB; LLM not inventing Recommendations on this path; calm copy

## Risks / not covered
- Host does not auto-call LLM on Recommendations path (same as Insights).
- Higher-than-baseline Focus still yields pattern Insight but **no** pace Recommendation (engine gate unchanged).
- Phase 9 close / PR → PM only (PR freeze).

## Notes for QA
- Prefer mock smoke for UI; unit test covers live evaluate path without full Tauri window.
- Suggestions section title is “Suggestions” (calm UX); IPC/command remains `get_recommendations`.
