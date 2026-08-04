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

Metadata only: **no** window title, keystrokes, clipboard, or screenshots. Frontmost app via `NSWorkspace` (no Accessibility). Window title / input aggregates → later E2 tasks.
