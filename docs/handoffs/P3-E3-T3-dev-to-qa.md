# UX|Dev → QA: P3-E3-T3

## Meta
- **Task ID:** P3-E3-T3
- **Title:** Menubar traffic-light UX
- **Role that built:** UX + Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P3-E3-T3
- **Branch:** `phase/3-pipeline-features`

## What changed
- Shell alert indicator (color + Steady / Elevated / High) from IPC `alertLevel`.
- Tray tooltip: `BioFocus — {Ready|…} · {Steady|Elevated|High}`.
- Soft poll ~5s for status/alert.
- QA mocks: `?mockAlert=green|yellow|red` (optional with `mockStatus`).
- Public docs: removed personal device inventory; generic wearable forward note only.

## Files
- `apps/desktop/src/App.tsx`, `App.css`, `coreStatus.ts`
- `docs/12-development.md`, `PROJECT_CANVAS.md`, `SPRINT_ROADMAP.md`, `apps/companion/README.md`
- handoffs `P3-E3-T3-*`

## How to verify
```bash
cargo test -p desktop
cd apps/desktop && pnpm exec tsc --noEmit
# Manual: pnpm tauri dev
# Open ?mockStatus=ready&mockAlert=yellow — yellow dot + “Elevated”; tray tooltip updates.
# Same for green / red.
```

## AC checklist
- [x] Indicator color from IPC alert level
- [x] Calm tray/shell copy (non-evaluative)
- [x] Idle/Ready/Error unchanged; no charts
- [x] Manual smoke steps above
- [x] UI↛DB

## Notes
- macOS tray **icon** stays template monochrome; color is shell dot + tooltip text (platform constraint).
