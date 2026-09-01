# Dev → QA: P11-E2-T1

## Meta
- **Task ID:** P11-E2-T1
- **Title:** Versioned prompt packs in report-engine
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P11-E2-T1; brief `docs/handoffs/P11-E2-T1-pm-brief.md`
- **Branch:** `phase/11-ai-coaching-polish`

## What changed
- Public pack API in `crates/report-engine`: `build_report_with_pack(id, version, features, insights, recommendations) → Result<ReportDocument>` per ADR-011.
- Default pack `biofocus.default` @ `1`: calm / non-clinical; `llm_prompt` forbids inventing metrics, Evidence, Insights, Recommendations, or actions.
- In-process registry helpers: `list_prompt_packs`, `default_prompt_pack`, `PromptPackRef`, constants.
- Empty / partial Evidence → soft empty sections, `Ok`; unknown id/version → `ReportEngineError::UnknownPromptPack`.
- Unit tests for pack selection, empty/partial inputs, recommendations-only, stability/sort.
- Phase 4 `build_report` kept for existing host; Desktop match arm updated for new error variant (no UI / no pack wiring yet).
- Docs: `09-api` pack API finalized; glossary / security / `12-development` updated from planned → shipped.
- **No SQLite schema / no migration / no network from pack builder / no Dashboard UI.**

## Crates / apps / files touched
- `crates/report-engine/src/packs.rs` (new)
- `crates/report-engine/src/lib.rs`
- `crates/report-engine/src/builder.rs` (shared section helpers)
- `crates/report-engine/src/error.rs` (`UnknownPromptPack`)
- `crates/report-engine/Cargo.toml`
- `apps/desktop/src-tauri/src/lib.rs` (exhaustive `calm_llm_error` match)
- `docs/09-api.md`, `docs/10-security.md`, `docs/16-glossary.md`, `docs/12-development.md`
- `docs/handoffs/P11-E2-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p report-engine
cargo check -p report-engine -p desktop
rg -n "build_report_with_pack|biofocus.default|UnknownPromptPack" crates/report-engine docs/09-api.md docs/16-glossary.md docs/10-security.md
# No migration SQL
rg -n "CREATE TABLE.*(pack|chat|coach)" docs/ crates/report-engine || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Public API select/build by `id` + `version` → offline `markdown` + `llm_prompt` from Features / Insights / Recommendations; `Result`; no Feature/Recommendation math in pack builder
- [ ] AC2: Default pack `biofocus.default` / `1`; calm; `llm_prompt` forbids inventing metrics, Evidence, Insights, Recommendations, or actions
- [ ] AC3: Empty/partial → soft Ok; unit tests cover selection + empty/partial + default present
- [ ] AC4: No SQLite schema; no network from pack builder; no Desktop UI (host may keep `build_report`)
- [ ] AC5: Docs finalize pack API in `09-api`; glossary / security updated from planned
- [ ] AC6: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms; LLM interpret-only; no Coach Engine; PR freeze

## Risks / not covered
- Dashboard provider status / pack picker → **P11-E3-T1** (out of scope).
- Host `generate_report` still uses Phase 4 `build_report` until E3 wires packs.
- On-disk editable packs / chat history → future ADR + approve.
- Only one pack shipped in v1 registry (by design).

## Notes for QA
- Prefer `cargo test -p report-engine` (20 tests expected: prior 13 + 7 pack tests).
- Confirm default `llm_prompt` contains explicit forbid of inventing Recommendations/actions, not only metrics.
- Spot-check Desktop still compiles after `UnknownPromptPack` arm.
