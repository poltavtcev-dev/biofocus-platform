# Dev → QA: P8-E1-T1

## Meta
- **Task ID:** P8-E1-T1
- **Title:** ADR-008: Pattern Discovery history / recompute
- **Role that built:** Dev
- **Date:** 2026-08-07
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P8-E1-T1; brief `docs/handoffs/P8-E1-T1-pm-brief.md`
- **Branch:** `main` (per user; Phase 8 docs cluster)

## What changed
- **ADR-008** in `docs/decision-log.md`: Pattern Discovery v1 = **recompute-on-read** multi-window Features from local Observations (+ optional in-process memo); evolve `knowledge-engine`; **no** Feature/baseline history SQLite table in v1.
- Relationship table: Observations (durable) → Features (derived / live cache + recomputed series) → Insights (Knowledge rules).
- Idle/privacy: no always-on worker / busy-loop; local-only; thin history → omit Insight.
- Rejected: persisted Feature history for v1; cloud sync of patterns; ML training; parallel Correlation Engine crate; always-on recompute worker.
- Schema: **none proposed for apply** — deferred daily rollup only via future ADR + user approve.
- Knowledge consumption sketch: `focus_vs_recent_baseline_v1` (current FocusScore vs ≤7 prior comparable windows; confidence gate; calm copy) — implement → **P8-E2**.
- Notes: `02-domain-model`, `04-storage`, `05-pipeline`, `09-api` (`get_insights`), `16-glossary`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-008 summary + detail)
- `docs/02-domain-model.md`, `docs/04-storage.md`, `docs/05-pipeline.md`
- `docs/09-api.md`, `docs/16-glossary.md`
- `docs/handoffs/P8-E1-T1-dev-to-qa.md` (this file)
- (pre-existing Phase 8 PM open docs on tree — not authored in this Dev pass)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-008 present and no migration SQL applied
rg -n "ADR-008" docs/decision-log.md docs/02-domain-model.md docs/04-storage.md docs/05-pipeline.md docs/09-api.md docs/16-glossary.md
rg -n "CREATE TABLE.*(feature|baseline)" docs/decision-log.md docs/04-storage.md || true
# No crate changes expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-008 in `docs/decision-log.md` — chosen approach (recompute-on-read), relationship to Observations + Features + Knowledge Insights, idle/privacy (local-only, no busy-loop)
- [ ] AC2: Rejected alternatives documented (cloud sync, ML training, Correlation Engine crate, persisted Feature history for v1, always-on worker)
- [ ] AC3: Schema — none to apply; deferred rollup only after future ADR + user approve; no migration in repo from this task
- [ ] AC4: Knowledge consumption sketch for ≥1 calm baseline Insight (`focus_vs_recent_baseline_v1` / contracts note in ADR + `09-api`)
- [ ] AC5: This handoff exists
- [ ] Global DoD: glossary terms; calm non-clinical framing; UI↛DB; no new parallel crate

## Risks / not covered
- Implementing baseline Insight rule / Observation load path → **P8-E2-T1** (out of scope).
- If dogfood later proves recompute too slow, a rollup table needs a **new** ADR + approve — not silently added.
- `04-storage.md` / `05-pipeline.md` remain partially stub-wrapped (pre-existing); ADR notes appended after existing content.

## Notes for QA
- Decision is explicitly **recompute-on-read**, not hybrid-with-SQLite. Optional in-process memo is allowed and is **not** persistence.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- Calm copy examples in ADR must not read as clinical diagnosis.
