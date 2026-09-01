# iOS HealthKit companion (P5-E3 + P15 ADR-016 + P17 ADR-018)

Runnable Xcode app that posts HealthKit wearable Observations to Desktop ingest — with optional **Auto-sync** (queue + background delivery). Same HTTP contract as Rust `apps/companion`.

**Emits (when samples exist):** `heart_rate`, soft-optional `hrv` (SDNN), `step_count`, `active_energy`, `sleep_interval`, soft-optional `oxygen_saturation`. No Mi Cloud. No busy-loop.

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
  StepCountSample.swift ActiveEnergySample.swift SleepIntervalSample.swift \
  OxygenSaturationSample.swift \
  ObservationQueue.swift HealthKitSyncCoordinator.swift SamplePost.swift \
  BioFocusCompanion/BioFocusCompanionApp.swift BioFocusCompanion/ContentView.swift \
  -o .derivedData/swiftc-check/BioFocusCompanion.swiftmodule
```

## Layout

| File | Role |
| :--- | :--- |
| `HeartRateSample.swift` | HealthKit HR → Observation DTO |
| `HrvSample.swift` | HealthKit SDNN → `hrv` Observation (`sdnn_ms`) |
| `StepCountSample.swift` | Steps → `step_count` |
| `ActiveEnergySample.swift` | Active energy → `active_energy` (`kcal`) |
| `SleepIntervalSample.swift` | Sleep analysis → `sleep_interval` |
| `OxygenSaturationSample.swift` | Soft-optional SpO2 → `oxygen_saturation` |
| `ObservationQueue.swift` | Durable local queue |
| `HealthKitSyncCoordinator.swift` | Observers + background delivery + flush |
| `IngestClient.swift` | `POST /v1/ingest` |

Contract: JSON array of Observations (`provider_id=com.biofocus.applehealth`) → `POST {baseURL}/v1/ingest` with Bearer token.

## Connectivity (P28-E1-T1)

- **Test connection** — `GET /v1/status` with ≤5s timeout; shows `bind_mode` + `db_status` on success.
- **Send / Flush** — preflight status before POST (≤10s ingest timeout).
- Physical iPhone **cannot** use `127.0.0.1` — use LAN Base URL from Desktop Companion.
- Calm errors: loopback blocked, timeout/unreachable, unauthorized (check token).

### Troubleshooting (physical iPhone)

| Symptom | Fix |
| :--- | :--- |
| Timeout / unreachable | Desktop: enable LAN → restart → copy LAN URL; same Wi‑Fi; check firewall |
| Loopback message | Replace `127.0.0.1` with Desktop LAN URL |
| 401 unauthorized | Re-copy pairing token from Desktop |
| LAN on, no URL on Desktop | Set `BIOFOCUS_INGEST_BIND_HOST` to Mac LAN IP → restart |

See also `docs/12-development.md` § Companion connectivity.

## Autonomy

- HealthKit **observer** + **background delivery** (hourly) enqueue new samples — **no** busy-loop poll.
- Soft-omit sparse types (HRV / SpO2 / sleep) — never invent.
- Core validates ADR-018 payloads; reject codes `invalid_step_count` / `invalid_active_energy` / `invalid_sleep_interval` / `invalid_oxygen_saturation`.

## Privacy

- HealthKit **read** only; local Desktop ingest; no BioFocus cloud; no clinical SpO2/sleep framing.
