# iOS HealthKit stub (P2-E3-T1)

Swift sources sketch the **same** HTTP contract as the Rust `companion` client:

1. Read one `HKQuantityTypeIdentifier.heartRate` sample (user-authorized).
2. Map to `Observation` JSON (`provider_id=com.biofocus.applehealth`, `data_type=heart_rate`, payload `{bpm, source?}`).
3. `POST` JSON **array** to `{baseURL}/v1/ingest` with `Authorization: Bearer <pairing_token>`.
4. Surface network failures and `401` to the user (no silent swallow).

This is **not** a full Xcode project / App Store target yet — drop into an iOS app target when wiring UI.

## Files
- `IngestClient.swift` — URLSession POST + error mapping
- `HeartRateSample.swift` — HealthKit → Observation DTO
- `SamplePost.swift` — one-shot orchestration (no polling loop)

## Config
- `BIOFOCUS_INGEST_BASE_URL` default for Simulator: `http://127.0.0.1:8787`
- Pairing token: paste from Desktop shell (**Companion** → Copy, or scan **Show QR**)

## Privacy
- Request HealthKit heart-rate read only; do not send RR intervals or identifiers beyond the Observation contract.
- Local Desktop only — no third-party analytics.
