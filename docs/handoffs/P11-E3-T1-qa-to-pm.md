# QA → PM: P11-E3-T1

**From:** QA  
**To:** PM  
**Date:** 2026-08-10  
**Dev/UX handoff:** `docs/handoffs/P11-E3-T1-dev-to-qa.md`  
**Verdict:** Pass with notes

## What was verified

### Commands
```text
cargo test -p desktop --lib assemble_report
→ 3 passed (disabled offline, features markdown, Recommendations section)

cargo test -p desktop --lib local_llm_provider
→ 3 passed (disabled / ready / bad-url error; no token/Bearer in JSON)

cd apps/desktop && pnpm exec tsc --noEmit
→ ok
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Calm provider status via IPC (`disabled`/`ready`/`error`); UI↛SQLite; no secrets | **Pass** — `get_local_llm_status` + Dashboard status row; model id only when ready; pack meta; mocks `mockLlmStatus` |
| AC2 `build_report_with_pack` default pack + Features/Insights/Recommendations; explicit Generate only | **Pass** — host wires pack + evaluates Recommendations; UI invokes only on button; poll fetches status only |
| AC3 LLM off → offline markdown + `llm_prompt`; calm optional copy | **Pass** — `llmStatus: disabled`; ReportSlot + `llmStatusDetail` copy |
| AC4 LLM on → soft-fail without losing markdown | **Pass** — Phase 4 soft-fail path retained in `assemble_report_dto` |
| AC5 Non-clinical; no diagnosis / surveillance / marketplace | **Pass** — calm Local AI copy; pack disclaimer only |
| AC6 Docs + mocks | **Pass** — `09-api`, `12-development`, `10-security`, glossary, desktop README |
| Global DoD | **Pass** — no new prod `unwrap`/`expect`; UI↛DB |

### Extra checks
- Soft refresh (~30s) reloads snapshot/insights/recommendations/status — **never** `generate_report`.
- Status DTO serialization omits tokens; config-only (no HTTP probe on open) — matches ADR honesty.
- Code review: invoke handler registers `get_local_llm_status`; Generate path imports `DEFAULT_PROMPT_PACK_*`.

## Defects
None blocking.

## Notes
- Live Ollama smoke not run in this QA pass (optional; same note as P4-E3-T3). Soft-fail + disabled/ready/error mapping covered by unit tests.
- Provider `ready` means env/config present, not that the endpoint answered — intentional per brief.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — mark **P11-E3-T1** Done; close Epic **P11-E3** and **Phase 11** Kanban if no further P11 tasks
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS` Phase 11 → closed; next horizon Phase 12+
- [x] **Do not open a PR** (PR freeze until 2026-09-01)

## Suggested next Ready task
- Phase 12+ gate via separate PM brief (ambient sources / commercial packaging per roadmap) — **not** auto-start in this chat.

## Notes for PM
- Public IPC additions: `get_local_llm_status` → `{ status, detail, model?, packId, packVersion }`; `generate_report` now pack-aware (`biofocus.default` @ `1` + Recommendations).
- Branch remains `phase/11-ai-coaching-polish` (commit/PR later per freeze policy).
