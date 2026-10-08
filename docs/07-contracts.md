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

Canonical after `pipeline::normalize_*`: at least one of `rmssd_ms` (milliseconds) **or** `sdnn_ms` (ADR-016 Apple HealthKit HRV-proxy); optional `pnn50` (0–100). Providers may send aliases (`rmssd` / `hrv_ms` / `hrv`, `sdnn`) and `unit: "s"` (converted ×1000). See `crates/pipeline/src/normalize.rs`. Features prefer RMSSD then SDNN (`mean_hrv_ms`).

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

### `now_playing` payload (Phase 12 ambient wave-1 — ADR-012 / P12-E2-T1)

Ambient media facts are **ordinary Observations** in the existing store. Wave-1 ambient source = **Now Playing / music** (weather deferred; light → ADR-015 / Phase 15).

| Field | Value |
| :--- | :--- |
| `data_type` | always `"now_playing"` |
| `provider_id` | `com.biofocus.macos.now_playing` |
| `payload.media_kind` | required coarse label: `music` \| `podcast` \| `other` \| `none` \| `unknown` |
| `payload.is_playing` | required boolean |

**Privacy:** collectors must **not** put titles, artists, albums, lyrics, playlist ids, or content-identifying artwork URLs into the Observation payload. No always-on mic. Personal self-tracking only — not workplace ambient monitoring.

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abf0",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.now_playing",
  "data_type": "now_playing",
  "payload": {
    "media_kind": "music",
    "is_playing": true
  },
  "confidence": 0.85
}
```

Enable: `BIOFOCUS_NOW_PLAYING=1` (default **off**). Emit on play-state / media-kind change or rare poll ≥5s; no busy-loop.

Validation: `bio_spec::validate_now_playing_payload` (via `validate_observation_payload`). Ingest rejects malformed Now Playing with HTTP `400` + `{"error":"invalid_now_playing"}`.

**OS probe (v1):** `SystemNowPlayingProbe` soft-fails to no emission when OS mapping is unavailable (no content-bearing MediaRemote / AppleScript in v1). Scripted/mock probes supply closed-set labels in tests.

### `git_activity` payload (Phase 13 plugin wave-2 — ADR-013 / P13-E2 shipped)

Git activity facts are **ordinary Observations** in the existing store. Wave-2 source = **Git activity aggregates** (IDE deferred — ADR-013).

| Field | Value |
| :--- | :--- |
| `data_type` | always `"git_activity"` |
| `provider_id` | `com.biofocus.macos.git` |
| `payload.activity_kind` | required coarse label: `commit` \| `checkout` \| `sync` \| `other` \| `idle` \| `unknown` |
| `payload.event_count` | optional positive integer (≥ 1) for batched aggregates; treat as `1` when absent |

**Privacy:** collectors must **not** put repo paths, remotes / clone URLs, branch names, commit SHAs / messages, diffs, authors, or file-change lists into the Observation payload. Personal self-tracking only — not workplace git monitoring. Pipeline normalize **strips** those extras if present.

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abf0",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.git",
  "data_type": "git_activity",
  "payload": {
    "activity_kind": "commit",
    "event_count": 1
  },
  "confidence": 0.9
}
```

Enable: `BIOFOCUS_GIT_ACTIVITY=1` (default **off**). Emit on activity change or rare poll ≥5s; no busy-loop.

Validation: `bio_spec::validate_git_activity_payload` (also via `validate_observation_payload`). Ingest reject code: `invalid_git_activity`.

**OS probe (ADR-014 / P14-E2 shipped):** `SystemGitActivityProbe` loads user-chosen absolute roots from `~/.biofocus/git-watched-roots.toml`. Empty / missing / unreadable file → soft-fail idle (no emit, no whole-disk scan). When the config file is **absent**, optional `BIOFOCUS_GIT_WATCHED_ROOTS` (colon- or comma-separated absolute paths) may supply roots for tests/CI; **file is SoT when present** (no env merge). Probe considers nested repos under listed roots only. Observation **payload contract unchanged** (`activity_kind` + optional `event_count` — never persist allowlist/repo paths). Logs prefer Observation `id` / root_count / repo_count; do not dump roots or discovered repo paths at default log levels. Scripted/mock probes remain for tests. **No** SQLite allowlist table / **no** migration.

### `ambient_light` payload (ADR-015 / P16-E1 shipped)

Ambient light facts are **ordinary Observations** in the existing store. ADR-015 locked the contract; collector **shipped** (P16-E1); Feature `AmbientLightShare` **shipped** (P16-E2). Weather remains deferred.

