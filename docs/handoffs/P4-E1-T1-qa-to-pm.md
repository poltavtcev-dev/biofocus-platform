# QA → PM: P4-E1-T1

## Meta
- **Task ID:** P4-E1-T1
- **Title:** Feature snapshot API + IPC
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P4-E1-T1-dev-to-qa.md`
- **Verdict:** Pass
- **Branch:** `phase/4-dashboard-ai`

## What was verified
### Commands
| Command | Result |
| :--- | :--- |
| `cargo test -p feature-engine` | **28 passed** (incl. `snapshot::*` empty + non-empty) |
| `cargo test -p desktop --lib` | **15 passed** (incl. empty/non-empty DTO, `get_status` has no `features`, hook caches StressIndex / High_Stress) |
| `cargo check -p desktop` | **ok** |

### AC results
| AC | Result |
| :--- | :--- |
| AC1: Public Core `FeatureSnapshot` — Features (+ Signals), ids / values / time windows / provenance Observation ids | **Pass** — `feature_engine::FeatureSnapshot::{empty,from_engine_output}`; re-exports `Feature` / `FeatureValue` |
| AC2: IPC `get_feature_snapshot` registered; `get_status` not bloated | **Pass** — command registered; `CoreStatus` = version / dbStatus / alertLevel only; unit assert no `features` key |
| AC3: No raw Observation biometrics / absolute paths in IPC payload | **Pass** — DTO maps provenance ids only; tests assert no `payload` / `rmssd` / `/Users` / `.biofocus`; catalog emits no `FeatureValue::Object` |
| AC4: Unit tests empty + non-empty synthetic | **Pass** — Core + desktop DTO + hook integration |
| AC5: Idle-safe pure cache read | **Pass** — `get_feature_snapshot` → `SnapshotState::current()` clone; no SQLite / busy-loop in invoke |
| AC6: Contract in `docs/09-api.md` + note in `docs/12-development.md` | **Pass** |
| Global DoD: no prod `unwrap`/`expect`; UI↛DB; glossary; no Recharts / LLM / new SQLite table | **Pass** — `unwrap`/`expect` only in `#[cfg(test)]`; cache soft-fails on poison → empty |

### Extra checks
- Security: UI still IPC-only; snapshot cache written only by Feature Worker hook (in-process).
- Edge: missing `SnapshotState` → empty snapshot; poisoned mutex → empty; idle `[]`/`[]`.
- Scope: no React consumer / Recharts / knowledge-engine / LLM (correctly deferred).

## Defects
- None blocking.
- **Note (non-blocking):** live Tauri `invoke` smoke not run — expected; Dashboard consumer is **P4-E1-T2**.
- **Note (PM/git):** branch may still carry Menubar (`P3-E3-T3`) as base if not yet on `main` — merge ordering is cluster concern, not an AC fail.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P4-E1-T1 → Done; Ready → **P4-E1-T2**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Optional refresh: `docs/ARCHITECTURE_STATUS.md` / `docs/12-development.md` Ready line (contract already documented)

## Suggested next Ready task
- **P4-E1-T2** — Dashboard shell (window / route) — role **UX + Dev**

## Notes for PM
- Handoff + docs already on disk; no QA code fixes needed.
- After Done: brief for T2 if not present; UX may stub IPC with mocks if documented.
