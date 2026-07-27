---

### `docs/09-api.md`

```markdown
# 09. Internal REST & Ingestion API

## 1. Endpoints

### `POST /v1/ingest`
- **Description:** Прием биометрии от мобильных мостов/компаньонов по локальной сети.
- **Headers:** `Authorization: Bearer <PAIRING_TOKEN>`
- **Request Body:** `Array<Observation>`
- **Response:** `202 Accepted` -> `{"status": "queued", "count": 1}`

### `GET /v1/status`
- **Description:** Проверка статуса Core Daemon и активных подключениях.
- **Response:** `200 OK` -> `{"version": "1.0.0", "db_status": "ok", "active_plugins": 2}`