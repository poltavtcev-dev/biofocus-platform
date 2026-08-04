# QA → PM: P1-E3-T4

## Meta
- **Task ID:** P1-E3-T4
- **Title:** Smoke: UI ↔ Core boundary
- **Date:** 2026-08-03
- **Dev/UX handoff:** n/a (QA = исполнитель по brief `docs/handoffs/P1-E3-T4-pm-brief.md`; depends T2+T3 Done)
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `cargo test -p desktop` → **ok** (4/4)
  - `cd apps/desktop && CI=true pnpm install` → **ok**
  - `pnpm build` → **ok**
  - `pnpm tauri build` → **ok** → `BioFocus.app`
  - Smoke-start `BioFocus.app` binary ~3s → process alive, empty stderr, clean stop
  - `rg -n "rusqlite|CREATE TABLE|biofocus_main\\.db" apps/desktop/src apps/desktop/dist` → **no matches**
  - Mock/tooltip contract matrix (idle|ready|error) → **ok**; mock copy strings present in `dist`
- AC results (pass/fail per item):
  - **AC1 Pass** — Tray id `main`, initial tooltip `BioFocus — Idle`, click → show/focus window `main`; UI states Idle / Ready / Error (dot + label + calm detail). Binary shell starts.
  - **AC2 Pass** — `?mockStatus=error` → Error (“Could not reach Core.”) + **Try again** (retry → Idle then re-fetch); mock also covers idle/ready. Strings in production bundle.
  - **AC3 Pass** — Frontend `src`/`dist` без `rusqlite`, `CREATE TABLE`, хардкода `biofocus_main.db`.
  - **AC4 Pass** — Live IPC `get_status` → только `version` / `dbStatus` / `dbError?`; unit `status_json_has_no_observation_fields`; UI path `invoke` only (`coreStatus.ts`), fallback `core_ping` без Observation.
  - **AC5 Pass with notes** — см. Notes: при части storage-ошибок `dbError` **может** содержать абсолютный path (не Fail по brief; зафиксировано для E4/ADR).
  - **AC6 Pass** — tests + build + tauri bundle + binary smoke зелёные.
- Extra checks (edge / security):
  - Нет biometric/Observation полей в IPC payload.
  - Mock path не трогает реальную DB.
  - Интерактивный hover tray / webview URL mock в GUI session не крутился — закрыто code+bundle+binary smoke.

## Defects (if any)
- Нет блокирующих.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P1-E3-T4** to Done; закрыть **Epic E3**; Ready → **P1-E4-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs: `docs/12-development.md` + ARCHITECTURE_STATUS note; sanitization follow-up tracked in E4

## Suggested next Ready task
- **P1-E4-T1** — Phase 1 acceptance checklist (PM lead, QA collaborator)

## Notes for PM
### Soft note (AC5) — `dbError` path leakage
- Host: `probe_database_at` → `err.to_string()` из `StorageError`.
- `StorageError::CreateDir` display: `failed to create data directory {path}: …` — **абсолютный path** может попасть в IPC `dbError` и далее в UI meta (`coreStatus.ts`).
- `HomeDirUnavailable` — без path (пример из docs ок).
- Рекомендация E4 / короткий ADR: санитизировать `dbError` на host (категория + без `PathBuf` / home), UI оставляет спокойный primary copy.
- Не блокер закрытия E3 (brief: Pass with notes).

### Smoke coverage summary
| Check | Result |
| :--- | :--- |
| Menubar/tray + status states | Pass (code + binary) |
| Mock error / retry | Pass (contract + dist) |
| UI↛DB boundary | Pass |
| No Observation over IPC | Pass |
| dbError path soft check | Pass with notes |
| Build/test/bundle | Pass |
