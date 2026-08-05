# 09. Internal REST & Ingestion API

## 1. Endpoints (Phase 2)

Default bind (skeleton): **`127.0.0.1:8787`** (`crates/ingest`). Loopback only until pairing/LAN epic.

### Companion sample path (P2-E3-T1)
- Rust: `apps/companion` — `CompanionClient::post_sample_heart_rate` / CLI `biofocus-companion-sample`
- iOS stub: `apps/companion/ios/` — HealthKit one-shot → same body
- Auth errors: client maps **`401` → unauthorized** (must not be swallowed); transport failures → network error
- Docs / smoke: `apps/companion/README.md`, `docs/12-development.md`

### `POST /v1/ingest`
- **Description:** Приём `Observation` от companion/collectors (локально). Enqueue в bounded channel; async worker пишет через `ObservationRepository::insert` (immutable append).
- **Headers:** `Authorization: Bearer <PAIRING_TOKEN>` (token from `~/.biofocus/pairing_token` or `BIOFOCUS_INGEST_TOKEN`; see `docs/10-security.md`)
- **Request Body:** `Array<Observation>`
- **Response:** `202 Accepted` → `{"status": "queued", "count": N}` (все элементы batch приняты в канал)
- **Errors:**
  - `401` missing/wrong token → `{"error":"unauthorized"}`
  - `400` invalid JSON / domain → `{"error":"invalid_json"}` (и аналоги)
  - `503` backpressure / closed channel (см. mid-batch ниже)
- **Persist:** duplicate PK при insert → явный `StorageError::DuplicateObservation` (log, **без overwrite**); HTTP `202` означает «принято в очередь», не «уже закоммичено в SQLite».
- **Status:** Host wire shipped (**P2-E1-T4**): Desktop starts/stops ingest with the app.

#### Mid-batch / queue full (contract **C**)

Bounded `try_send` per item. If a later item in the same request hits a full (or closed) channel after some earlier items were enqueued:

1. **Stop** further enqueue for this request (already-queued items stay in the channel — no rollback).
2. Respond **`503 Service Unavailable`** with counts:

```json
{"error":"queue_full","accepted":N,"rejected":M}
```

| Field | Meaning |
| :--- | :--- |
| `error` | `"queue_full"` or `"queue_closed"` |
| `accepted` | Items from this request successfully enqueued before stop |
| `rejected` | Remaining items in this request not enqueued (`accepted + rejected = batch length`) |

Same shape when the channel is full on the **first** item (`accepted: 0`). Collectors should back off and retry rejected Observations (new request).

### `GET /v1/status`
- **Description:** Статус Core / ingest для companion/debug (loopback). Shell UI uses IPC `get_status`, not this endpoint.
- **Auth:** none (loopback-only bind).
- **Response:** `200 OK` — **без** Observation / biometric payload:

```json
{"version":"0.1.0","db_status":"ok"}
```