| Field | Value |
| :--- | :--- |
| `data_type` | always `"ambient_light"` |
| `provider_id` | `com.biofocus.macos.ambient_light` |
| `payload.light_kind` | required coarse label: `dark` \| `dim` \| `moderate` \| `bright` \| `unknown` |
| `payload.level` | optional bounded integer 0–100 (relative brightness band) |

**Privacy:** collectors must **not** put camera frames, screen contents / screenshots, precise geolocation, or always-on mic audio into the Observation payload. No cloud light telemetry. Personal self-tracking only — not workplace ambient monitoring. Pipeline normalize **strips** those extras if present.

```json
{
  "id": "0190ecb5-7c2a-7123-8901-23456789abf0",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.ambient_light",
  "data_type": "ambient_light",
  "payload": {
    "light_kind": "dim",
    "level": 25
  },
  "confidence": 0.8
}
```

Enable: `BIOFOCUS_AMBIENT_LIGHT=1` (default **off**). Emit on light-band change or rare poll ≥5s; no busy-loop.

Validation: `bio_spec::validate_ambient_light_payload` (also via `validate_observation_payload`). Ingest reject code: `invalid_ambient_light`.

**OS probe (P16-E1 shipped):** `SystemAmbientLightProbe` soft-fails when OS mapping unavailable (no camera / scene / screen / geo / mic capture in v1 — idle, no emit). `ScriptedAmbientLightProbe` for tests. Emit on light-band change or rare poll ≥5s; `stop_stream` joins. Host arms only when `BIOFOCUS_AMBIENT_LIGHT=1`. **No** migration.

### Wearable depth payloads (ADR-018 / P17-E2 shipped)

Companion HealthKit path keeps existing `heart_rate` + soft-optional `hrv` (ADR-016). Phase 17 emits Observation families Mi Fitness typically writes into Apple Health. **Companion emit shipped (P17-E2)**; chart ranges + Features **shipped (P17-E3)**. Existing `observations` store — **no** migration in v1.

| `data_type` | Required payload | Optional | Notes |
| :--- | :--- | :--- | :--- |
| `step_count` | `count` (≥ 0 integer) | `window_secs` (≥ 1) | Interval or sample step sum |
| `active_energy` | `kcal` (≥ 0 number) | — | Active energy kilocalories |
| `sleep_interval` | `start`, `end` (Unix s; `end` ≥ `start`) | `stage`: `asleep` \| `in_bed` \| `awake` \| `unknown` \| `asleep_core` \| `asleep_deep` \| `asleep_rem` (ADR-030; old stages stay valid) | Coarse sleep interval |
| `oxygen_saturation` | `spo2_percent` (0–100) | — | **Soft-optional** — emit only when HK has samples; non-clinical |

**Deferred:** parallel `workout` Observation family (use Life Event `workout` + steps/energy). **Forbidden:** Mi Cloud payloads; inventing SpO2/sleep when absent; clinical framing.

Validation: `bio_spec::validate_*_payload` / `validate_observation_payload`. Ingest reject codes: `invalid_step_count` / `invalid_active_energy` / `invalid_sleep_interval` / `invalid_oxygen_saturation`. Pipeline normalize strips aliases / out-of-range known types.

### Wearables v2 (ADR-030)

Additive payloads on the existing `observations` table. `provider_id` stays `com.biofocus.applehealth`. Optional `src` object: `kind` ∈ `apple_watch` | `iphone` | `xiaomi_mi_fitness` | `zepp_life` | `other_app` | `manual`. Do not store `src.name` / `device_name` / `source_name`.

| `data_type` | Required | Notes |
| :--- | :--- | :--- |
| `heart_rate`, `walking_heart_rate_average` | `bpm` | Normalize skips outside 25–250 |
| `resting_heart_rate` | `bpm` | Skips outside 25–150 |
| `hrv` | `sdnn_ms` and/or `rmssd_ms` (aliases `sdnn` / `rmssd` still accepted) | Optional `method`: `sdnn` \| `rmssd`. A method keeps only that metric. Skips outside 1–300 ms. Old payloads without `method` stay valid |
| `respiratory_rate` | `breaths_per_min` | Skips outside 4–40 |
| `sleeping_wrist_temperature` | `celsius` | Delta, skips outside −5…+5 |
| `vo2_max` | `ml_kg_min` | Skips outside 10–90 |
| `oxygen_saturation` | `spo2_percent` | After fraction→percent, skips outside 50–100 |
| `workout` | `activity_type`, `start`, `end` | |
| `exercise_time`, `stand_time` | `minutes` | |
| `stand_hour`, `mindful_session` | `start`, `end` | |
| `distance_walking_running` | `meters` | |
| `basal_energy` | `kcal` | |
| `source_deletion` | `target_id` (UUID) | Only `provider_id = com.biofocus.applehealth`. Read-side filter, same idea as `life_event_retraction`. No row delete |

