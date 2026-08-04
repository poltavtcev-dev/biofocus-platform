# UX → QA: P1-E3-T2

## Meta
- **Task ID:** P1-E3-T2
- **Title:** Menubar status UX (minimal)
- **Role that built:** UX
- **Date:** 2026-08-03
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P1-E3 / P1-E3-T2; brief `docs/handoffs/P1-E3-T2-pm-brief.md`

## What changed
- Summary:
  - Menubar shell UI shows neutral Core status: **Idle** / **Ready** / **Error** (calm copy, no evaluative phrasing).
  - Status driven by IPC: prefers `get_status` (T3), falls back to `core_ping` until T3 lands.
  - Tray id `main`: tooltip `BioFocus — {Idle|Ready|Error}`; left-click shows/focuses the status window; macOS template icon.
  - Compact non-dashboard window (300×168); subtle status dot; dark/light via `prefers-color-scheme`.
  - QA mock without Core/DB: `?mockStatus=idle|ready|error`.
- Apps / files:
  - `apps/desktop/src/coreStatus.ts` (new)
  - `apps/desktop/src/App.tsx`, `App.css`
  - `apps/desktop/src-tauri/src/lib.rs` (thin tray wire)
  - `apps/desktop/src-tauri/tauri.conf.json` (window size / label)
  - `apps/desktop/README.md` (status + mock note)

## Screens / states checklist (for QA)

| State | How to see | Window | Tray tooltip |
| :--- | :--- | :--- | :--- |
| **Idle** (initial) | Brief flash on launch before IPC returns; or `?mockStatus=idle` | Dot muted · label Idle · “Waiting for Core.” | `BioFocus — Idle` |
| **Ready** | Normal `pnpm tauri dev` after `core_ping` ok (or T3 `get_status` db ok); or `?mockStatus=ready` | Dot soft green-gray · Ready · “Core is available.” (+ meta if IPC) | `BioFocus — Ready` |
| **Error** | `?mockStatus=error`; or break IPC; or (after T3) db error | Dot soft red-gray · Error · “Could not reach Core.” · **Try again** | `BioFocus — Error` |

**Not present (by design):** charts, dashboard widgets, traffic-light alerts, judgmental copy («ты выгорел»), DB paths / `rusqlite` in frontend.

## How to verify (commands)
```bash
cd apps/desktop
pnpm install
pnpm build
# Interactive:
pnpm tauri dev
# Then hover tray → tooltip; click tray → window; compare Ready.
# Force Error (append in webview URL or reload with query):
#   http://localhost:1420/?mockStatus=error
#   http://localhost:1420/?mockStatus=idle
#   http://localhost:1420/?mockStatus=ready

# Boundary smoke
rg -n "rusqlite|biofocus_main\\.db|CREATE TABLE" apps/desktop/src apps/desktop/dist || true

# Host still compiles
cargo check -p desktop
```

Ожидание:
- Labels only Idle / Ready / Error (English, calm).
- No charts / metrics panels.
- Frontend status path uses `invoke` only (`coreStatus.ts`).
- `rg` finds nothing for rusqlite / db file / SQL in `src`/`dist`.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Menubar/tray and/or minimal window shows neutral Core status Idle / Ready / Error (no judgmental copy)
- [ ] AC2: States readable (HIG-ish compact shell); no charts, no dashboard widgets, no Phase 4 clutter
- [ ] AC3: Status updates via IPC invoke (mock OK; prefers `get_status` if present)
- [ ] AC4: UI↛DB — no rusqlite / db paths / SQL in frontend
- [ ] AC5: Handoff present with states checklist
- [ ] Global DoD: UI↛DB; glossary; Phase 1 shell only

## Risks / not covered
- Full IPC contract / `get_status` implementation → **P1-E3-T3** (UI already prefers it when available)
- Boundary smoke suite formalization → **P1-E3-T4**
- GUI screenshots not attached (headless/agent session); verify visually via `pnpm tauri dev`
- Mock query works in webview URL; production users won’t use it — intended for QA/T4

## Notes for QA
- IPC today: `core_ping` → Ready when `status === "ok"`.
- Future: `get_status` → Ready when `dbStatus`/`db_status === "ok"`, else Error; version may appear as meta.
- Tray id: `main` (must match window label).
- Retry button only on Error.
- Copy intentionally English and neutral to match Apple HIG menubar tone for Phase 1.
