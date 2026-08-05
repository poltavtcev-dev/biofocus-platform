# Dev|UX → QA: P4-E3-T3

**From:** UX + Dev  
**To:** QA  
**Date:** 2026-08-05  
**AC source:** `docs/handoffs/P4-E3-T3-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P4-E3-T3  
**Branch:** `phase/4-dashboard-ai`

## What changed
- Dashboard **Report** section with explicit «Generate report» control (no auto-invoke on open / soft poll).
- Tauri IPC `generate_report`: host builds offline report from cached Feature snapshot + evaluate-on-read Insights via `report_engine::build_report`; optional `interpret_report` only when `LocalLlmConfig::from_env().enabled`.
- When LLM off: `llmStatus: "disabled"`, markdown + `llmPrompt` returned, **no HTTP**.
- Calm copy: AI is local / optional; offline-first path; soft LLM errors keep markdown.
- Frontend module `report.ts` + styles; QA mock `?mockReport=idle|disabled|ok|error`.
- Docs: `docs/09-api.md` § `generate_report`.

## Crates / apps / files touched
- `apps/desktop/src-tauri/Cargo.toml` — `report-engine` dep
- `apps/desktop/src-tauri/src/lib.rs` — `generate_report`, `ReportDto`, `assemble_report_dto`, unit tests
- `apps/desktop/src/report.ts` — IPC + view states
- `apps/desktop/src/Dashboard.tsx` — `ReportSlot`
- `apps/desktop/src/App.css` — report styles
- `docs/09-api.md` — IPC contract

## How to verify (commands)
```bash
cargo check -p desktop
cargo test -p desktop assemble_report
cargo test -p report-engine
cd apps/desktop && pnpm exec tsc --noEmit
```

### Smoke (manual / app)
1. Open Dashboard (`open_dashboard` / `?view=dashboard`). Confirm Report section idle; **no** network / LLM call on open.
2. Click **Generate report** → see deterministic markdown (+ collapsible prompt). Default: calm line that local AI is optional and off (`llmStatus` disabled).
3. Soft poll (30s) must **not** regenerate the report.
4. Optional LLM smoke (manual): launch desktop with `BIOFOCUS_LOCAL_LLM=1` (and local Ollama on default URL if available) → Generate again → interpretation when endpoint ok; timeout/error soft-fails without panicking markdown.

### QA mocks (no Core)
- `?mockReport=disabled` — ready offline mock
- `?mockReport=ok` — mock with interpretation
- `?mockReport=error` — error state

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Dashboard has clear «Generate report» control / user-opened flow
- [ ] AC2 Flow shows deterministic markdown and/or prompt from `build_report` via IPC (UI ↛ SQLite)
- [ ] AC3 LLM enabled → optional interpreted output on same explicit action; disabled → calm offline path, no network from UI when off
- [ ] AC4 Copy: AI local / optional; calm, non-evaluative, no clinical claims
- [ ] AC5 Never auto-invoke LLM / networked report on app / Dashboard open
- [ ] AC6 Idle-safe: no busy-loop; report refresh only on user action
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms
- [ ] Handoff smoke steps present (this file)

## Risks / not covered
- Full GUI click-through against live Tauri not automated here (unit + tsc only).
- LLM path needs host env set **before** process start; no in-app toggle (OOS).
- Empty Feature cache still yields calm minimal markdown from `build_report` — expected.

## Notes for QA
- Report is **not** wired into the 30s Feature/Insights poll — by design (AC5/AC6).
- `assemble_report_dto` unit tests cover disabled offline DTO + markdown with Features.
