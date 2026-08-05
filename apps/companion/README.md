# BioFocus Companion (P2-E3-T1)

Minimal path: sample **`heart_rate`** `Observation` → Desktop **`POST /v1/ingest`** with Bearer pairing token.

| Piece | Role |
| :--- | :--- |
| Rust crate `companion` | Contract client + CLI smoke (`biofocus-companion-sample`) |
| `ios/` | Swift HealthKit stub (same HTTP contract; needs Xcode / device) |

## Prerequisites
- Desktop BioFocus running (ingest on `127.0.0.1:8787`) **or** another host that serves the same API
- Pairing token from Desktop shell (**Companion** → Copy / QR) or `~/.biofocus/pairing_token` / `BIOFOCUS_INGEST_TOKEN`

## Rust sample (CI-friendly)

```bash
# From repo root — unit/integration (spins ephemeral loopback ingest)
cargo test -p companion

# Against a live Desktop ingest:
export BIOFOCUS_INGEST_TOKEN="$(cat ~/.biofocus/pairing_token)"
cargo run -p companion --bin biofocus-companion-sample -- 74
# optional: BIOFOCUS_INGEST_URL=http://127.0.0.1:8787
```

Exit codes: `0` queued · `2` network · `3` unauthorized (`401`) · `1` other.

## Reachability
- **Same Mac (CLI / iOS Simulator):** `http://127.0.0.1:8787` — default Desktop bind (loopback).
- **Physical iPhone on LAN:** set `BIOFOCUS_INGEST_LAN=1` on Desktop, then use `base_url_hints[0]` from `GET /v1/status` or pairing IPC (`ingestBaseUrl`). Companion UI polish → P5-E2-T1. Do not use cloud.

## iOS stub
See [`ios/README.md`](ios/README.md) — HealthKit one-shot HR → same JSON array body. Paste the token from Desktop **Companion** (Copy / QR).

## Out of scope
Feature pipeline, dashboard, cloud accounts, changing ingest HTTP contract.

## Future: more wearables

Keep the Observation HTTP contract; add phone companion bridges later. Physical phone → Desktop needs **LAN ingest** (not only `127.0.0.1`). See `docs/PROJECT_CANVAS.md` § Wearable / companion.

