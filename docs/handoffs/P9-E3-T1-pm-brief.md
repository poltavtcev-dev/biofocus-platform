# PM Brief → Dev|UX: P9-E3-T1

**From:** PM  
**To:** Dev (+ UX)  
**Status:** Ready  
**Date:** 2026-08-08  
**Closed previous:** P9-E2-T1 (QA Pass — `focus_dip_pace_hint_v1` + evaluate path); Epic P9-E2 ✅  
**Evidence:** `docs/handoffs/P9-E2-T1-qa-to-pm.md`

## Task
**P9-E3-T1 — Recommendations IPC / UX**

## Why
Core already evaluates `focus_dip_pace_hint_v1` (ADR-009 evaluate-on-read). Phase 9 closes when Recommendations are calmly visible via a first-class IPC path and Dashboard surface — not via thin `Insight.actionRecommendation` alone, and never UI→DB.

## Acceptance Criteria
1. Implement dedicated IPC **`get_recommendations`** per `docs/09-api.md` / ADR-009: host registers `register_recommendations_v1` (and Insights path as needed); evaluate-on-read → DTO with `recommendations[]` (`id`, `title`, `suggestion`, `category`, `evidenceList` including Feature / Signal / Insight kinds). Empty / no-match → calm empty list (no error noise).
2. Dashboard calm surface (Insights-adjacent section OK — **not** a new Coach window) shows Core Recommendations with non-clinical chrome; title/suggestion as returned.
3. Mock/dev path for UI without rich history (e.g. `?mockRecommendations=empty|ready|pace|error` or equivalent) documented in handoff / README note.
4. Smoke notes in handoff: how to see a Recommendation (mock and/or dogfood with pattern Focus-below-baseline); confirm UI still does not touch SQLite; no Recommendation persistence.
5. Handoff: `docs/handoffs/P9-E3-T1-dev-to-qa.md`.

## Out of scope
- New “Coach” / Recommendations product window redesign
- Push notifications; cloud sync; dismiss/history SQLite
- Changing `focus_dip_pace_hint_v1` emit gates (document-only if copy needs a hint)
- Plugin wave-1 (P10); AI coaching polish (P11)
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `apps/desktop` (Tauri host + React) / `docs/09-api.md` (+ README mock notes if useful)
- Branch: `phase/9-recommendations`
- Core evaluate API already shipped in P9-E2 — wire host + UI; prefer reuse of Insights evaluate path patterns
- Calm copy only — optional personal hints, not medical advice
- LLM remains L5 interpret-only — must not invent Recommendations on this path

## After QA Pass
PM → mark P9-E3-T1 Done; close Epic **P9-E3** and **Phase 9** Kanban if no further P9 tasks — **without** opening a PR during freeze. Next horizon gate: Phase 10 (plugins) via separate PM brief only.
