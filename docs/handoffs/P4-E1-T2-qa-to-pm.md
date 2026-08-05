# QA → PM: P4-E1-T2

## Meta
- **Task ID:** P4-E1-T2
- **Title:** Dashboard shell (window / route)
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P4-E1-T2-dev-to-qa.md`
- **Verdict:** Pass with notes
- **Branch:** `phase/4-dashboard-ai`

## What was verified
### Commands
| Command | Result |
| :--- | :--- |
| `cd apps/desktop && pnpm exec tsc --noEmit` | **ok** (exit 0) |
| `cargo test -p desktop --lib` | **15 passed** |
| Node check of `mockSnapshotFromLocation` | **ok** — `empty` / `ready` (FocusScore, StressIndex) / `error`; bogus → null |

### AC results
| AC | Result |
| :--- | :--- |
| AC1: Открываемый Dashboard из Menubar (отдельное окно или route) | **Pass** — design: separate Tauri window `label: dashboard`, URL `index.html?view=dashboard`; Menubar **Open Dashboard** → `invoke("open_dashboard")` (show / focus); Vite preview falls back to `?view=dashboard` |
| AC2: Спокойный shell loading / empty / error; нет evaluative copy | **Pass** — `Dashboard.tsx` + copy in `featureSnapshot.ts` («No features yet», «Could not load», «Features available»); Insights placeholder non-evaluative |
| AC3: Данные только через IPC; UI↛SQLite; mocks documented | **Pass** — `get_feature_snapshot` / Menubar still `get_status`; no sqlite/recharts in `apps/desktop/src`; mocks in README + handoff (`?view=dashboard&mockSnapshot=…`) |
| AC4: Menubar alert UX не ломается | **Pass (static)** — `AlertIndicator` + `STATUS_POLL_MS = 5_000` unchanged in `MenubarShell`; Dashboard is separate surface (`isDashboardSurface`) |
| AC5: Нет Recharts Feature series — layout slots OK | **Pass** — `ChartSlot` placeholder only; `recharts` absent from `package.json` / imports |
| AC6: Handoff с manual smoke | **Pass** — steps in `P4-E1-T2-dev-to-qa.md` + README |
| Global DoD: no prod `unwrap`/`expect`; UI↛DB; glossary | **Pass** — `open_dashboard` returns `Result<(), String>`; `unwrap`/`expect` only under `#[cfg(test)]` in `lib.rs`; terms Feature / Signal / Insight used correctly |

### Extra checks
- Security: UI IPC-only; capabilities `windows: ["main", "dashboard"]`; no new SQLite schema.
- Window lifecycle: CloseRequested → `prevent_close` + `hide()` (reopen cheap) — code review.
- Scope: Insights list / Recharts / LLM correctly deferred (placeholders only).
- Soft snapshot refresh ~30s on Dashboard (idle-safe); retry on error.

## Defects
- None blocking.
- **Note (non-blocking):** interactive `pnpm tauri dev` window smoke (open / hide / re-show, live Menubar poll after Dashboard open) **not** run in this QA session — shell states covered via mocks + static review; recommend PM/UX quick visual once before cluster PR if desired.
- **Note (Kanban drift):** `SPRINT_ROADMAP.md` still lists **P4-E1-T1** as Ready while T1 QA report exists — PM should reconcile T1 Done when closing T2.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P4-E1-T2 → Done; T1 Done reconciled; Ready → **P4-E1-T3**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Optional: `docs/12-development.md` / `ARCHITECTURE_STATUS` — Dashboard window note (README already has smoke)

## Suggested next Ready task
- **P4-E1-T3** — Recharts Feature series — role **UX (+ Dev IPC glue if needed)**

## Notes for PM
- Design choice for AC1 is documented in Dev handoff (separate window, not in-app Menubar route).
- Live non-empty snapshot still depends on Feature Worker evidence (same as T1); layout QA uses `mockSnapshot=ready`.
- No QA code fixes; handoff on disk only — fold into next code/docs commit of the Phase 4 cluster.
)
