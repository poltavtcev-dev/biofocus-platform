# 07. Contracts & Serialization Standards

## 1. Canonical Payload Schema

### Observation Event Payload (JSON)
```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.applehealth",
  "data_type": "heart_rate",
  "payload": {
    "bpm": 74.0,
    "source": "Apple Watch Series 9"
  },
  "confidence": 0.98
}
```

### `context_window` payload (macOS collector, P2-E2-T1)
```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.context",
  "data_type": "context_window",
  "payload": {
    "bundle_id": "com.apple.Terminal",
    "app_name": "Terminal"
  },
  "confidence": 1.0
}
```

Metadata only: **no** window title, keystrokes, clipboard, or screenshots. Frontmost app via `NSWorkspace` (no Accessibility).

### `keystrokes` payload (macOS input aggregates, P2-E2-T2)
```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.input",
  "data_type": "keystrokes",
  "payload": {
    "count": 120,
    "window_secs": 60,
    "rate_per_min": 120.0
  },
  "confidence": 1.0
}
```

Aggregates only: **never** characters, reconstructable key codes, clipboard, or screenshots. Opt-in via `BIOFOCUS_INPUT_AGGREGATES=1`. Requires macOS Accessibility for live counts; deny → idle (no panic).

### `hrv` payload (pipeline normalize canon, P3-E1-T3)
```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcd",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.applehealth",
  "data_type": "hrv",
  "payload": {
    "rmssd_ms": 45.0,
    "sdnn_ms": 50.0,
    "pnn50": 12.5
  },
  "confidence": 1.0
}
```

Canonical after `pipeline::normalize_*`: required `rmssd_ms` (milliseconds); optional `sdnn_ms`, `pnn50` (0–100). Providers may send aliases (`rmssd` / `hrv_ms` / `hrv`) and `unit: "s"` (converted ×1000). See `crates/pipeline/src/normalize.rs`.

### `life_event` payload (Life Events v1, P6-E1-T1 / ADR-006)

Life Events are **ordinary Observations** — not a parallel DB. Shared discriminator:

| Field | Value |
| :--- | :--- |
| `data_type` | always `"life_event"` |
| `payload.kind` | v1: `"coffee"` \| `"walk"` \| `"lunch"` \| `"workout"` |
| `payload.note` | optional string (calm, user-authored; may be omitted) |
| `payload.duration_secs` | optional finite number ≥ 0 (seconds); omit for point-in-time logs |
| `provider_id` | e.g. `com.biofocus.desktop` (manual quick-log); any local provider OK |

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abcf",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.desktop",
  "data_type": "life_event",
  "payload": {
    "kind": "coffee",
    "note": "morning"
  },
  "confidence": 1.0
}
```

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abd0",
  "timestamp": 1721994000,
  "provider_id": "com.biofocus.desktop",
  "data_type": "life_event",
  "payload": {
    "kind": "walk",
    "duration_secs": 1200
  },
  "confidence": 1.0
}
```

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abd1",
  "timestamp": 1721997600,
  "provider_id": "com.biofocus.desktop",
  "data_type": "life_event",
  "payload": { "kind": "lunch" },
  "confidence": 1.0
}
```

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abd2",
  "timestamp": 1722001200,
  "provider_id": "com.biofocus.desktop",
  "data_type": "life_event",
  "payload": {
    "kind": "workout",
    "duration_secs": 2700,
    "note": "strength"
  },
  "confidence": 1.0
}
```

Validation: `bio_spec::validate_observation_payload` / `validate_life_event_payload`. Ingest rejects malformed Life Events with HTTP `400` + `{"error":"invalid_life_event"}` (batch not enqueued). Unknown payload keys are allowed (forward-compatible). Default remains local-only (no cloud Life Event sync).

## Companion → ingest (P2-E3-T1)

Same Observation JSON; companion posts a **JSON array** to `POST /v1/ingest`.

| Path | Location |
| :--- | :--- |
| Rust client + CLI | `apps/companion` (`cargo test -p companion`, `biofocus-companion-sample`) |
| iOS HealthKit stub | `apps/companion/ios/` (Swift; Xcode / device) |

Default host for same-machine / Simulator: `http://127.0.0.1:8787`. See `apps/companion/README.md`.
