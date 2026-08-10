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

### `calendar_event` payload (local Calendar dogfood, P6-E3-T1)

Calendar / meeting facts are **ordinary Observations** — same store as Life Events / collectors. No cloud OAuth (Google/Outlook). Dogfood source: opt-in local `.ics` file.

| Field | Value |
| :--- | :--- |
| `data_type` | always `"calendar_event"` |
| `provider_id` | `com.biofocus.macos.calendar` (local ICS plugin) |
| `payload.uid` | stable local id (ICS `UID`) |
| `payload.start` / `payload.end` | Unix seconds UTC; `end >= start` |
| `payload.all_day` | optional bool |
| `payload.busy` | optional bool (`false` when ICS `TRANSP:TRANSPARENT`) |

**Privacy:** collectors must **not** put event titles, descriptions, locations, or attendees into the Observation payload. Pipeline normalize strips those keys if present. Logs must not print titles/bodies (channel-full logs use Observation `id` only).

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abe0",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.calendar",
  "data_type": "calendar_event",
  "payload": {
    "uid": "meet-standup@local",
    "start": 1721990400,
    "end": 1721994000,
    "all_day": false,
    "busy": true
  },
  "confidence": 1.0
}
```

Enable: `BIOFOCUS_CALENDAR=1` and `BIOFOCUS_CALENDAR_ICS=/path/to/calendar.ics` (default **off**). Rare poll ≥60s; emits each `(uid,start,end)` once within a lookaround horizon (past 24h / future 48h).

Validation: `bio_spec::validate_calendar_event_payload` (via `validate_observation_payload`). Ingest rejects malformed Calendar Events with HTTP `400` + `{"error":"invalid_calendar_event"}`.

### `browser_category` payload (Plugin wave-1 — ADR-010 / P10-E2-T1)

Browser context facts are **ordinary Observations** in the existing store. Wave-1 source = **Browser categories** (IDE/Git deferred).

| Field | Value |
| :--- | :--- |
| `data_type` | always `"browser_category"` |
| `provider_id` | `com.biofocus.macos.browser` |
| `payload.category` | required coarse label: `work` \| `communication` \| `entertainment` \| `reference` \| `shopping` \| `unknown` |
| `payload.browser_bundle_id` | optional frontmost browser bundle id |

**Privacy:** collectors must **not** put full URLs, query strings, page titles, form content, keystrokes, or screenshots into the Observation payload. In-process category mapping stays out of SQLite fields. Personal self-tracking only — not workplace monitoring.

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abf0",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.browser",
  "data_type": "browser_category",
  "payload": {
    "category": "work",
    "browser_bundle_id": "com.apple.Safari"
  },
  "confidence": 0.9
}
```

Enable: `BIOFOCUS_BROWSER_CATEGORIES=1` (default **off**). Emit on category change or rare poll ≥5s; no busy-loop.

Validation: `bio_spec::validate_browser_category_payload` (via `validate_observation_payload`). Ingest rejects malformed Browser Categories with HTTP `400` + `{"error":"invalid_browser_category"}`.

**OS probe (v1):** when frontmost is a known browser bundle, emits `category: "unknown"` + `browser_bundle_id` (no URL capture). Richer category mapping without persisting URLs may arrive later; scripted/mock probes supply closed-set labels in tests.

## Companion → ingest (P2-E3-T1)

Same Observation JSON; companion posts a **JSON array** to `POST /v1/ingest`.

| Path | Location |
| :--- | :--- |
| Rust client + CLI | `apps/companion` (`cargo test -p companion`, `biofocus-companion-sample`) |
| iOS HealthKit stub | `apps/companion/ios/` (Swift; Xcode / device) |

Default host for same-machine / Simulator: `http://127.0.0.1:8787`. See `apps/companion/README.md`.
