# Dev → QA: P9-E1-T1

## Meta
- **Task ID:** P9-E1-T1
- **Title:** ADR-009: Recommendations domain / engine shape
- **Role that built:** Dev
- **Date:** 2026-08-08
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P9-E1-T1; brief `docs/handoffs/P9-E1-T1-pm-brief.md`
- **Branch:** `phase/9-recommendations`

## What changed
- **ADR-009** in `docs/decision-log.md`: Recommendations v1 = first-class `Recommendation` + `RecommendationRule` hosted in `knowledge-engine` (evaluate-on-read after Insights); keep thin `Insight.actionRecommendation` as optional hint only; **no** Recommendation SQLite store; **no** parallel Coach Engine crate.
- Relationship table: Observations → Features/Signals → Insights (L3) → Recommendations (L4) with own Evidence (Feature / Signal / Insight ids).
- Idle/privacy: local-only; no busy-loop / always-on recommendation worker; thin/low-confidence → omit.
- Rejected: LLM as SoT for actions; clinical/prescription framing; parallel Coach Engine; cloud sync; persist Recommendation history in v1; thin-text-only L4; replace Insights with Recommendations.
- Schema: **none to apply** — deferred dismiss/feedback history only via future ADR + user approve.
- E2 sketch: `focus_dip_pace_hint_v1` (pattern Focus-below-baseline Insight + FocusScore confidence → calm pace/pause Recommendation with Evidence).
- Glossary + domain + storage/pipeline/API/vision/dev notes updated for `Recommendation`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-009 summary + detail)
- `docs/16-glossary.md`, `docs/02-domain-model.md`
- `docs/04-storage.md`, `docs/05-pipeline.md`
- `docs/09-api.md` (`get_recommendations` contract sketch)
- `docs/00-vision.md` (L4 row + UL chain)
- `docs/12-development.md` (ADR-009 note)
- `docs/handoffs/P9-E1-T1-dev-to-qa.md` (this file)
- (pre-existing Phase 9 PM open docs on tree — not authored in this Dev pass)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-009 present and no migration SQL applied
rg -n "ADR-009" docs/decision-log.md docs/02-domain-model.md docs/04-storage.md docs/05-pipeline.md docs/09-api.md docs/16-glossary.md docs/00-vision.md docs/12-development.md
rg -n "focus_dip_pace_hint_v1" docs/decision-log.md docs/09-api.md
rg -n "CREATE TABLE.*(recommendation)" docs/decision-log.md docs/04-storage.md || true
# No crate changes expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-009 in `docs/decision-log.md` — chosen approach (first-class Recommendation + RecommendationRule in knowledge-engine), relationship to Features / Insights / Evidence, evaluate-on-read vs persist, idle/privacy (local-only, no busy-loop)
- [ ] AC2: Rejected alternatives documented (LLM as SoT; clinical framing; parallel Coach Engine; cloud sync; …)
- [ ] AC3: Schema — none to apply; deferred history only after future ADR + user approve; no migration in repo from this task
- [ ] AC4: Short sketch ≥1 calm deterministic recommendation for E2 (`focus_dip_pace_hint_v1`; Feature/Insight Evidence; non-clinical copy)
- [ ] AC5: Glossary / domain note for `Recommendation` updated (not deferred thin-text-only)
- [ ] AC6: This handoff exists
- [ ] Global DoD: glossary terms; calm non-clinical framing; UI↛DB; no new parallel crate; LLM not computing Recommendations

## Risks / not covered
- Implementing `Recommendation` type / `RecommendationRule` / evaluator → **P9-E2-T1** (out of scope).
- IPC / Dashboard Recommendations surface → **P9-E3-T1** (out of scope).
- Extending `EvidenceRef` with `Insight` variant is a contract for E2 — not coded here.
- If dogfood later wants dismiss history, needs a **new** ADR + approve — not silently added.

## Notes for QA
- Decision is explicitly **first-class Recommendation + evaluate-on-read**, not “evolve thin text only” and not a new Coach Engine crate.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- Calm copy examples in ADR / API sketch must not read as clinical advice.
