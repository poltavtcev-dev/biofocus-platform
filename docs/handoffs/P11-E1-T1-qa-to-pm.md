# QA → PM: P11-E1-T1

## Meta
- **Task ID:** P11-E1-T1
- **Title:** ADR-011: AI coaching polish (prompt packs + provider UX)
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P11-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg -n "ADR-011"` across decision-log, 09-api, 10-security, 16-glossary — all present.
  - Prompt packs + provider UX wording present in ADR detail + planned docs notes + glossary.
  - Rejected alternatives present: Cloud LLM by default; LLM as SoT; Auto-invoke; parallel Coach Engine; clinical tone; chat history SQLite; shipping packs without ADR (+ cloud marketplace).
  - No `CREATE TABLE … pack|chat|coach` — OK (no schema to apply).
  - `git diff --name-only -- crates/ apps/` — empty (docs-only; no migration / code).
  - Dev handoff file present.
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-011: prompt packs (named/versioned over Features / Insights / Recommendations) + provider UX; L5 interpret-only; local-first / idle / no auto-invoke | **Pass** |
| AC2 Rejected alternatives documented (cloud default; LLM SoT; auto-invoke; Coach Engine; clinical; chat history SQLite; shipping without ADR) | **Pass** |
| AC3 Schema — none to apply; in-process + env/IPC; **no migration**; deferred overrides need future ADR + approve | **Pass** — **no user schema approve required** for v1 |
| AC4 E2 packs API in `report-engine` → E3 Dashboard provider UX on `generate_report` / interpret (explicit action) | **Pass** |
| AC5 Docs touch `09-api` / `10-security` / `16-glossary` as planned/ADR notes | **Pass** |
| AC6 `docs/handoffs/P11-E1-T1-dev-to-qa.md` | **Pass** |
| Global DoD: UL terms; calm framing; UI↛DB; LLM not computing Features/Recommendations/Evidence; no Coach Engine; PR freeze | **Pass** |

- Extra checks: Out of scope respected (no packs code, no Dashboard UI, no PR). Persistence stance matches Phase 4 (env/IPC, no chat store).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move P11-E1-T1 to Done; Ready **P11-E2-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: confirm `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` reflect ADR-011 recorded
- [ ] Brief for next: `docs/handoffs/P11-E2-T1-pm-brief.md`

## Suggested next Ready task
- **P11-E2-T1** — Versioned prompt packs in `report-engine` (named pack API, default calm pack, unit tests, no SQLite / no network from builder). No schema approve gate.

## Notes for PM
- Branch: `phase/11-ai-coaching-polish`.
- Evidence: this file + `docs/handoffs/P11-E1-T1-dev-to-qa.md` + ADR-011 in `docs/decision-log.md`.
- Evolve `report-engine` + existing Report IPC — not a parallel Coach Engine.
- PR freeze still active — no PR.
