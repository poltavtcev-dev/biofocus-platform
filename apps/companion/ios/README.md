# iOS HealthKit companion (P5-E3-T1)

Runnable Xcode app that posts one HealthKit heart-rate `Observation` to Desktop ingest — same HTTP contract as Rust `apps/companion`.

## Open & run

```bash
open apps/companion/ios/BioFocusCompanion.xcodeproj
```

1. Select scheme **BioFocusCompanion**.
2. Destination: **iOS Simulator** (loopback) or a physical iPhone (LAN).
3. For a physical device: set your Apple Development Team under Signing & Capabilities (bundle id `com.biofocus.companion`).
4. Run (⌘R).

Compile check (no Simulator runtime required — verifies Swift sources + UI against the iOS Simulator SDK):

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
  IngestClient.swift HeartRateSample.swift SamplePost.swift \
  BioFocusCompanion/BioFocusCompanionApp.swift BioFocusCompanion/ContentView.swift \
  -o .derivedData/swiftc-check/BioFocusCompanion.swiftmodule
```

Full `xcodebuild` / Run in Xcode needs a matching **iOS Simulator runtime** for your Xcode (Xcode → Settings → Components). List destinations:

```bash
xcodebuild -project BioFocusCompanion.xcodeproj -scheme BioFocusCompanion -showdestinations
```

## Files

| Path | Role |
| :--- | :--- |
| `BioFocusCompanion.xcodeproj` | Runnable target |
| `BioFocusCompanion/` | SwiftUI shell (base URL, token, one-shot button, status) |
| `IngestClient.swift` | `POST /v1/ingest` + 401 / network errors |
| `HeartRateSample.swift` | HealthKit → Observation DTO |
| `SamplePost.swift` | One-shot orchestration (**no** polling loop) |

## Pairing (Desktop → Companion)

1. Start Desktop ingest (normal `pnpm tauri dev` / app launch).
2. **Simulator / same Mac:** Base URL `http://127.0.0.1:8787` (default in the form).
3. **Physical phone:** restart Desktop with `BIOFOCUS_INGEST_LAN=1`, then copy **Base URL** from Desktop → Companion (see `docs/12-development.md`).
4. Paste the pairing token (Show / Copy / QR on Desktop Companion).
5. Tap **Send one heart-rate sample** once.

Contract: JSON array of `Observation` (`provider_id=com.biofocus.applehealth`, `data_type=heart_rate`) → `POST {baseURL}/v1/ingest` with `Authorization: Bearer …`. Errors (401, network, HTTP) surface in the Status section — not swallowed.

## Smoke notes

### Simulator (loopback)

1. Desktop running; ingest on `127.0.0.1:8787`.
2. Run companion on Simulator; leave default Base URL; paste token.
3. Authorize heart-rate read when prompted.
4. If Status says no sample: open the **Health** app on Simulator (or Settings → Health) and add a heart-rate sample, then send again.
5. Expect Status success, or an explicit 401 / network message if Desktop is down / token wrong.
6. Optional check: `curl -s http://127.0.0.1:8787/v1/status`.

### Physical device (LAN)

1. Same Wi-Fi as the Mac; Desktop with `BIOFOCUS_INGEST_LAN=1`.
2. Copy LAN Base URL + token from Desktop Companion.
3. Run on device (signed with your team); send once.
4. Local network permission may appear — allow so the phone can reach Desktop.

## Privacy

- HealthKit **read** heart rate only (no write of clinical data; no continuous background delivery).
- Local Desktop only — no third-party analytics.
- Idle-safe: send only on explicit button tap.
