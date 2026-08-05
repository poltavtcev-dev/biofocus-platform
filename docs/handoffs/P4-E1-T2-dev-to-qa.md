# UX → QA: P4-E1-T2

## Meta
- **Task ID:** P4-E1-T2
- **Title:** Dashboard shell (window / route)
- **Role that built:** UX (+ thin Tauri host for window open)
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P4-E1-T2 · brief `docs/handoffs/P4-E1-T2-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## Design choice (document for AC1)
**Separate Tauri window** (not in-app route inside the compact Menubar).

| Surface | Window label | URL |
| :--- | :--- | :--- |
| Menubar / tray shell | `main` | default |
| Dashboard | `dashboard` | `index.html?view=dashboard` |

- Menubar button **Open Dashboard** → `invoke("open_dashboard")` (show / focus; hide-on-close).
- Vite preview without Tauri: same button sets `?view=dashboard` in the current tab.
- Closing Dashboard hides the window (does not destroy) so reopen stays cheap.

## What changed
- Dashboard shell: calm **loading / empty / error / ready** from IPC `get_feature_snapshot`.
- Empty chart slot + Insights placeholder (no Recharts).
- Ready state: simple Feature list (ids + values) — layout only; series → T3.
- Menubar alert indicator + 5s status poll unchanged.
- Soft snapshot refresh ~30s on Dashboard (idle-safe; also refresh on open / Try again).
- QA mocks: `?view=dashboard&mockSnapshot=empty|ready|error`.

## Files
- `apps/desktop/src/App.tsx` — Menubar vs Dashboard surface; Open Dashboard
- `apps/desktop/src/Dashboard.tsx` — shell UI
- `apps/desktop/src/featureSnapshot.ts` — IPC client + mocks
- `apps/desktop/src/dashboardWindow.ts` — surface detect + open helper
- `apps/desktop/src/App.css` — Dashboard layout slots
- `apps/desktop/src-tauri/tauri.conf.json` — `dashboard` window
- `apps/desktop/src-tauri/capabilities/default.json` — both windows
- `apps/desktop/src-tauri/src/lib.rs` — `open_dashboard` + hide-on-close
- `apps/desktop/README.md` — Dashboard smoke note

## How to verify (commands)
```bash
cd apps/desktop && pnpm exec tsc --noEmit
cargo test -p desktop --lib
# Manual (preferred):
cd apps/desktop && pnpm tauri dev
```

### Manual smoke
1. Tray → open Menubar: status Idle/Ready/Error + alert Steady/Elevated/High still work (`?mockStatus=ready&mockAlert=yellow` OK).
2. Click **Open Dashboard** → larger window titled “BioFocus — Dashboard”.
3. Idle / no Features: **No features yet** + dashed chart slot + Insights placeholder.
4. Force states in Dashboard URL (or Vite):  
   - `?view=dashboard&mockSnapshot=empty`  
   - `?view=dashboard&mockSnapshot=ready` (list FocusScore / StressIndex; still no Recharts)  
   - `?view=dashboard&mockSnapshot=error` → Try again
5. Close Dashboard (red traffic light) → window hides; Open Dashboard again → reappears.
6. Confirm Menubar alert colors still update after opening Dashboard (poll ~5s).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dashboard opens from Menubar (separate window; documented above)
- [ ] AC2: Calm shell states loading / empty / error (non-evaluative copy)
- [ ] AC3: Data only via IPC (`get_feature_snapshot` / Menubar still `get_status`); UI↛SQLite; mocks documented
- [ ] AC4: Menubar alert UX still works after opening Dashboard
- [ ] AC5: Chart layout slot only — **no** Recharts Feature series
- [ ] AC6: This handoff includes manual smoke steps
- [ ] Global DoD: no unwrap/expect in prod paths touched; UI↛DB; glossary terms

## Risks / not covered
- Live non-empty snapshot needs Feature Worker evidence (same as T1); use `mockSnapshot=ready` for layout QA.
- macOS tray icon remains template monochrome (unchanged from T3).
- Recharts series → **P4-E1-T3**; Insights list → **P4-E2**; LLM → **P4-E3**.

## Notes for QA
- IPC: `open_dashboard` (show), `get_feature_snapshot` (data). No new SQLite tables.
- Production Rust path for `open_dashboard` uses `Result` / soft `Err` strings — no `unwrap`/`expect`.