`POST /v1/ingest` allowlist is the health types above plus `step_count`, `active_energy`, `sleep_interval`, `life_event`. Anything else, including `life_event_retraction` and Mac collector types, is `403 forbidden_data_type`. Body > 2 MB or more than 1000 Observations → `413 payload_too_large`. Plain HTTP on LAN is unchanged (ADR-005): the token is visible on Wi-Fi. Use a trusted network.

Source choice is on read, after normalize. Storage keeps every row. Default order is `apple_watch`, `xiaomi_mi_fitness`, `zepp_life`, `iphone`, `other_app`, `manual`, overridable in `source-priority.toml` (`order = [...]`) under `BIOFOCUS_HOME` or `~/.biofocus/`. Cumulative types (steps, energy, distance, exercise time, stand time) keep one source per UTC day and are not summed. Heart rate, HRV, SpO2, and respiratory rate keep one source per 15-minute bucket. Sleep keeps one source per night; stages come only from that source.

### `notification_event` payload (ADR-019 / P18 shipped · ADR-020 live probe shipped)

Phase 18 notification pressure Observation family. **Contracts locked (ADR-019)**; collector + Feature **shipped** (P18-E2/E3). **ADR-020** live `SystemNotificationEventProbe` mapping **shipped** (P19-E2): read-only **usernoted** NC SQLite; **payload unchanged**; soft-fail when mapping unavailable / TCC denied. Existing `observations` store — **no** migration in v1.

| Field | Rule |
| :--- | :--- |
| `data_type` | always `"notification_event"` |
| `provider_id` | `com.biofocus.macos.notifications` |
| `payload.count` | **Required.** Integer ≥ 1 (usually `1`; >1 when coalesced) |
| `payload.category` | **Optional.** `communication` \| `calendar` \| `system` \| `media` \| `social` \| `other` \| `unknown` |
| `payload.interruption_level` | **Optional.** `passive` \| `active` \| `time_sensitive` \| `critical` \| `unknown` |
| `payload.app_kind` | **Optional.** `messaging` \| `mail` \| `calendar` \| `social` \| `system` \| `other` \| `unknown` |

```json
{
  "provider_id": "com.biofocus.macos.notifications",
  "data_type": "notification_event",
  "payload": {
    "count": 1,
    "category": "communication",
    "interruption_level": "active",
    "app_kind": "messaging"
  },
  "confidence": 0.85
}
```

Enable: `BIOFOCUS_NOTIFICATION_EVENTS=1` (default **off**). Emit on delivery / coalesced change or rare poll ≥5s; no busy-loop.

**Forbidden:** notification body / title / subtitle / message text / screenshots / attachments / userInfo dumps / free-form app display names.

Validation: `bio_spec::validate_notification_event_payload` (also via `validate_observation_payload`). Ingest reject code: `invalid_notification_event`. Pipeline normalize strips any accidental content keys (`title` / `body` / `subtitle` / `message` / `screenshot` / `userInfo`).

**OS probe (P19-E2 shipped · P19-E3 dogfood):** Live `SystemNotificationEventProbe` maps macOS **usernoted** Notification Center SQLite (`~/Library/Group Containers/group.com.apple.usernoted/db2/db`, fallback `$DARWIN_USER_DIR/com.apple.notificationcenter/db2/db`) → ADR-019 payloads. **Allowlisted SELECT only:** `record.delivered_date` + `app.identifier` (bundle → closed-set labels in-memory). **Never** `SELECT` `record.data` / title / body / subtitle / message / userInfo / attachments. Soft-fail idle when DB missing, TCC/authorization denies access, or schema lacks allowlisted columns. First successful open sets a watermark (no historical dump); later polls emit coalesced `count` for new deliveries. `ScriptedNotificationEventProbe` / `with_db_path` fixtures for tests. Emit on identity change or rare poll ≥5s; `stop_stream` joins. Host arms only when `BIOFOCUS_NOTIFICATION_EVENTS=1`. **No** migration. Dogfood runbook (FDA / verify Feature): `docs/12-development.md` § Notification events dogfood.

## Companion → ingest (P2-E3-T1)

Same Observation JSON; companion posts a **JSON array** to `POST /v1/ingest`.

| Path | Location |
| :--- | :--- |
| Rust client + CLI | `apps/companion` (`cargo test -p companion`, `biofocus-companion-sample`) |
| iOS HealthKit stub | `apps/companion/ios/` (Swift; Xcode / device) |

Default host for same-machine / Simulator: `http://127.0.0.1:8787`. See `apps/companion/README.md`.
