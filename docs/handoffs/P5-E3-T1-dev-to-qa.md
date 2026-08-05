# Dev (+ UX) → QA: P5-E3-T1

## Meta
- **Task ID:** P5-E3-T1
- **Title:** Runnable iOS companion + HealthKit one-shot
- **Role that built:** Dev (+ UX for calm form copy)
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P5-E3-T1; brief `docs/handoffs/P5-E3-T1-pm-brief.md`
- **Branch:** `phase/5-wearable-dogfood`

## What changed
- Runnable Xcode target `BioFocusCompanion` at `apps/companion/ios/BioFocusCompanion.xcodeproj` (shared scheme).
- SwiftUI shell: Base URL + pairing token fields, **Send one heart-rate sample** button, Status for success / 401 / network / HealthKit gaps.
- Reuses contract sources: `IngestClient.swift`, `HeartRateSample.swift`, `SamplePost.swift` (ingest URL path join hardened; `HeartRateSampleError` now `LocalizedError`).
- Info.plist: HealthKit usage, local networking ATS (`NSAllowsLocalNetworking`), local-network usage string; HealthKit entitlements.
- README smoke notes: Simulator loopback + physical LAN; `apps/companion/README.md` points at the runnable project.

## How to verify (commands)
```bash
# Swift module compile (Simulator SDK; no device runtime required)
cd apps/companion/ios
SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
mkdir -p .derivedData/swiftc-check
xcrun --sdk iphonesimulator swiftc \
  -sdk "$SDK" \
  -target arm64-apple-ios17.0-simulator \
  -parse-as-library -emit-module -module-name BioFocusCompanion \
  -framework Foundation -framework HealthKit -framework SwiftUI \
  IngestClient.swift HeartRateSample.swift SamplePost.swift \
  BioFocusCompanion/BioFocusCompanionApp.swift BioFocusCompanion/ContentView.swift \
  -o .derivedData/swiftc-check/BioFocusCompanion.swiftmodule

# Rust companion regression (unchanged contract)
cargo test -p companion

# Full Xcode Run (needs matching iOS Simulator runtime in Xcode → Settings → Components)
open apps/companion/ios/BioFocusCompanion.xcodeproj
# Scheme BioFocusCompanion → Simulator or device → paste Base URL + token → Send once
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Runnable Xcode target (or clearly documented project) using existing Swift contract sources
- [ ] AC2: User can set base URL + paste pairing token
- [ ] AC3: One-shot HealthKit HR → Observation JSON array → `POST {baseURL}/v1/ingest` Bearer
- [ ] AC4: 401 and network errors surfaced (not swallowed)
- [ ] AC5: No busy-loop / continuous HealthKit polling — button / one-shot only
- [ ] AC6: Privacy — HR read only; local Desktop; no third-party analytics
- [ ] AC7: Handoff includes Simulator and/or device smoke notes
- [ ] Global DoD: glossary terms; no new SQLite/schema; idle-safe UI

## Risks / not covered
- On this Dev host, `xcodebuild -showdestinations` has **no eligible iOS Simulator destination** (Xcode 26.2 reports platform not installed) — full Simulator Run not executed here; `swiftc -emit-module` against `iphonesimulator` SDK **did** succeed.
- Live Desktop + real HealthKit end-to-end smoke not run in this session (needs Desktop + Health sample / device).
- Physical device still needs a Development Team in Signing & Capabilities (documented).
- Full dogfood runbook → **P5-E3-T2**.

## Notes for QA
- Default Base URL in UI: `http://127.0.0.1:8787` (Simulator).
- LAN: Desktop `BIOFOCUS_INGEST_LAN=1`, copy Base URL from Desktop Companion (P5-E2-T1).
- Simulator often has zero HR samples until added in Health — Status should say so, not hang.
- No Analytics / crash SDKs; no background HealthKit delivery APIs.
