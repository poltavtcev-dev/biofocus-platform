# Dev|UX → QA: P6-E2-T1

## Meta
- **Task ID:** P6-E2-T1
- **Title:** Desktop quick-log Life Events
- **Role that built:** UX + Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P6-E2-T1 + `docs/handoffs/P6-E2-T1-pm-brief.md`
- **Branch:** `phase/6-life-context`

## What changed
- Menubar **Life events** quick-log: four calm buttons (Coffee / Walk / Lunch / Workout) + Recent list with manual Refresh.
- Tauri IPC `log_life_event` / `list_recent_life_events`: validate via `bio_spec`, append/read through `ObservationRepository` (same Observation store as ingest — no parallel table; UI ↛ SQLite).
- Contract docs: `docs/09-api.md` IPC section; Desktop README smoke/mock note.
- Host unit tests: v1 kinds accept / unknown reject / temp-DB insert+list round-trip.

### Crates / apps / files touched
- `apps/desktop/src-tauri/src/life_event_ipc.rs` (new)
- `apps/desktop/src-tauri/src/lib.rs` (commands + module docs)
- `apps/desktop/src/lifeEvents.ts` (new)
- `apps/desktop/src/App.tsx`, `apps/desktop/src/App.css`
- `apps/desktop/README.md`
- `docs/09-api.md`

## How to verify (commands)
```bash
cargo check -p desktop
cargo test -p desktop --lib life_event
cd apps/desktop && pnpm exec tsc --noEmit
```

QA mock (Vite/browser, no DB): `?mockLifeEvents=empty|ready|error`

## Acceptance Criteria checklist (for QA)
- [ ] AC1: User can log v1 Life Event (`coffee` / `walk` / `lunch` / `workout`) from Desktop with calm, non-evaluative copy
- [ ] AC2: Event becomes an Observation via IPC (UI ↛ SQLite); payload matches ADR-006 / `docs/07-contracts.md` (`data_type: "life_event"`, `provider_id: "com.biofocus.desktop"`, `payload.kind`)
- [ ] AC3: Logged event visible via existing path — **Menubar Recent list** (`list_recent_life_events` → `ObservationRepository::list_by_data_type("life_event")`)
- [ ] AC4: Idle-safe — no busy-loop poll; logging is on-demand invoke; Recent loads once on open + manual Refresh (status poll unchanged, unrelated)
- [ ] AC5: Manual smoke steps below (at least one kind end-to-end)
- [ ] AC6: Handoff present (`docs/handoffs/P6-E2-T1-dev-to-qa.md`)
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Manual smoke (end-to-end)
1. `pnpm tauri dev` (from `apps/desktop`) on branch `phase/6-life-context`.
2. In Menubar shell, open **Life events** → click **Coffee**.
3. Expect calm confirmation: `Logged coffee.`
4. **Recent** should show Coffee with a local time (or click **Refresh** once).
5. Optional: confirm storage via host path — same Observation repository / `data_type = life_event` (no UI SQL).

## Risks / not covered
- Desktop insert is **direct** `ObservationRepository::insert` (same store as ingest persist worker), not HTTP `/v1/ingest` enqueue — intentional for IPC reliability + immediate Recent visibility.
- Optional `note` / `duration_secs` UI fields not exposed (point-in-time kind-only log is enough for v1 dogfood).
- Calendar import / Insights / new schema — out of scope (P6-E3+).

## Notes for QA
- Visibility path for AC3: **Menubar Recent** via `list_recent_life_events` (document in qa-to-pm).
- Unknown kind rejected with calm string (unit-tested); HTTP `invalid_life_event` unchanged from P6-E1-T1.