```json
{"version":"0.1.0","db_status":"error","db_error":"…"}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `version` | string | Host / desktop package version |
| `db_status` | `"ok"` \| `"error"` | Soft-fail probe (open + WAL + migrate) |
| `db_error` | string? | Present only when `db_status` is `"error"`; short + **no absolute filesystem paths** |

- **Host:** Desktop starts ingest on app launch via `IngestConfig::load()` (pairing file / `BIOFOCUS_INGEST_TOKEN`) and stops accept + persist worker on exit.

---

## 2. Desktop Tauri IPC (Phase 1)

Boundary: **UI ↔ Tauri IPC ↔ Core Runtime ↔ SQLite**. Frontend must not import
`rusqlite`, open `biofocus_main.db`, or run SQL.

### `get_status`

- **Invoke:** `invoke("get_status")` (`@tauri-apps/api/core`)
- **Purpose:** Menubar / shell status. Probes default DB via
  `storage::default_db_path()` + `Database::open` (WAL + migrate-on-open).
- **Fails soft:** open/migrate errors become `dbStatus: "error"` (no panic).
- **Never returns** raw `Observation` or biometric payloads.

Example success:

```json
{
  "version": "0.1.0",
  "dbStatus": "ok",
  "alertLevel": "green"
}
```

Example failure:

```json
{
  "version": "0.1.0",
  "dbStatus": "error",
  "dbError": "Could not locate local data directory.",
  "alertLevel": "green"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `version` | string | Desktop package version (`CARGO_PKG_VERSION`) |
| `dbStatus` | `"ok"` \| `"error"` | Result of storage probe (Idle/Ready/Error path) |
| `dbError` | string? | Present only when `dbStatus` is `"error"`; short + **no absolute filesystem paths** (`StorageError::public_message`) |
| `alertLevel` | `"green"` \| `"yellow"` \| `"red"` | Menubar traffic-light from Core (`feature_engine::map_alert_level`); independent of `dbStatus`; default `green` without Feature evidence. No Observation / biometric fields. |

`get_status` stays lean (version / dbStatus / alertLevel only). Feature time-series for Dashboard use [`get_feature_snapshot`](#get_feature_snapshot-p4-e1-t1).

### `get_feature_snapshot` (P4-E1-T1)

- **Invoke:** `invoke("get_feature_snapshot")`
- **Purpose:** Latest cached Feature snapshot (+ optional Signals) for Dashboard charts.
- **Source:** In-memory cache updated by the Feature Worker (`CatalogAlertHook` → `feature_engine::FeatureSnapshot`). Command path is a **pure cache read** — no busy-loop, no SQLite open inside the invoke.
- **Empty:** `{ "features": [], "signals": [] }` when idle / worker not started / no evidence yet.
- **Never returns** raw Observation biometric payloads or absolute filesystem paths. Provenance is Observation **ids only**.

Example (non-empty):

```json
{
  "features": [
    {
      "featureId": "FocusScore",
      "timeWindow": { "start": 100, "end": 1000 },
      "value": 72.5,
      "provenance": ["0190…"]
    }
  ],
  "signals": [
    {
      "id": "0190…",
      "type": "High_Stress",
      "timestampStart": 900,
      "timestampEnd": 1260,
      "severity": "high"
    }
  ]
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `features[]` | object | Windowed Features (`FocusScore`, `StressIndex`, `FatigueIndex`, `ContextSwitchRate`, …) |
| `features[].featureId` | string | Stable Feature name |
| `features[].timeWindow` | `{ start, end }` | Unix seconds UTC |
| `features[].value` | number \| object | Scalar `f64` or structured JSON |
| `features[].provenance` | string[] | Observation UUIDs used as evidence |
| `signals[]` | object | Optional Signals from the same engine run |
| `signals[].type` | string | e.g. `High_Stress` |
| `signals[].severity` | `"low"` \| `"medium"` \| `"high"` \| `"critical"` | Wire form of `bio_spec::Severity` |

Core API (crate): `feature_engine::FeatureSnapshot::from_engine_output` — Features + Signals without Observation payloads.

### `get_insights` (P4-E2-T3)

- **Invoke:** `invoke("get_insights")`
- **Purpose:** Recent Insights for the Dashboard list (calm copy + evidence refs).
- **Source:** Evaluate-on-read over the same in-memory Feature snapshot cache as [`get_feature_snapshot`](#get_feature_snapshot-p4-e1-t1). Host registers `knowledge_engine::register_insights_v1` once at startup, then `KnowledgeEngine::evaluate(&features, &signals)`.
- **Empty:** `{ "insights": [] }` when idle / no matching rules / evaluate soft-fail / engine not managed. An empty/`new()` engine without registration also yields `[]`.
- **Never returns** raw Observation biometric payloads, absolute filesystem paths, or LLM text. No SQLite on the command path.

Example (non-empty):

```json
{
  "insights": [
    {
      "id": "0190…",
      "title": "Sustained stress pattern",
      "description": "Stress stayed elevated long enough in this period to raise a High_Stress signal.",
      "category": "stress",
      "evidenceList": [
        { "kind": "signal", "id": "0190…" },
        { "kind": "feature", "id": "StressIndex" }
      ],
      "actionRecommendation": "A brief pause or slower pace may help when it fits your schedule."
    }
  ]
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `insights[]` | object | Zero or more Insights from v1 product rules |
| `insights[].id` | string | Insight UUID |
| `insights[].title` | string | Calm, non-clinical title |
| `insights[].description` | string | Short explanatory copy |
| `insights[].category` | string | e.g. `stress`, `focus` |
| `insights[].evidenceList[]` | object | `kind`: `"feature"` \| `"signal"`; `id`: Feature id or Signal UUID |
| `insights[].actionRecommendation` | string? | Optional gentle suggestion |

Host contract:

```rust
let mut engine = KnowledgeEngine::new();
register_insights_v1(&mut engine)?;
let insights = engine.evaluate(&features, &signals)?;
```

### `core_ping` (legacy fallback)

Scaffold probe from P1-E3-T1. UI prefers `get_status`. Still registered for
compatibility until consumers drop the fallback path.

```json
{
  "status": "ok",
  "runtime": "runtime",
  "storage": "storage",
  "dbFile": "biofocus_main.db",
  "schemaVersion": 1
}
```

Does **not** open SQLite. Do not treat `dbFile` as a frontend open path.

### `get_pairing_token` (P2-E3-T2)

- **Invoke:** `invoke("get_pairing_token")`
- **Purpose:** Expose the local pairing Bearer token for companion share (copy / QR).
- **Source:** same resolution as ingest — `BIOFOCUS_INGEST_TOKEN` if set, else load-or-create `~/.biofocus/pairing_token` (host-side only).
- **Never returns** filesystem paths, Observation rows, or cloud credentials.

```json
{
  "token": "…64 hex…",
  "ingestBaseUrl": "http://127.0.0.1:8787",
  "fromEnv": false,
  "qrSvg": "<svg …>…</svg>"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `token` | string | Bearer value for `Authorization` |
| `ingestBaseUrl` | string | Loopback ingest base (Simulator / same Mac) |
| `fromEnv` | bool | `true` when env override is active |
| `qrSvg` | string | SVG QR encoding the token |

Errors are short UI-safe strings (no absolute paths).
