# PM Brief → Dev (+ UX): P5-E3-T1

**From:** PM  
**To:** Dev (+ UX for minimal UI copy)  
**Status:** Done (QA Pass with notes, 2026-08-05)  
**Date:** 2026-08-05  
**Closed previous:** P5-E2-T1 (QA Pass with notes — Companion LAN Base URL + token/QR; Epic **P5-E2** ✅)  
**Closed this:** QA Pass with notes → `docs/handoffs/P5-E3-T1-qa-to-pm.md`; Ready **P5-E3-T2**  
**Evidence:** `docs/handoffs/P5-E2-T1-qa-to-pm.md`

## Task
**P5-E3-T1 — Runnable iOS companion + HealthKit one-shot**

## Why
Desktop pairing now exposes a copyable LAN (or loopback) base URL + token. Dogfood still needs a **runnable** iOS path that reads one HealthKit HR sample and POSTs the existing Observation contract to Desktop ingest — Simulator loopback and/or physical phone on LAN.

## Acceptance Criteria
1. Runnable Xcode target **or** clearly documented project that builds/runs using existing Swift sources under `apps/companion/ios/`.
2. User can set **base URL** + paste **pairing token** (from Desktop Companion).
3. One-shot HealthKit heart-rate read → Observation JSON array → `POST {baseURL}/v1/ingest` with `Authorization: Bearer …`.
4. Surface **401** and network errors to the user (no silent swallow).
5. **No** busy-loop / continuous HealthKit polling — one-shot (or explicit user action) only.
6. Privacy: HR read only per existing stub; local Desktop only; no third-party analytics.
7. Handoff includes Simulator and/or device smoke notes: `docs/handoffs/P5-E3-T1-dev-to-qa.md`.

## Out of scope
- App Store release / signing polish beyond what’s needed to run locally
- Background continuous streaming
- Vendor wearable SDKs beyond HealthKit
- New SQLite schema / Desktop ingest auth changes
- Full dogfood runbook (→ **P5-E3-T2**)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Modules: `apps/companion/ios/` — reuse `IngestClient.swift`, `HeartRateSample.swift`, `SamplePost.swift`
- Contract: same Observation JSON as Rust companion / `docs/09-api.md`
- Pairing: copy Base URL + token from Desktop Companion (P5-E2-T1); LAN needs Desktop `BIOFOCUS_INGEST_LAN=1`
- Branch: `phase/5-wearable-dogfood`
- Idle-safe; calm non-evaluative copy

## Starting points
- Stub README: `apps/companion/ios/README.md`
- Desktop how-to: `docs/12-development.md` (LAN opt-in + Companion Base URL)

## After QA Pass
~~PM → Ready **P5-E3-T2**~~ — done; see `docs/handoffs/P5-E3-T2-pm-brief.md`.
