# QA → PM: P5-E3-T1

## Meta
- **Task ID:** P5-E3-T1
- **Title:** Runnable iOS companion + HealthKit one-shot
- **Date:** 2026-08-05
- **Dev/UX handoff:** `docs/handoffs/P5-E3-T1-dev-to-qa.md`
- **Verdict:** Pass with notes

## What was verified
- Commands run + results:
  - `swiftc -emit-module` (iphonesimulator SDK, all app + contract Swift sources) → **OK** (`BioFocusCompanion.swiftmodule`)
  - `cargo test -p companion` → **4 passed**
  - Project present: `BioFocusCompanion.xcodeproj` + shared scheme `BioFocusCompanion`
  - Static: no `HKObserverQuery` / background delivery / busy-loop timers in iOS sources
  - `xcodebuild -showdestinations` on QA host → **no eligible Simulator destination** (Xcode 26.2: “iOS 26.2 is not installed” under Components) — full Simulator Run not executed here
- AC results:
  - AC1 Runnable Xcode target + documented open/run/compile → **Pass** (project + README; compile verified via SDK)
  - AC2 Base URL + token fields → **Pass** (code review)
  - AC3 One-shot HK → Observation array → `POST …/v1/ingest` Bearer → **Pass** (code path `SamplePost` → `IngestClient`; live HK/Desktop smoke not run)
  - AC4 401 / network surfaced in Status → **Pass** (`IngestClientError` + UI mapping)
  - AC5 No continuous polling → **Pass**
  - AC6 Privacy (HR read, local ATS/LAN strings, no analytics) → **Pass**
  - AC7 Smoke notes in README + Dev handoff → **Pass**
- Extra checks: calm non-evaluative copy; local-only networking keys; contract fields match `docs/07-contracts.md` heart_rate shape.

## Defects (if any)
- None blocking. Host gap: install iOS 26.2 platform (Xcode → Settings → Components) before local `xcodebuild`/Simulator Run on this machine.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — P5-E3-T1 → Done; Ready **P5-E3-T2**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `docs/12-development.md` (point at `BioFocusCompanion.xcodeproj`), `ARCHITECTURE_STATUS` / Phase 5 notes; full dogfood runbook stays **P5-E3-T2**

## Suggested next Ready task
- **P5-E3-T2** — Dogfood runbook + contract docs

## Notes for PM
- Pass with notes solely for missing live Simulator/device + Desktop E2E on the QA host (platform component / Health sample). Implementation and documented run path meet AC.
- Device signing still requires a Development Team in Xcode (expected; out of App Store polish scope).
