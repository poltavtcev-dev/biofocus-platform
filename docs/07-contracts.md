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

## Companion → ingest (P2-E3-T1)

Same Observation JSON; companion posts a **JSON array** to `POST /v1/ingest`.

| Path | Location |
| :--- | :--- |
| Rust client + CLI | `apps/companion` (`cargo test -p companion`, `biofocus-companion-sample`) |
| iOS HealthKit stub | `apps/companion/ios/` (Swift; Xcode / device) |

Default host for same-machine / Simulator: `http://127.0.0.1:8787`. See `apps/companion/README.md`.
