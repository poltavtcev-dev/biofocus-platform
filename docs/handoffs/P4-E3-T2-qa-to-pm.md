# QA → PM: P4-E3-T2

## Meta
- **Task ID:** P4-E3-T2
- **Title:** Optional local LLM adapter
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P4-E3-T2-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
### Commands
```bash
cargo test -p report-engine   # 13 passed (builder + llm mock HTTP)
cargo check -p report-engine  # ok
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Opt-in env/config; default OFF (no network when disabled) | **Pass** — `from_env_lookup` default disabled; `disabled_returns_error_without_http` (`hits == 0`) |
| AC2 Localhost Ollama / OpenAI-compatible; configurable base + model | **Pass** — default `http://127.0.0.1:11434/v1` + `llama3.2`; env overrides tested |
| AC3 Consumes `llm_prompt`; `Result`; no Feature math | **Pass** — `interpret_report` uses `document.llm_prompt`; mock asserts `messages` present / `features` absent |
| AC4 HTTP timeout; never auto-send on startup | **Pass** — `timeout_maps_to_typed_error`; no desktop wiring / no startup call; docs state explicit-only |
| AC5 `thiserror`; no prod `unwrap`/`expect` | **Pass** — variants on `ReportEngineError`; unwrap/expect only in `#[cfg(test)]` |
| AC6 Privacy note in `docs/12-development.md` | **Pass** — opt-in / local-only preference / what leaves the machine |
| AC7 Handoff with tests + how to enable | **Pass** — `P4-E3-T2-dev-to-qa.md` |
| Global DoD / out of scope | **Pass** — T3 UX untouched; no SQLite; Feature math not in LLM path |

### Extra checks
- Desktop has **no** `report_engine` / `interpret_*` calls (correct for T3).
- Fixed during QA: duplicate `### Optional local LLM` block in `docs/10-security.md` and restored Input-aggregates table adjacency.

## Defects (if any)
- None blocking. Docs duplicate in `10-security.md` fixed in-tree before this report.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P4-E3-T2 → Done; Ready → **P4-E3-T3**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs: `docs/ARCHITECTURE_STATUS.md` / status line in `docs/12-development.md` / `docs/14-roadmap.md` as needed
- [ ] Note public surface for canvas: `LocalLlmConfig`, `interpret_report` / `interpret_llm_prompt`, env `BIOFOCUS_LOCAL_LLM*`

## Suggested next Ready task
- **P4-E3-T3** — Report UX in Dashboard (UX + Dev): wire `build_report` (+ optional interpret) behind explicit user action; no auto-send.

## Notes for PM
- Placement: **`crates/report-engine`** (not host).
- Enable for manual smoke: `BIOFOCUS_LOCAL_LLM=1` (+ optional base/model/timeout); automated Pass used in-process mock, not live Ollama.
- Branch remains `phase/4-dashboard-ai`.
