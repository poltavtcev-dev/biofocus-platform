# Dev → QA: P27-E2-T1

## Meta
- **Task ID:** P27-E2-T1
- **Title:** Ship locked Insight/Recommendation rules in `knowledge-engine` per ADR-028
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P27-E2-T1-pm-brief.md` · `/docs/SPRINT_ROADMAP.md` Phase 27 · ADR-028
- **Branch:** `phase/27-pattern-rules`
- **Code:** committed in `a7f5971` (re-verified this build-qa chat)

## What changed
- Implemented + registered **exactly** the ADR-028 E2 slate in `knowledge-engine`:
  1. InsightRule `cognitive_load_elevated_v1` — latest `CognitiveLoad` ≥ **60**; category `demand`; optional Evidence `MeetingDensity` / `ContextSwitchRate` / `NotificationPressure`; calm “Combined demand looked elevated in this window”.
  2. InsightRule `sustained_load_elevated_v1` — latest `SustainedLoadIndicator` ≥ **60**; category `prolonged_load`; optional Evidence `StressIndex` / `FatigueIndex` / `MeetingDensity`; calm “Prolonged load looked elevated in this window”; distinct from High_Stress / cognitive-load.
  3. RecommendationRule `combined_demand_pace_hint_v1` — emit when demand Insight present (ADR-009 style match); Evidence = Insight id (+ `CognitiveLoad` when present); gentle pace/pause copy.
- Wired into existing `register_insights_v1` (now **5** Insight rules) / `register_recommendations_v1` (now **2** Recommendation rules). Existing v1 rules not rewritten beyond shared helper registration.
- Unit tests: emit / omit / Evidence / calm copy / threshold band / Recommendation only with cognitive-load Insight / no Feature rewrite path.
- Docs: `12-development`, `16-glossary`, `09-api` → E2 **shipped** notes for the three rule ids.
- **No** migration / new Observation / Feature catalog math / Insight SQLite store.

## Crates / files touched
| Path | Change |
| :--- | :--- |
| `crates/knowledge-engine/src/rules/cognitive_load_elevated.rs` | **new** — InsightRule + tests |
| `crates/knowledge-engine/src/rules/sustained_load_elevated.rs` | **new** — InsightRule + tests |
| `crates/knowledge-engine/src/rules/combined_demand_pace.rs` | **new** — RecommendationRule + tests |
| `crates/knowledge-engine/src/rules/mod.rs` | register + integration tests; table docs |
| `crates/knowledge-engine/src/lib.rs` | exports + crate docs |
| `crates/knowledge-engine/Cargo.toml` | `serde_json` dev-dep (non-scalar tests) |
| `docs/12-development.md` | Phase 27 E2 ship notes |
| `docs/16-glossary.md` | Pattern Discovery E2 shipped |
| `docs/09-api.md` | Insights/Recommendations shipped rule notes |

## How to verify (commands)
```bash
cargo test -p knowledge-engine

# Rule ids present; no Feature formula rewrite in knowledge-engine
rg -n "cognitive_load_elevated_v1|sustained_load_elevated_v1|combined_demand_pace_hint_v1" \
  crates/knowledge-engine docs/12-development.md docs/16-glossary.md docs/09-api.md

# Confirm Feature catalog untouched this task
rg -n "CognitiveLoadNode|SustainedLoadIndicator|register_sustained_load|register_cognitive" \
  crates/knowledge-engine
# (expect Feature *ids* as string inputs only — no FeatureEngine register helpers)
```

Dev local (this chat): `cargo test -p knowledge-engine` → **46 passed**.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Exactly three new rules registered via `register_insights_v1` / `register_recommendations_v1` with locked triggers, Evidence, calm copy
- [ ] AC2: No rule outside locked slate
- [ ] AC3: Existing v1 rules not rewritten beyond shared registration wiring
- [ ] AC4: Unit tests cover emit / omit / Evidence / calm copy / threshold / Rec only with demand Insight / no Feature rewrite
- [ ] AC5: No migration / no new Observation / no Feature catalog math / no Insight-Rec SQLite store
- [ ] AC6: Docs 12 / 16 (+ knowledge notes) reflect E2 ship
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; Pattern rules DoD

## Risks / not covered
- Dashboard category labels for `demand` / `prolonged_load` may fall back to raw category until **P27-E3** surface polish.
- Host already calls `register_insights_v1` / `register_recommendations_v1` — new rules pick up automatically; no desktop IPC change this task.
- Unrelated dirty Phase 26 / PM-close docs may exist on the branch — out of AC unless they contradict ADR-028 ship.

## Notes for QA
- Thresholds fixed at **60** (within ADR ±10 band).
- Recommendation matching is ADR-009 style (category `demand` + “combined demand” / “elevated” copy + CognitiveLoad Evidence) — same approach as `focus_dip_pace_hint_v1` (no Insight.rule_id field).
- Do **not** mark Done / canvas — that is PM after your qa-to-pm.
