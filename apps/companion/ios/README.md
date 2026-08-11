# iOS HealthKit companion (P5-E3 + P15 ADR-016)

Runnable Xcode app that posts HealthKit **heart rate** and **HRV (SDNN)** Observations to Desktop ingest — with optional **Auto-sync** (queue + background delivery). Same HTTP contract as Rust `apps/companion`.

## Signing (physical iPhone)

Free **Personal Team** is enough for dogfood. Entitlements: **HealthKit** + **background delivery** (`com.apple.developer.healthkit` / `healthkit.background-delivery`) — do **not** enable Clinical / Verifiable Health Records (`healthkit.access`).

If Xcode still fails on the profile, set a unique Bundle Identifier, e.g. `com.<yourname>.biofocus.companion`.

## Open & run

```bash
open apps/companion/ios/BioFocusCompanion.xcodeproj
```

1. Select scheme **BioFocusCompanion**.
2. Destination: **iOS Simulator** (loopback) or a physical iPhone (LAN).
3. For a physical device: set your Apple Development Team under Signing & Capabilities.
4. Run (⌘R).

Compile check (no Simulator runtime required):

```bash
cd apps/companion/ios
SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
mkdir -p .derivedData/swiftc-check
xcrun --sdk iphonesimulator swiftc \
  -sdk "$SDK" \
  -target arm64-apple-ios17.0-simulator \
  -parse-as-library \
  -emit-module \
  -module-name BioFocusCompanion \
  -framework Foundation -framework HealthKit -framework SwiftUI \
  IngestClient.swift HeartRateSample.swift HrvSample.swift \
  ObservationQueue.swift HealthKitSyncCoordinator.swift SamplePost.swift \
  BioFocusCompanion/BioFocusCompanionApp.swift BioFocusCompanion/ContentView.swift \
  -o .derivedData/swiftc-check/BioFocusCompanion.swiftmodule
```

## Files

| Path | Role |
| :--- | :--- |
| `BioFocusCompanion/` | SwiftUI: pairing, Auto-sync toggle, send / flush, status |
| `IngestClient.swift` | `POST /v1/ingest` + 401 / network errors |
| `HeartRateSample.swift` | HealthKit HR → Observation DTO |
| `HrvSample.swift` | HealthKit SDNN → `hrv` Observation (`sdnn_ms`) |
| `ObservationQueue.swift` | Durable pending JSON (UserDefaults), dedupe |
| `HealthKitSyncCoordinator.swift` | Observers + background delivery + flush |
| `SamplePost.swift` | Thin wrapper → coordinator |

## Pairing (Desktop → Companion)

1. Start Desktop ingest (`pnpm tauri dev`).
2. **Simulator:** Base URL `http://127.0.0.1:8787`.
3. **Physical phone:** `BIOFOCUS_INGEST_LAN=1` (+ optional `BIOFOCUS_INGEST_BIND_HOST=<lan-ip>`), copy **Base URL** from Desktop → Companion.
4. Paste pairing token.
5. Leave **Auto-sync** on (default), or tap **Send latest HR + HRV now**.

Contract: JSON array of Observations (`provider_id=com.biofocus.applehealth`, `heart_rate` and/or `hrv`) → `POST {baseURL}/v1/ingest` with Bearer token.

## Autonomy (ADR-016)

- HealthKit **observer** + **background delivery** (hourly) enqueue new samples — **no** busy-loop poll.
- Offline / Desktop down → stay in local queue; **Flush queue only** when reachable.
- HRV from many bands is **sparse** (often overnight). Missing HRV is calm / soft — HR still syncs. Not a clinical claim.
- Core accepts `sdnn_ms`-only `hrv` Observations (prefer `rmssd_ms` when present).

## Verify on Desktop

```bash
sqlite3 ~/.biofocus/data/biofocus_main.db \
  "SELECT datetime(timestamp,'unixepoch','localtime'), data_type, payload
   FROM observations
   WHERE provider_id='com.biofocus.applehealth'
   ORDER BY timestamp DESC LIMIT 10;"
```

Expect `heart_rate` and (when HealthKit has SDNN) `hrv` with `{"sdnn_ms":…}`.

## Privacy

- HealthKit **read** HR + HRV SDNN only; local Desktop ingest; no BioFocus cloud.
- Idle-safe: event / background-delivery driven.
