# Dev|UX → QA: P11-E3-T1

**From:** UX + Dev  
**To:** QA  
**Date:** 2026-08-10  
**AC source:** `docs/handoffs/P11-E3-T1-pm-brief.md` / `docs/SPRINT_ROADMAP.md` → P11-E3-T1  
**Branch:** `phase/11-ai-coaching-polish`

## What changed
- Calm **Local AI** provider status on Dashboard via new IPC `get_local_llm_status` (`disabled` / `ready` / `error`) — config-only from `BIOFOCUS_LOCAL_LLM*`; **no** HTTP probe, no secrets/tokens.
- `generate_report` now uses `build_report_with_pack("biofocus.default", "1", features, insights, recommendations)` and evaluates Recommendations on the Generate path.
- Soft-fail `llmStatus` pattern preserved (markdown kept on timeout/error; offline when LLM off).
- UX: provider status row + pack meta in `ReportSlot`; calm non-clinical copy; mocks `?mockLlmStatus=…`.
- Docs: `09-api`, `12-development`, `10-security`, glossary, desktop README.

## Crates / apps / files
- `apps/desktop/src-tauri/src/lib.rs` — status DTO + command; pack-aware assemble; tests
- `apps/desktop/src/llmProvider.ts` — status fetch + mocks
- `apps/desktop/src/Dashboard.tsx`, `report.ts`, `App.css`
- `apps/desktop/README.md`
- `docs/09-api.md`, `docs/12-development.md`, `docs/10-security.md`, `docs/16-glossary.md`

## How to verify (commands)

```bash
# From repo root
cargo test -p desktop --lib assemble_report
cargo test -p desktop --lib local_llm_provider

cd apps/desktop && pnpm exec tsc --noEmit
```

### Manual / browser smoke (Dashboard)
1. Open Dashboard (`?view=dashboard`). Confirm **Local AI · Off** (or Ready/Issue) status row + pack `biofocus.default @ 1` — **without** clicking Generate. Soft poll must **not** generate a report.
2. Click **Generate report** → deterministic markdown (+ collapsible prompt). Default host: `llmStatus` disabled + calm “optional / off” line.
3. QA mocks: `?view=dashboard&mockLlmStatus=disabled|ready|error` and `?mockReport=idle|ready|disabled|ok|error`.
4. Optional LLM: launch with `BIOFOCUS_LOCAL_LLM=1` (+ local Ollama) → status Ready → Generate → interpretation when ok; timeout/error soft-fails without losing markdown.

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Dashboard shows calm provider status via IPC (`disabled`/`ready`/`error`); UI↛SQLite; no secrets/tokens
- [ ] AC2 Generate uses `build_report_with_pack` + default pack; Features+Insights+Recommendations; explicit user action only (not on open/poll)
- [ ] AC3 LLM off → offline markdown + `llm_prompt`; calm optional-AI copy
- [ ] AC4 LLM on → soft-fail `llmStatus` without losing markdown
- [ ] AC5 Copy non-clinical; no diagnosis / surveillance / cloud marketplace UI
- [ ] AC6 Docs updated (`09-api` / `12-development` / desktop README) + browser mocks
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live Ollama smoke is optional (env + local endpoint); automated Pass covers disabled/ready/error mapping + pack markdown with Recommendations.
- Status is **config honesty**, not endpoint reachability (no probe on open — by design).
- No in-app toggle that writes env/secrets (OOS per brief).

## Notes for QA
- Pack id/version are hard-coded to default on Generate (no picker — ADR/brief OK).
- Soft refresh (~30s) re-reads status IPC only; never `generate_report`.
