# Dev → QA: P4-E2-T2

## Meta
- **Task ID:** P4-E2-T2
- **Title:** Rule Insights v1
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P4-E2-T2; brief `docs/handoffs/P4-E2-T2-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- ≥2 deterministic product `InsightRule`s + host helper `register_insights_v1`:
  1. `high_stress_period_v1` — Signal `High_Stress` → calm Insight; evidence = Signal id(s) + optional `StressIndex` Feature.
  2. `context_switch_elevated_v1` — latest `ContextSwitchRate` ≥ `1.0` → calm Insight; evidence = CSR Feature + optional `FocusScore`.
- Empty / unregistered `KnowledgeEngine::new()` still returns `Ok([])`; host must call `register_insights_v1` (documented on crate + `rules` module).
- Unit tests: trigger paths, no-trigger / empty, both rules, calm-copy smoke (banned clinical terms).
- No persistence ADR, no UI/IPC, no LLM, no feature-engine dependency (string id contracts only).

## Crates / files touched
- `crates/knowledge-engine/Cargo.toml` — `uuid` runtime dep (Insight ids)
- `crates/knowledge-engine/src/lib.rs` — re-exports + entrypoint docs
- `crates/knowledge-engine/src/engine.rs` / `rule.rs` — comment pointers to v1 register
- `crates/knowledge-engine/src/rules/mod.rs` — `register_insights_v1` + tests
- `crates/knowledge-engine/src/rules/high_stress.rs`
- `crates/knowledge-engine/src/rules/context_switch.rs`

## How to verify (commands)
```bash
cargo test -p knowledge-engine
```

Expected: 15 unit tests passed (engine skeleton + rules v1).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ≥2 deterministic product rules registered via `register_insights_v1` (High_Stress + elevated ContextSwitch / Focus pattern)
- [ ] AC2: Insight copy calm, non-evaluative / non-clinical
- [ ] AC3: `evidence_list` uses `EvidenceRef::Feature` and/or `EvidenceRef::Signal`
- [ ] AC4: Unit tests cover trigger + no-trigger / empty → no spurious Insights
- [ ] AC5: No persistence ADR, no UI/IPC, no LLM rewrite
- [ ] AC6: Handoff + `cargo test -p knowledge-engine` green
- [ ] Global DoD: no `unwrap`/`expect` in production paths; UI↛DB; glossary terms

## Host registration (for QA / next IPC task)
```rust
let mut engine = KnowledgeEngine::new();
register_insights_v1(&mut engine)?;
let insights = engine.evaluate(&features, &signals)?;
```

## Risks / not covered
- Thresholds (`CONTEXT_SWITCH_ELEVATED_THRESHOLD = 1.0`) are v1 product guesses — not calibrated on user data.
- No integration with Feature Worker / desktop yet (→ P4-E2-T3).
- `StressIndex` / `FocusScore` are optional evidence only; rules do not re-compute Features.

## Notes for QA
- Scaffold `ScaffoldEchoRule` remains `#[cfg(test)]` only in `engine.rs` — not a product rule.
- Confirm no new SQLite migrations / desktop IPC in this diff.
