# Dev → QA: P3-E3-T2

## Meta
- **Task ID:** P3-E3-T2
- **Title:** IPC expose alert level
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P3-E3-T2; `docs/handoffs/P3-E3-T2-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- `get_status` IPC adds `alertLevel`: `"green"` | `"yellow"` | `"red"` (camelCase JSON).
- `dbStatus` Idle/Ready/Error path unchanged; alert independent of DB probe.
- Feature Worker uses `CatalogAlertHook`: rolling snapshot → `register_catalog_v1` → `map_alert_level` → `AlertState` read by `get_status`.
- Frontend `coreStatus.ts` parses `alertLevel` onto `CoreStatusView` (no traffic-light UX yet → T3).
- Docs: `09-api.md`, `12-development.md`.

## Crates / apps / files touched
- `apps/desktop/src-tauri/Cargo.toml` — deps: feature-engine, pipeline, bio-spec, uuid
- `apps/desktop/src-tauri/src/alert_state.rs` (new)
- `apps/desktop/src-tauri/src/feature_host.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/coreStatus.ts`
- `docs/09-api.md`, `docs/12-development.md`
- `docs/handoffs/P3-E3-T2-pm-brief.md`

## How to verify (commands)
```bash
cargo test -p desktop
cargo test -p feature-engine
```

## Acceptance Criteria checklist (for QA)
- [x] AC1 — `get_status` returns `alertLevel`; `dbStatus` ok/error still works
- [x] AC2 — UI obtains alert via IPC only (`coreStatus.ts` parse; no SQLite)
- [x] AC3 — no absolute paths / Observation / biometric fields in status JSON
- [x] AC4 — host tests: status JSON + hook Yellow/Red
- [x] AC5 — this handoff
- [x] Global DoD — no unwrap/expect in prod paths; idle-safe worker

## Risks / not covered
- Menubar 🟢/🟡/🔴 colors / copy → **P3-E3-T3**.
- Live alert only updates after Feature Worker sees **new** Observations past tip-cursor (launch tip skip is existing E1 carry).
- Rolling snapshot is in-memory window (~15m+); not persisted.

## Notes for QA
- Default without Feature evidence / missing `AlertState`: `green`.
- Hook tests: elevated RMSSD → Yellow; contiguous high stress → Red.
