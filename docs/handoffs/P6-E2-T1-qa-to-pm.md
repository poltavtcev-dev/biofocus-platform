# QA → PM: P6-E2-T1

## Meta
- **Task ID:** P6-E2-T1
- **Title:** Desktop quick-log Life Events
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P6-E2-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p desktop --lib life_event` → **3 passed** (v1 kinds, unknown reject, temp-DB insert+list)
  - `cargo test -p bio-spec --test contracts life_event` → **4 passed** (contract still green)
  - `cargo check -p desktop` → ok
  - `pnpm exec tsc --noEmit` (apps/desktop) → ok
- AC results:
  - **AC1 Pass** — Menubar Life events: Coffee / Walk / Lunch / Workout; copy «Note what happened… No scores.» / «Logged coffee.»
  - **AC2 Pass** — IPC `log_life_event` → `bio_spec` validate → `ObservationRepository::insert`; `data_type: "life_event"`, `provider_id: "com.biofocus.desktop"`, `payload.kind` ∈ v1; UI has no SQLite imports
  - **AC3 Pass** — Visibility path: **Menubar Recent** via `list_recent_life_events` → `list_by_data_type("life_event")` (documented in Dev handoff)
  - **AC4 Pass** — Logging is on-demand invoke; Recent = load-on-open + manual Refresh; no life-event poll/timer (status 5s poll unchanged, unrelated)
  - **AC5 Pass** — Smoke steps in Dev handoff; host e2e covered by temp-DB round-trip for `coffee`
  - **AC6 Pass** — `docs/handoffs/P6-E2-T1-dev-to-qa.md` present
  - **Global DoD Pass** — no `unwrap`/`expect` in prod `life_event_ipc` / command paths; UI↛DB; glossary Observation / Life Event
- Extra checks:
  - Unknown kind rejected with calm string (unit test)
  - `docs/09-api.md` documents both IPC commands
  - Out of scope respected: no calendar, no new schema, no Insight rules, ADR-006 kinds unchanged

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P6-E2-T1 → Done; Ready → **P6-E3-T1** (Calendar → Observations)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs (as needed): `docs/ARCHITECTURE_STATUS.md`, `docs/12-development.md`, `docs/PROJECT_CANVAS.md` — Desktop quick-log shipped

## Suggested next Ready task
- **P6-E3-T1** — Calendar → Observations (per pm-brief After QA Pass)

## Notes for PM
- **Note:** Persist path is direct `ObservationRepository::insert` from IPC (same Observation store as ingest worker), not HTTP `/v1/ingest` enqueue — intentional for immediate Recent visibility; still UI ↛ SQLite and no parallel store.
- **Note:** Live `pnpm tauri dev` click-through not executed in this QA session; Core path + UI code review + mock `?mockLifeEvents=` available for browser smoke.
- Epic **P6-E2** closes with this task (single T1).
