# Dev → QA: P27-E1-T1

## Meta
- **Task ID:** P27-E1-T1
- **Title:** ADR-028: lock Pattern Discovery / Recommendations expansion scope (which rules, Feature inputs, non-goals)
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P27-E1-T1-pm-brief.md`
- **Branch:** `phase/27-pattern-rules`

## What changed
- **ADR-028** in `docs/decision-log.md` (summary table + detail): Phase 27 v1 locks
  - Primary = **Pattern Discovery rule expansion** (InsightRule / RecommendationRule in `knowledge-engine`)
  - Evaluate-on-read (ADR-008/009); **no** Insight/Recommendation SQLite store; personal self-tracking framing
  - **E2 slate (exactly 3):**
    1. Insight `cognitive_load_elevated_v1` ← `CognitiveLoad` (threshold sketch ≥60; omit if absent)
    2. Insight `sustained_load_elevated_v1` ← `SustainedLoadIndicator` (threshold sketch ≥60; omit if absent; distinct from CognitiveLoad / High_Stress)
    3. Recommendation `combined_demand_pace_hint_v1` ← after cognitive-load Insight (ADR-009 pairing style)
  - Justified omit of DistractionScore / DeepWork / AttentionStability / DeskAway / Circadian / Recovery / MeetingDensity-alone for this wave
  - Schema **none**; E2 register via existing helpers; E3 dogfood / Dashboard sketched
  - Rejected: Feature math; TypingRhythm; DeepFocusLikelihood; IDE; weather; App Store; Companion polish-as-primary; GPS; LLM-authored rules; workplace framing; PR freeze; migration; public launch Done / visibility flip; parallel Coach crate; >3 rules without ADR amend
- Docs planned/ADR notes: `12-development`, `16-glossary` (slate named)
- **No** rule implementation; **no** `knowledge-engine` code; **no** Feature math; **no** migration

## Crates / apps / files touched
- Docs only (no `crates/` / `apps/` for this task).
- Branch: `phase/27-pattern-rules`

## How to verify (commands)
```bash
rg -n "ADR-028" docs/decision-log.md docs/12-development.md docs/16-glossary.md

rg -n "cognitive_load_elevated_v1|sustained_load_elevated_v1|combined_demand_pace_hint_v1|CognitiveLoad|SustainedLoadIndicator" \
  docs/decision-log.md docs/12-development.md docs/16-glossary.md

rg -n "TypingRhythm|DeepFocusLikelihood|LLM-authored|workplace|PR during freeze|migration|public launch Done|parallel Coach|Feature catalog math" \
  docs/decision-log.md

# No rule impl yet
rg -n "cognitive_load_elevated_v1|sustained_load_elevated_v1|combined_demand_pace_hint_v1" \
  crates/knowledge-engine || true
# expect: no matches (impl → P27-E2)

git diff --name-only -- crates/ apps/
# expect: empty for this task
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-028 Phase 27 primary = rule expansion; evaluate-on-read; no Insight/Rec SQLite; personal framing
- [ ] AC2: Explicit E2 slate 1–3 rules with Feature inputs, trigger/omit/Evidence/calm copy; omit of others justified
- [ ] AC3: Rejected alternatives documented
- [ ] AC4: Schema none — no migration / Observation / Feature formulas
- [ ] AC5: E2 register helpers + E3 dogfood sketched
- [ ] AC6: Docs planned/ADR notes (`12` / `16`)
- [ ] AC7: Handoff present
- [ ] Global DoD: no impl in E1; PR freeze; public launch not Done; no parallel crate

## Risks / not covered
- Exact thresholds (±10 band) and Dashboard category strings deferred to E2 within ADR locks.
- DistractionScore / DeepWork waves deferred — not defects for E1.

## Notes for QA
- Do **not** expect new rules in `knowledge-engine` yet — that is **P27-E2**.
- Kanban Done / canvas are PM-only after QA Pass.
- Unrelated dirty Phase 26 docs may exist on the branch — out of AC unless they contradict ADR-028.
