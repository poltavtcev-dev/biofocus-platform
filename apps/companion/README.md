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
- **Same Mac (CLI / iOS Simulator):** `http://127.0.0.1:8787` — matches current Desktop bind (loopback only).
- **Physical iPhone on LAN:** needs Desktop ingest reachable on LAN (not shipped in E1 bind; documented for later pairing / host config). Do not use cloud.

## iOS stub
See [`ios/README.md`](ios/README.md) — HealthKit one-shot HR → same JSON array body. Paste the token from Desktop **Companion** (Copy / QR).

## Out of scope
Feature pipeline, dashboard, cloud accounts, changing ingest HTTP contract.

## Future: more wearables

Keep the Observation HTTP contract; add phone companion bridges later. Physical phone → Desktop needs **LAN ingest** (not only `127.0.0.1`). See `docs/PROJECT_CANVAS.md` § Wearable / companion.

