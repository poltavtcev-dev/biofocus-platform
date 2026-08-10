# QA → PM: P10-E1-T1

## Meta
- **Task ID:** P10-E1-T1
- **Title:** ADR-010: Plugin wave-1 source + Observation contract
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P10-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `rg -n "ADR-010"` across decision-log, plugin-sdk, contracts, glossary, storage, feature-catalog, 12-development — all present.
  - `rg -n "browser_category|BIOFOCUS_BROWSER_CATEGORIES|DistractionScore"` — contract + ADR detail + glossary + catalog wiring present.
  - No `CREATE TABLE … plugin|browser` in ADR/storage — OK (no schema to apply).
  - `git diff --name-only -- crates/ apps/` — empty (docs-only; no migration / code).
  - Dev handoff file present.
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 ADR-010: wave-1 = **Browser categories**; rationale (catalog `DistractionScore`, CSR complement, Capability Model); IDE/Git deferred | **Pass** |
| AC2 Observation contract: `browser_category`, coarse payload, `com.biofocus.macos.browser`, opt-in default off, idle poll; forbids URLs / content / surveillance framing | **Pass** |
| AC3 Rejected alternatives (both sources; always-on; cloud sync; marketplace; plugin SQLite registry; ambient; IDE/Git v1; URL logging; NotificationPressure here) | **Pass** |
| AC4 Schema — none to apply; existing `observations`; **no migration**; deferred allowlist needs future ADR + approve | **Pass** — **no user schema approve required** for v1 |
| AC5 E2 plugin → channel → persist; E3 `DistractionScore` + ADR-007 confidence + calm non-clinical framing | **Pass** |
| AC6 Docs touch `08` / `07` / `16` (+ storage / catalog / 12-development) as planned/ADR notes | **Pass** |
| AC7 `docs/handoffs/P10-E1-T1-dev-to-qa.md` | **Pass** |
| Global DoD: UL terms; calm framing; UI↛DB; no marketplace crate; LLM not defining payloads/Features; personal self-tracking | **Pass** |

- Extra checks: Out of scope respected (no collector implementation, no Feature DAG, no PR). Probe mechanics correctly deferred to E2 while privacy bar is locked.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move P10-E1-T1 to Done; Ready **P10-E2-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: confirm `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` reflect ADR-010 recorded (Browser chosen)
- [ ] Brief for next: `docs/handoffs/P10-E2-T1-pm-brief.md`

## Suggested next Ready task
- **P10-E2-T1** — Implement wave-1 Browser categories plugin (`BioFocusPlugin`, opt-in `BIOFOCUS_BROWSER_CATEGORIES`, mock probe, contracts finalize, idle-safe tests). No schema approve gate.

## Notes for PM
- Branch: `phase/10-plugin-wave-1`.
- Evidence: this file + `docs/handoffs/P10-E1-T1-dev-to-qa.md` + ADR-010 in `docs/decision-log.md`.
- Chosen source for Phase 10: **Browser categories** → E3 Feature **`DistractionScore`**. IDE/Git deferred beyond this wave.
- PR freeze still active — no PR.
