# QA → PM: P20-E3-T1

## Meta
- **Task ID:** P20-E3-T1
- **Title:** Dogfood notes + optional calm Dashboard surface for `CognitiveLoad`
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P20-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - Dogfood § **CognitiveLoad dogfood** present in `docs/12-development.md` (fixture + live IPC + partial emit note + UI ↛ SQLite)
  - Calm framing: “combined demand in this window” / chart label **Combined demand**; clinical ban stated in dogfood
  - Chart allowlist + mock ready include `CognitiveLoad`; `npx tsc --noEmit` ok
  - `cargo test -p feature-engine cognitive_load` — **6/6 ok** (math untouched this task)
  - `git diff --name-only -- crates/feature-engine/` — empty (no formula rewrite)
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Dogfood notes + verify paths + partial emit | **Pass** |
| AC2 Calm framing only (docs + UI) | **Pass** |
| AC3 Optional Dashboard — Combined demand when present; omit quiet; paths stated | **Pass** |
| AC4 Smoke notes; UI ↛ SQLite | **Pass** |
| AC5 No formula rewrite / migration / new Observation / Insights rule | **Pass** |
| AC6 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: Snapshot list path already worked via IPC; chart surface is thin allowlist reuse (not a redesign).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P20-E3-T1** to Done; close Epic **P20-E3** / **Phase 20** if no further P20 tasks
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — Phase 20 complete
- [ ] Open **PM-GATE-POST-P20** (IDE · weather · App Store · polish) — **no PR** during freeze
- [ ] Write gate brief / next Ready as needed

## Suggested next Ready task
- **PM-GATE-POST-P20** — choose next Phase 21+ slice (IDE · weather · App Store · polish).

## Notes for PM
- Phase 20 math was Done at E2; E3 is dogfood + calm chart label only.
- PR freeze until 2026-09-01 — no PR.
- Unrelated dirty E2-close roadmap docs may already be on the branch — fold into PM close as needed.
