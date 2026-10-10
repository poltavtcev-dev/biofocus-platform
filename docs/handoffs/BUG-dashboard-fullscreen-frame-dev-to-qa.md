# Dev → QA: BUG-dashboard-fullscreen-frame

## Meta
- **Task ID:** BUG-dashboard-fullscreen-frame
- **Title:** Dashboard stays a small window after fullscreen close or minimize
- **Role that built:** Dev
- **Date:** 2026-10-09
- **AC source:** user bug report (not a roadmap feature; do not treat as P27-E3)

## What changed
- Closing or minimizing the dashboard while it is fullscreen or zoomed no longer leaves the AppKit frame collapsed.
- The host remembers the last windowed size (default 720×520, or the user’s resize) and puts it back on the way out of fullscreen, and again on the next show if the frame is still below the minimum.
- A visible fullscreen dashboard is only focused on reopen, so the green button is not cancelled by the menubar.
- Files: `apps/desktop/src-tauri/src/dashboard_window.rs`, `apps/desktop/src-tauri/src/lib.rs` (`open_dashboard` + setup).

## How to verify (commands)
```bash
cargo test -p desktop --lib dashboard_window
```

Manual (macOS):
1. Menubar → open Dashboard.
2. Green button → fullscreen.
3. Red close, then open Dashboard again. Window is the previous size (720×520 unless resized), not a tiny frame.
4. Repeat with the yellow minimize button, then open Dashboard again. Same restored size.
5. Fullscreen, then green button again (leave fullscreen without closing). Window returns to the previous size and stays open.
6. Resize the dashboard, fullscreen, close, reopen. The resized size comes back.

## Acceptance Criteria checklist (for QA)
- [ ] After fullscreen + close, the next Dashboard open is the last windowed size.
- [ ] After fullscreen + minimize, the next Dashboard open is the last windowed size.
- [ ] Leaving fullscreen with the green button does not hide the window.
- [ ] Reopening an already-fullscreen Dashboard only focuses it.
- [ ] Global DoD: no `unwrap`/`expect` in the prod path; UI does not touch SQLite.

## Risks / not covered
- Live traffic-light clicks are macOS-only; unit tests cover the frame state machine, not AppKit itself.
- The first red/yellow click while native fullscreen can still be consumed by macOS as “exit fullscreen”. The frame is restored; a second click closes or minimizes.

## Notes for QA
- Not Xiaomi, not P27-E3, not a new observation screen. Hide-on-close is unchanged for a normal (non-fullscreen) window.
