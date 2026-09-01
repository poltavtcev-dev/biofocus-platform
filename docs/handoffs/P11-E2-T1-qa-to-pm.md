# QA → PM: P11-E2-T1

## Meta
- **Task ID:** P11-E2-T1
- **Title:** Versioned prompt packs in report-engine
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P11-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p report-engine` → **20 passed** (7 pack + prior builder/llm).
  - `cargo check -p report-engine -p desktop` → **ok**.
  - `build_report_with_pack` / `biofocus.default` / `UnknownPromptPack` present in crate + `09-api` / glossary / security.
  - Default `llm_prompt` forbids inventing metrics, Evidence, Insights, Recommendations, or actions.
  - No pack/chat/coach `CREATE TABLE` schema.
  - `unwrap`/`expect` only in `#[cfg(test)]` helpers — OK.
  - Dev handoff present.
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Public API by `id`+`version` → offline markdown/`llm_prompt` from Features/Insights/Recommendations; `Result`; no Feature/Recommendation math | **Pass** |
| AC2 Default `biofocus.default` / `1`; calm; forbid invent metrics/Evidence/Insights/Recommendations/actions | **Pass** |
| AC3 Empty/partial soft Ok; tests: selection, empty/partial, default registered | **Pass** |
| AC4 No SQLite; no network from pack builder; no Dashboard UI (`build_report` retained) | **Pass** |
| AC5 `09-api` pack API finalized; glossary/security (and `12-development`) updated | **Pass** |
| AC6 `docs/handoffs/P11-E2-T1-dev-to-qa.md` | **Pass** |
| Global DoD: no prod unwrap; UI↛DB; UL; interpret-only; no Coach Engine; PR freeze | **Pass** |

- Extra checks: Desktop `calm_llm_error` exhaustiveness for `UnknownPromptPack` compiles; host still on Phase 4 `build_report` until E3 (documented).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move P11-E2-T1 to Done; Ready **P11-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` (packs shipped)
- [ ] Brief for next: `docs/handoffs/P11-E3-T1-pm-brief.md`

## Suggested next Ready task
- **P11-E3-T1** — Local LLM provider UX + pack-aware Report flow (Dashboard status IPC, explicit Generate report with pack, soft-fail `llmStatus`).

## Notes for PM
- Branch: `phase/11-ai-coaching-polish`.
- Evidence: this file + `docs/handoffs/P11-E2-T1-dev-to-qa.md` + `cargo test -p report-engine`.
- Default pack ready for E3 wiring; no schema approve gate.
- PR freeze still active — no PR.
