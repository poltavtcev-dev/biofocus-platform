# PM Acceptance: P1-E4-T1 — Phase 1 checklist

**Role:** PM (lead)  
**Date:** 2026-08-03  
**Verdict:** Pass (evidence green)  
**Brief:** `docs/handoffs/P1-E4-T1-pm-brief.md`

## Evidence pack (re-run)

| Check | Result |
| :--- | :--- |
| `cargo check` (workspace) | ok |
| `cargo test -p bio-spec` | 5/5 ok |
| `cargo test -p runtime` | 5/5 ok |
| `cargo test -p storage` | 17/17 ok |
| `cargo test -p desktop` | 4/4 ok |
| `cd apps/desktop && pnpm build` | ok (this run) |
| Menubar smoke (E3-T4) | Pass with notes — prior handoff |

## Phase 1 Done IDs

| Epic | Tasks |
| :--- | :--- |
| E1 Foundation crates | T1 Workspace · T2 bio-spec · T3 runtime · T4 LICENSE |
| E2 Storage | T1 WAL · T2 migrate · T3 ObservationRepository · T4 integration tests |
| E3 Menubar | T1 Tauri scaffold · T2 UX · T3 get_status · T4 boundary smoke |
| E4 Exit | **T1 acceptance** ← this document · T2 docs sync (next) |

## Phase 2 Ready list (NOT started)

Captured from `/docs/14-roadmap.md` — planning only:

1. Local HTTP Server `POST /v1/ingest` (+ pairing token / local bind)
2. macOS active window & keystroke collector → Observations
3. iOS Companion (HealthKit webhook) → ingest

**Scope guard:** no Phase 2 implementation in this task.

## Open follow-ups (non-blocking Phase 1)

1. **Sanitize `dbError` on host** — `StorageError::CreateDir` may expose absolute path via IPC → UI meta (E3-T4 notes). Future small Dev task / optional ADR for error surface.
2. GUI interactive tray hover not fully exercised in CI sessions — acceptable for MVP scaffold; revisit in Phase 3 alerts if needed.
3. **`docs/14-roadmap.md` Phase 1 checkbox** — **[x] Done** (user approve 2026-08-03).

## What PM updated after this checklist
- [x] `/docs/SPRINT_ROADMAP.md` — P1-E4-T1 Done; assign T2
- [x] Execution canvas
- [x] This acceptance note

## Suggested next
- **P1-E4-T2** — Done 2026-08-04 (`docs/handoffs/P1-E4-T2-qa-to-pm.md`)
- Push CI (`.github/workflows/ci.yml`) → PM Phase 2 decomposition
