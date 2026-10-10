# QA → PM: BUG-dashboard-fullscreen-frame

## Meta
- **Task ID:** BUG-dashboard-fullscreen-frame
- **Title:** Dashboard stays a small window after fullscreen close or minimize
- **Date:** 2026-10-09
- **Dev/UX handoff:** `docs/handoffs/BUG-dashboard-fullscreen-frame-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results: `cargo test -p desktop --lib dashboard_window` — 9 passed, 0 failed.
- AC results:
  - Fullscreen close restores the last windowed size before hide — **Pass** (state machine).
  - Fullscreen exit into the minimum or a tiny frame restores the larger size — **Pass**.
  - Windowed close still hides immediately — **Pass**.
  - Green-button exit at the same size does not hide — **Pass**.
  - Reopen of a visible fullscreen window does not force a restore — **Pass**.
  - Hidden fullscreen and a visible collapsed frame repair on show — **Pass**.
  - No prod `unwrap`/`expect` on the new path — **Pass** (poisoned mutex uses `into_inner`).
- Extra checks: UI still reaches the dashboard only through `open_dashboard`; no SQLite from the window hook.

## Defects (if any)
- None in the state machine. Live red/yellow clicks on a running macOS window were not driven in this pass.

## What PM must update
- [x] Do **not** mark P27-E3 Done and do **not** treat this bug as that task. The observation rules and screen are already on `main`.
- [x] Optional note only: dashboard fullscreen close/minimize restores the windowed frame (`dashboard_window.rs`).
- [x] Execution canvas: no queue change for this bug unless the board is being rewritten to match `main`.

## Suggested next Ready task
- Not this bug. Companion queue still reads batches up to 8000 rows if the memory window returns. Xiaomi ZIP / Health Connect stay deferred until a separate decision.

## Notes for PM
- Roadmap was not edited. `p0`–`p6` worktrees were not used.
