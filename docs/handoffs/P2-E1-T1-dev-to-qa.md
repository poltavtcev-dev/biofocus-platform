# Dev → QA: P2-E1-T1

## Meta
- **Task ID:** P2-E1-T1
- **Title:** Local ingest HTTP skeleton (`POST /v1/ingest`)
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P2-E1 / P2-E1-T1; brief `docs/handoffs/P2-E1-T1-pm-brief.md`
- **Branch:** `phase/2-ingest-http`

## What changed
- Summary:
  - Новый crate `ingest`: Axum HTTP server на **только** `127.0.0.1`.
  - `POST /v1/ingest` принимает JSON array `Observation` (`bio-spec`), требует `Authorization: Bearer <token>`.
  - Success → **202** + `{"status":"queued","count":N}`; Observations уходят в bounded `runtime::observation_channel` (без SQLite).
  - Missing/wrong token → **401**; invalid JSON / domain validation (напр. confidence) → **400**; channel full/closed → **503** (скелет; формализация backpressure → T3).
  - Токен: env `BIOFOCUS_INGEST_TOKEN` или const `biofocus-dev-ingest-token`. Порт по умолчанию **8787** (`DEFAULT_INGEST_PORT`); в тестах — ephemeral `0`.
  - CI: `cargo test -p ingest` добавлен в `rust-core` job.
- Crates / files:
  - `crates/ingest/` (new: `lib`, `config`, `auth`, `routes`, `server`, `error`)
  - `crates/ingest/tests/ingest_http.rs` (new)
  - `Cargo.toml` (workspace member + `axum`/`http`/`tower` deps; tokio `net`)
  - `Cargo.lock` (updated)
  - `.github/workflows/ci.yml`
  - `crates/runtime/src/lib.rs` (doc: HTTP → `ingest` crate)

## How to verify (commands)
```bash
cargo check --workspace --exclude desktop
cargo test -p ingest
# optional regression:
cargo test -p bio-spec -p runtime -p storage -p ingest
```

Ожидание: все тесты `ingest` зелёные (unit auth + integration bind/auth/202/bad JSON + live loopback HTTP).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: сервер биндится только на `127.0.0.1` (порт `DEFAULT_INGEST_PORT=8787` или `0` в тестах)
- [ ] AC2: `POST /v1/ingest` + Bearer → 202 `{"status":"queued","count":N}`; без/неверный токен → 401; битый JSON / invalid Observation → 400
- [ ] AC3: нет busy-loop после bind (tokio/`axum::serve` accept)
- [ ] AC4: тесты auth reject, 202 happy path, bad JSON (без Tauri) — зелёные
- [ ] AC5: нет persist в SQLite; нет Feature/pipeline; нет Tauri UI
- [ ] Global DoD: нет `unwrap`/`expect` в `crates/ingest/src/`; UI↛DB; glossary `Observation`

## Risks / not covered
- При `503 queue_full` mid-batch часть Observation уже может быть в канале (частичный enqueue) — формализовать в **P2-E1-T3**.
- Pairing token persistence / генерация → **P2-E1-T2**.
- Host wire (desktop start/stop) + `GET /v1/status` → **P2-E1-T4**.
- LAN bind / QR / iOS / collector — out of scope.

## Notes for QA
- Константы: `ingest::DEFAULT_INGEST_PORT` (8787), `ingest::DEFAULT_SKELETON_TOKEN`, `ingest::INGEST_TOKEN_ENV` (`BIOFOCUS_INGEST_TOKEN`), `ingest::INGEST_BIND_HOST` (`127.0.0.1`).
- Публичный API: `bind_loopback`, `serve_listener`, `serve_with_shutdown`, `ingest_router`, `IngestConfig::from_env`.
- Live-тест поднимает ephemeral port и бьёт `reqwest` по `http://127.0.0.1:<port>/v1/ingest`.
