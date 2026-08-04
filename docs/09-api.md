# 09. Internal REST & Ingestion API

## 1. Endpoints (Phase 2 — not implemented in Phase 1)

### `POST /v1/ingest`
- **Description:** Прием биометрии от мобильных мостов/компаньонов по локальной сети.
- **Headers:** `Authorization: Bearer <PAIRING_TOKEN>`
- **Request Body:** `Array<Observation>`
- **Response:** `202 Accepted` -> `{"status": "queued", "count": 1}`

### `GET /v1/status`
- **Description:** Проверка статуса Core Daemon и активных подключений (HTTP; Phase 2).
- **Response:** `200 OK` -> `{"version": "1.0.0", "db_status": "ok", "active_plugins": 2}`

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
