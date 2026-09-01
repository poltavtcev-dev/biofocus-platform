# Dev → QA: P9-E2-T1

## Meta
- **Task ID:** P9-E2-T1
- **Title:** Recommendations v1 engine path
- **Role that built:** Dev
- **Date:** 2026-08-08
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P9-E2-T1; brief `docs/handoffs/P9-E2-T1-pm-brief.md`
- **Branch:** `phase/9-recommendations`

## What changed
- **`bio-spec`:** `Recommendation` `{ id, title, suggestion, category, evidence_list }`; `EvidenceRef::Insight(InsightId)`.
- **`knowledge-engine`:** `RecommendationRule` trait; registry via `register_recommendation` / `register_recommendations_v1`; `evaluate_recommendations` (after Insights) + `evaluate_insights_and_recommendations` convenience.
- **Product rule `focus_dip_pace_hint_v1`:** emit when pattern Insight indicates Focus **lower** than recent baseline (category `pattern` + FocusScore evidence) **and** live FocusScore confidence ≥ 0.4; Evidence = Feature `FocusScore` + Insight id; calm pace/pause copy; category `pace`.
- Exhaustive `EvidenceRef` matches updated in `report-engine` + desktop IPC DTO (compile-only; no `get_recommendations` IPC yet).
- Unit tests: emit + Evidence; omit without matching Insight / higher-baseline / low confidence; clinical-copy ban; end-to-end via Insights+Recommendations.
- **No** Recommendation SQLite / busy-loop / IPC surface (→ P9-E3).

## Crates / apps / files touched
- `crates/bio-spec/src/{recommendation.rs,insight.rs,lib.rs}`, `crates/bio-spec/tests/contracts.rs`
- `crates/knowledge-engine/src/{lib.rs,engine.rs,error.rs,recommendation_rule.rs,rules/mod.rs,rules/focus_dip_pace.rs}`
- `crates/report-engine/src/builder.rs` (Insight evidence arm)
- `apps/desktop/src-tauri/src/lib.rs` (EvidenceRefDto Insight arm)
- `docs/12-development.md` (engine note)
- `docs/handoffs/P9-E2-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p bio-spec -p knowledge-engine -p report-engine
cargo check -p desktop
# Optional spot-checks
rg -n "focus_dip_pace_hint_v1|EvidenceRef::Insight|evaluate_recommendations" crates/
rg -n "CREATE TABLE.*(recommendation)" crates/ docs/ || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `bio-spec::Recommendation` + `EvidenceRef::Insight`
- [ ] AC2: `RecommendationRule` + registry; evaluate after Insights → `Result<Vec<Recommendation>>`; empty/no-match → `Ok([])`
- [ ] AC3: `focus_dip_pace_hint_v1` — Focus-below-baseline pattern Insight + FocusScore confidence gate; Evidence Feature + Insight; calm copy
- [ ] AC4: Idle/privacy — no busy-loop worker; no Recommendation SQLite; omit on thin/low-confidence
- [ ] AC5: Unit tests — emit+Evidence; omit cases; calm copy
- [ ] AC6: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; LLM not computing Recommendations

## Risks / not covered
- `get_recommendations` IPC / Dashboard UX → **P9-E3-T1**.
- Host desktop does **not** yet call `register_recommendations_v1` (no IPC consumer).
- Pattern match uses Insight `category == "pattern"` + description contains `"lower"` + FocusScore evidence (Insights do not carry rule_id today).

## Notes for QA
- Prefer `cargo test -p knowledge-engine` focus_dip / recommendations tests.
- Higher-than-baseline pattern Insight must **not** emit the pace Recommendation.
- Desktop change is only exhaustive match for `EvidenceRef` — Insights IPC still Feature/Signal only.
