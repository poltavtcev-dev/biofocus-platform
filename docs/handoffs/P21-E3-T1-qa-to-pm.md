# QA → PM: P21-E3-T1

## Meta
- **Task ID:** P21-E3-T1
- **Title:** Dogfood notes + optional calm Dashboard surface for `DeepWorkScore`
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P21-E3-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - Dogfood § **DeepWorkScore dogfood** present in `docs/12-development.md` (Focus required / Focus-only / omit without Focus / fixture + live IPC / UI ↛ SQLite)
  - Calm framing: “sustained focus in this window” / chart label **Sustained focus**; clinical ban stated in dogfood
  - Chart allowlist + mock ready include `DeepWorkScore`; `npx tsc --noEmit` ok
  - `cargo test -p feature-engine deep_work_score` — **6/6 ok** (math untouched this task)
  - `git diff --name-only -- crates/feature-engine/` — empty (no formula rewrite)
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Dogfood notes + verify paths + Focus-only / omit notes | **Pass** |
| AC2 Calm framing only (docs + UI) | **Pass** |
| AC3 Optional Dashboard — Sustained focus when present; omit quiet; paths stated | **Pass** |
| AC4 Smoke notes; UI ↛ SQLite | **Pass** |
| AC5 No formula rewrite / migration / new Observation / Insights rule | **Pass** |
| AC6 Dev handoff present | **Pass** |
| Global DoD | **Pass** |

- Extra: Snapshot list path already worked via IPC; chart surface is thin allowlist reuse (not a redesign).

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P21-E3-T1** to Done; close Epic **P21-E3** / **Phase 21** if no further P21 tasks
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` — Phase 21 complete
- [ ] Open **PM-GATE-POST-P21** (IDE · weather · App Store · polish · AttentionStability · CircadianOffset) — **no PR** during freeze
- [ ] Write gate brief / next Ready as needed

## Suggested next Ready task
- **PM-GATE-POST-P21** — choose next Phase 22+ slice.

## Notes for PM
- Phase 21 math was Done at E2; E3 is dogfood + calm chart label only.
- PR freeze until 2026-09-01 — no PR.
