# PM Brief → Dev: P5-E3-T2

**From:** PM  
**To:** Dev  
**Status:** Ready  
**Date:** 2026-08-05  
**Closed previous:** P5-E3-T1 (QA Pass with notes — runnable `BioFocusCompanion.xcodeproj` + HealthKit one-shot → ingest)  
**Evidence:** `docs/handoffs/P5-E3-T1-qa-to-pm.md`

## Task
**P5-E3-T2 — Dogfood runbook + contract docs**

## Why
iOS companion is runnable; dogfood still needs a single end-to-end runbook so an operator can enable LAN, pair, post HR from iPhone/Simulator, and confirm the Observation landed — without inventing steps from scattered READMEs.

## Acceptance Criteria
1. End-to-end runbook in `docs/12-development.md` (or clearly linked section): enable LAN → note Base URL → pair token → open/run iOS companion → one-shot post → Observation visible (status / Dashboard / storage path as appropriate).
2. Companion READMEs (`apps/companion/README.md`, `apps/companion/ios/README.md`) align with the runbook and point at `BioFocusCompanion.xcodeproj`.
3. `PROJECT_CANVAS` wearable § reflects runnable iOS path + runbook (no personal device inventory in git).
4. Contract docs (`09-api` / related) no longer call iOS a “stub-only” path if that contradicts the shipped Xcode target; keep Observation / Bearer contract accurate.
5. No secrets, hostnames of personal devices, or device serials committed.
6. Handoff: `docs/handoffs/P5-E3-T2-dev-to-qa.md`.

## Out of scope
- New product Features / Insights
- Non-HealthKit wearable bridges
- App Store / signing polish
- Changing ingest auth or SQLite schema
- Implementing missing Simulator runtime on CI hosts

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `docs/` + companion READMEs; code changes only if a doc inconsistency requires a one-line pointer fix
- Branch: `phase/5-wearable-dogfood`
- Closing this task closes Epic **P5-E3** and (with E1–E2 already Done) completes Phase 5 Kanban — cluster PR when Ready to ship

## Starting points
- iOS project: `apps/companion/ios/BioFocusCompanion.xcodeproj`
- Pairing / LAN: `docs/12-development.md` (existing LAN + Companion Base URL sections)
- Smoke notes: `docs/handoffs/P5-E3-T1-dev-to-qa.md`

## After QA Pass
PM → mark Epic **P5-E3** ✅ / Phase 5 Done (or next gate per roadmap) unless sprint re-order.
