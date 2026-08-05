# Dev|UX → QA: P4-E2-T3

## Meta
- **Task ID:** P4-E2-T3
- **Title:** Insights IPC + Dashboard list
- **Role that built:** Dev + UX
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P4-E2-T3; brief `docs/handoffs/P4-E2-T3-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- Host registers `KnowledgeEngine::new()` + `register_insights_v1` once at Tauri setup (`InsightsEngineState`).
- New IPC `get_insights`: evaluate-on-read over the same in-memory `FeatureSnapshot` cache as `get_feature_snapshot`; soft-fail / idle → `{ "insights": [] }`.
- Dashboard Insights list (title, description, evidence Feature/Signal ids, optional action) + calm empty / loading / error; refresh on open + ~30s with snapshot.
- Contract documented in `docs/09-api.md` + `docs/12-development.md`. No SQLite / ADR / new persistence.

### Crates / apps / files touched
| Path | Change |
| :--- | :--- |
| `apps/desktop/src-tauri/Cargo.toml` + `Cargo.lock` | `knowledge-engine` dep |
| `apps/desktop/src-tauri/src/lib.rs` | `get_insights` + DTOs + host registration + unit tests |
| `apps/desktop/src-tauri/src/alert_state.rs` | IPC comment includes `get_insights` |
| `apps/desktop/src/insights.ts` | IPC client + `?mockInsights=` |
| `apps/desktop/src/Dashboard.tsx` | Insights list wired to `fetchInsights` |
| `apps/desktop/src/App.css` | Insight row styles |
| `apps/desktop/README.md` | Insights + mock docs |
| `docs/09-api.md` | `get_insights` contract |
| `docs/12-development.md` | T3 entry |

## How to verify (commands)
```bash
cargo test -p desktop insights
cargo test -p knowledge-engine
cd apps/desktop && pnpm exec tsc --noEmit
```

### Smoke (manual)
1. Open Dashboard from Menubar → Insights calm empty (or list if snapshot matches rules).
2. QA mocks: `?view=dashboard&mockInsights=empty|ready|error` (ready shows 2 Insights + evidence).
3. Charts / Feature snapshot / Menubar still work; soft refresh ~30s.

## IPC contract (summary)
- **Invoke:** `invoke("get_insights")` → `{ insights: InsightDto[] }`
- **Source:** last cached `FeatureSnapshot` (Features + Signals); host `register_insights_v1` at startup.
- **Wire:** camelCase; `evidenceList[].kind` = `"feature"` \| `"signal"`; ids as strings.
- Full table: `docs/09-api.md` § `get_insights`.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: IPC `get_insights` returns Insights from Features/Signals (in-memory / last snapshot path — documented)
- [ ] AC2: Host `KnowledgeEngine::new()` + `register_insights_v1` then `evaluate`; empty/unregistered → `[]`
- [ ] AC3: Dashboard Insights list or calm empty; evidence refs (Feature/Signal ids); non-evaluative / non-clinical copy
- [ ] AC4: UI ↛ SQLite; no new persistence schema / ADR
- [ ] AC5: Idle-safe refresh (on open + ~30s); Menubar / charts still work
- [ ] AC6: Handoff + smoke steps + `cargo test` / typecheck
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live UI smoke with real High_Stress / elevated CSR depends on Feature Worker evidence; unit tests cover DTO + evaluate path; mocks cover layout.
- Insights are not cached separately — re-evaluate each invoke (intentional; AC ok).

## Notes for QA
- Unregistered engine test asserts `[]` even with matching Features/Signals.
- Registered engine test expects ≥2 Insights with `evidenceList` when High_Stress + elevated CSR present.
- Do **not** mark Done / touch canvas (PM after your report).
