# 09. Internal REST & Ingestion API

## 1. Endpoints (Phase 2)

Default bind (skeleton): **`127.0.0.1:8787`** (`crates/ingest`). Loopback only until pairing/LAN epic.

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
| `db_error` | string? | Present only when `db_status` is `"error"` |

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
  "dbStatus": "ok"
}
```

Example failure:

```json
{
  "version": "0.1.0",
  "dbStatus": "error",
  "dbError": "cannot resolve home directory for default BioFocus data path"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `version` | string | Desktop package version (`CARGO_PKG_VERSION`) |
| `dbStatus` | `"ok"` \| `"error"` | Result of storage probe |
| `dbError` | string? | Present only when `dbStatus` is `"error"` |

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
