# PM Brief → Dev + UX: P4-E2-T3

**From:** PM  
**To:** Dev + UX  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P4-E2-T2 (QA Pass — `register_insights_v1`: `high_stress_period_v1` + `context_switch_elevated_v1`)

## Task
**P4-E2-T3 — Insights IPC + Dashboard list**

## Why
Product rules exist in Core but Dashboard has no way to show Insights. Wire host registration + IPC so the open Dashboard lists calm Insights with evidence refs — still no SQLite, no LLM.

## Acceptance Criteria
1. IPC command `get_insights` (or equivalent documented name): returns recent Insights derived from Features/Signals (in-memory / from last snapshot path — document in handoff).
2. Host registers product rules: `KnowledgeEngine::new()` + `register_insights_v1(&mut engine)?` then `evaluate` (empty engine alone → `[]`).
3. Dashboard shows an Insights list (or calm empty state) with evidence refs (Feature/Signal ids) — non-evaluative, non-clinical copy only.
4. UI ↛ SQLite; no new persistence schema / ADR.
5. Idle-safe refresh (on open and/or rare timer); Menubar / charts still work.
6. Handoff `docs/handoffs/P4-E2-T3-dev-to-qa.md` with smoke steps + relevant `cargo test` / typecheck.

## Out of scope
- Report / LLM UI → **P4-E3**
- Insight persistence / new SQLite tables
- Changing rule thresholds / adding new product rules (CSR `1.0` is v1 constant)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Reuse: `knowledge_engine::register_insights_v1`, `FeatureSnapshot` / existing IPC patterns (`get_feature_snapshot`, Dashboard window)
- Document IPC contract briefly in handoff + `docs/09-api.md` / `12-development.md` if new command
- Branch: `phase/4-dashboard-ai`

## Host contract (from T2)
```rust
let mut engine = KnowledgeEngine::new();
register_insights_v1(&mut engine)?;
let insights = engine.evaluate(&features, &signals)?;
```

## After QA Pass
PM → Ready **P4-E3-T1** (report-engine prompt / markdown builder — Dev) unless sprint re-order.

## Next chat (скопируй в новый чат)
```text
как агент: режим build-qa для P4-E2-T3.
Brief: docs/handoffs/P4-E2-T3-pm-brief.md
1) Как Dev + UX — собери по AC, создай docs/handoffs/P4-E2-T3-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P4-E2-T3-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
