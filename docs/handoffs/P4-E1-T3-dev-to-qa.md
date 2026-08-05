# Dev|UX → QA: P4-E1-T3

## Meta
- **Task ID:** P4-E1-T3
- **Title:** Recharts Feature series
- **Role that built:** UX (+ chart data helpers; no host IPC changes)
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P4-E1-T3; brief `docs/handoffs/P4-E1-T3-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- Replaced T2 `ChartSlot` placeholder with Recharts `LineChart` over Feature snapshot scalars.
- Series v1: `FocusScore`, `StressIndex`, `FatigueIndex` on primary Y (0–100); `ContextSwitchRate` on secondary Y when present (dashed).
- Calm labels/units only (Focus / Stress index / Fatigue index / Context switches) — no medical or evaluative claims.
- Kept idle-safe refresh: load on open + `SNAPSHOT_POLL_MS = 30_000` (unchanged).
- Enriched `mockSnapshot=ready` with 5 window ends × 4 Features for chart smoke.
- Added dependency `recharts@2.15.4`; README Dashboard section updated.

### Files
- `apps/desktop/package.json` / `pnpm-lock.yaml` — `recharts`
- `apps/desktop/src/featureChart.ts` — series meta, `buildChartPoints`, helpers
- `apps/desktop/src/Dashboard.tsx` — ChartSlot → Recharts
- `apps/desktop/src/featureSnapshot.ts` — richer ready mock
- `apps/desktop/src/App.css` — chart body / units
- `apps/desktop/README.md` — Dashboard chart note

### Unchanged (by design)
- Menubar alert UX / `get_status` poll
- Dashboard window lifecycle (`open_dashboard`, hide-on-close)
- Rust host / `get_feature_snapshot` DTO
- Insights slot (still placeholder → P4-E2)

## How to verify (commands)
```bash
cd apps/desktop && pnpm exec tsc --noEmit
# optional chart/mock smoke (Node):
node --input-type=module -e "
import { mockSnapshotFromLocation } from './src/featureSnapshot.ts';
import { buildChartPoints, presentSeriesIds } from './src/featureChart.ts';
const r = mockSnapshotFromLocation('?mockSnapshot=ready');
const pts = buildChartPoints(r.snapshot.features);
console.log(presentSeriesIds(pts), pts.length);
"
```

### Manual smoke
1. `pnpm tauri dev` (or Vite) → open Dashboard (`?view=dashboard` / Menubar **Open Dashboard**).
2. `?view=dashboard&mockSnapshot=ready` — four series + units caption; snapshot list populated.
3. `?view=dashboard&mockSnapshot=empty` — «No Feature series yet.»
4. `?view=dashboard&mockSnapshot=error` — error shell; chart empty placeholder; Menubar still independent.
5. Confirm no busy-loop (soft refresh ~30s only).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Charts v1 from snapshot — at least FocusScore / StressIndex / FatigueIndex; ContextSwitchRate when present
- [ ] AC2: Calm labels / units; no medical or evaluative claims
- [ ] AC3: Refresh on open and/or rare timer — no busy-loop (~30s)
- [ ] AC4: Data only via `get_feature_snapshot` (mocks OK); UI ↛ SQLite
- [ ] AC5: Menubar alert UX unchanged; Dashboard remains separate window from T2
- [ ] AC6: `tsc --noEmit` + smoke steps in this handoff
- [ ] Global DoD: no new prod `unwrap`/`expect`; UI↛DB; glossary terms Feature / Signal / Insight

## Risks / not covered
- Live non-empty series still needs Feature Worker evidence; layout QA uses `mockSnapshot=ready`.
- Snapshot list shows **all** window samples (ready mock = 20 rows) — noisy but truthful; not redesigned this task.
- Interactive `pnpm tauri dev` visual not required if mocks + tsc pass; recommend quick look before cluster PR.
- Recharts 2.x installed (3.x available; 2.15 keeps API stable for v1).

## Notes for QA
- CSR uses right Y-axis + dashed stroke so scale ≠ 0–100 scores.
- Tooltip / legend use calm labels from `CHART_SERIES_META`, not raw evaluative copy.
- No Rust changes — `cargo test -p desktop` optional regression only.
