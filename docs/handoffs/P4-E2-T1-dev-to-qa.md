# Dev → QA: P4-E2-T1

## Meta
- **Task ID:** P4-E2-T1
- **Title:** knowledge-engine skeleton + Insight types
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P4-E2 / P4-E2-T1; brief `docs/handoffs/P4-E2-T1-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- `crates/knowledge-engine` left stub-only: pluggable `InsightRule` + `KnowledgeEngine::evaluate(features, signals) → Result<Vec<Insight>>`.
- Domain types reused from `bio-spec` (`Insight`, `EvidenceRef`, `Feature`, `Signal`) — no parallel metric model.
- Errors via `thiserror` (`KnowledgeEngineError`: duplicate rule / rule failed). Empty rules / empty input / no-match → `Ok([])`.
- Unit tests: happy path (≥1 Insight with Feature+Signal Evidence), empty input / no-match, duplicate id, rule failure. Product rules deferred to T2 (test uses `ScaffoldEchoRule` only under `#[cfg(test)]`).
- No SQLite, UI, LLM, persistence schema.

### Crates / files
- `crates/knowledge-engine/Cargo.toml` — `thiserror`; `uuid` as dev-dep
- `crates/knowledge-engine/src/lib.rs`
- `crates/knowledge-engine/src/error.rs`
- `crates/knowledge-engine/src/rule.rs`
- `crates/knowledge-engine/src/engine.rs`

## How to verify (commands)
```bash
cargo test -p knowledge-engine
cargo check -p knowledge-engine
```

Expected: 7 unit tests pass; crate checks clean.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: crate not stub-only; `Insight` + Evidence (`EvidenceRef`) reference Feature and/or Signal ids
- [ ] AC2: public API Features + Signals → `Result<Vec<Insight>>` (empty `Ok` valid)
- [ ] AC3: `thiserror`; no production `unwrap` / `expect`
- [ ] AC4: unit tests happy path (≥1 Insight) + empty / no-match → empty vec
- [ ] AC5: no SQLite, UI, LLM, new persistence schema
- [ ] AC6: this handoff includes `cargo test -p knowledge-engine`
- [ ] Global DoD: glossary terms; UI↛DB; Core-only compute

## Risks / not covered
- No product rules yet (intentional → **P4-E2-T2**). Default `generate_insights` / empty engine always returns `[]`.
- No IPC / Dashboard list (→ **P4-E2-T3**).
- `ScaffoldEchoRule` is test-only; not shipped as a product rule.

## Notes for QA
- Grep prod paths under `crates/knowledge-engine/src` for `unwrap`/`expect` (tests may use them).
- Confirm `EvidenceRef::Feature` / `EvidenceRef::Signal` used in happy-path evidence_list.
