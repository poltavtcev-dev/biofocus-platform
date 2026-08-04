# 09. Internal REST & Ingestion API

## 1. Endpoints (Phase 2)

Default bind (skeleton): **`127.0.0.1:8787`** (`crates/ingest`). Loopback only until pairing/LAN epic.

### `POST /v1/ingest`
- **Description:** Приём `Observation` от companion/collectors (локально).
- **Headers:** `Authorization: Bearer <PAIRING_TOKEN>` (token from `~/.biofocus/pairing_token` or `BIOFOCUS_INGEST_TOKEN`; see `docs/10-security.md`)
- **Request Body:** `Array<Observation>`
- **Response:** `202 Accepted` → `{"status": "queued", "count": N}`
- **Errors:** `401` missing/wrong token; `400` invalid JSON / domain; `503` queue full/closed (mid-batch semantics → **P2-E1-T3**)
- **Status:** Pairing token persistence → **P2-E1-T2**. Persist Observations → **P2-E1-T3**.

### `GET /v1/status`
- **Description:** Статус Core / ingest (HTTP; wire in **P2-E1-T4**).
- **Response:** `200 OK` → `{"version": "…", "db_status": "ok"|…}` — **без** Observation payload.

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
