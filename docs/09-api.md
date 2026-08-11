# 09. Internal REST & Ingestion API

## 1. Endpoints (Phase 2)

Default bind: **`127.0.0.1:8787`** (`crates/ingest`). LAN-reachable bind is **opt-in** (`BIOFOCUS_INGEST_LAN=1` and/or `BIOFOCUS_INGEST_BIND_HOST` — see `docs/10-security.md` §1.1 / ADR-005). Bearer auth unchanged.

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
  - `400` malformed Life Event Observation (`data_type: "life_event"`) → `{"error":"invalid_life_event"}` (ADR-006; whole batch rejected before enqueue)
  - `400` malformed Calendar Event Observation (`data_type: "calendar_event"`) → `{"error":"invalid_calendar_event"}` (P6-E3-T1; whole batch rejected before enqueue)
  - `503` backpressure / closed channel (см. mid-batch ниже)
- **Life Events (P6-E1-T1):** same `Array<Observation>` body; `data_type` must be `"life_event"` with `payload.kind` ∈ `coffee` / `walk` / `lunch` / `workout`. Shape + examples: `docs/07-contracts.md`. Persisted via existing Observation repository (no parallel store).
- **Calendar events (P6-E3-T1):** same array body; `data_type: "calendar_event"` with `uid` / `start` / `end` (optional `all_day` / `busy`). Prefer the opt-in local ICS collector over manual POST for dogfood. Shape: `docs/07-contracts.md`.
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
- **Description:** Статус Core / ingest для companion/debug. Shell UI uses IPC `get_status`, not this endpoint.
- **Auth:** none (intended for local/debug; prefer loopback; when LAN opt-in is on the endpoint is reachable on the LAN without Bearer — do not expose Observation data here).
- **Response:** `200 OK` — **без** Observation / biometric payload, tokens, or absolute DB paths:

```json
{"version":"0.1.0","db_status":"ok","bind_mode":"loopback","base_url_hints":["http://127.0.0.1:8787"]}
```

```json
{"version":"0.1.0","db_status":"ok","bind_mode":"lan","base_url_hints":["http://192.168.1.40:8787"]}
```

```json
{"version":"0.1.0","db_status":"error","db_error":"…","bind_mode":"loopback","base_url_hints":["http://127.0.0.1:8787"]}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `version` | string | Host / desktop package version |
| `db_status` | `"ok"` \| `"error"` | Soft-fail probe (open + WAL + migrate) |
| `db_error` | string? | Present only when `db_status` is `"error"`; short + **no absolute filesystem paths** |
| `bind_mode` | `"loopback"` \| `"lan"` | Reflects TCP bind (default loopback; LAN when opt-in / non-loopback host) |
| `base_url_hints` | string[] | Usable `http://<host>:<port>` URLs (primary first). Loopback mode → `http://127.0.0.1:<port>`. LAN + `0.0.0.0` → best-effort primary LAN IPv4; LAN + explicit bind host → that host. Hints derived on status build (no spin). |

- **Host:** Desktop starts ingest on app launch via `IngestConfig::load()` (pairing file / `BIOFOCUS_INGEST_TOKEN`) and stops accept + persist worker on exit.
- **Read the hint:** `curl -s http://127.0.0.1:8787/v1/status` (or the LAN URL once known) → use `base_url_hints[0]` as companion ingest base. Pairing IPC exposes the same fields (camelCase) — see [`get_pairing_token`](#get_pairing_token-p2-e3-t2).

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
      "provenance": ["0190…"],
      "confidence": 1.0,
      "factors": [
        { "id": "typing", "label": "Typing activity", "share": 0.4 },
        { "id": "stability", "label": "App stability", "share": 0.35 },
        { "id": "hrv", "label": "Heart-rate variability", "share": 0.25 }
      ]
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
| `features[]` | object | Windowed Features (`FocusScore`, `StressIndex`, `FatigueIndex`, `ContextSwitchRate`, `RecoveryScore`, `DistractionScore`, …) |
| `features[].featureId` | string | Stable Feature name |
| `features[].timeWindow` | `{ start, end }` | Unix seconds UTC |
| `features[].value` | number \| object | Scalar `f64` or structured JSON |
| `features[].provenance` | string[] | Observation UUIDs used as evidence |
| `features[].confidence` | number | `[0.0, 1.0]` derived Feature confidence (ADR-007); data quality, not clinical |
| `features[].factors` | object[]? | Optional calm explanation factors (P7-E2). Omitted when empty / catalog does not emit yet. Each: `id` (stable), `label` (calm), `share` ∈ `[0.0, 1.0]` (renormalized; present shares sum ≈ 1.0). Not LLM prose; not clinical. |
| `signals[]` | object | Optional Signals from the same engine run |
| `signals[].type` | string | e.g. `High_Stress` |
| `signals[].severity` | `"low"` \| `"medium"` \| `"high"` \| `"critical"` | Wire form of `bio_spec::Severity` |

Core API (crate): `feature_engine::FeatureSnapshot::from_engine_output` — Features + Signals without Observation payloads.

### Chart ranges + `get_feature_series` (ADR-018 / P17-E3)

Phase 17 Dashboard range picker (closed set): **`1h` / `8h` / `12h` / `1d` / `1w`**.

| Surface | IPC | Behavior |
| :--- | :--- | :--- |
| Snapshot list / “latest” cards | [`get_feature_snapshot`](#get_feature_snapshot-p4-e1-t1) | **Latest** Feature per `featureId` (collapsed from multi-window cache). Not a dump of every window. |
| Chart series | `get_feature_series` (**shipped P17-E3**) | **Recompute-on-read** from local Observations for the selected span (ADR-008); coarser `stepSecs` for longer ranges |

**Default steps (v1):** `1h`→60s · `8h`/`12h`→300s · `1d`→900s · `1w`→3600s.

```text
invoke("get_feature_series", {
  range: "1h" | "8h" | "12h" | "1d" | "1w",
  featureIds?: string[]
})
```

Success shape:

```json
{
  "range": "1d",
  "stepSecs": 900,
  "window": { "start": 1721904000, "end": 1721990400 },
  "features": [ /* Feature wire objects; multi-window series */ ]
}
```

- Empty / thin history → `{ "features": [] }` (calm).
- Unknown `range` → empty series (soft-fail).
- UI ↛ SQLite; host loads Observations + runs FeatureEngine with the range step.
- Optional in-process memo only — **no** Feature-history SQLite table in v1.
- Never returns raw Observation biometric payloads or absolute filesystem paths.
- LLM must not invent series points (L5 interpret-only).
- Optional `featureIds` filters emitted Features (omit / empty = all catalog Features for the span).
### `get_insights` (P4-E2-T3 / P8-E2 / P8-E3)

- **Invoke:** `invoke("get_insights")`
- **Purpose:** Recent Insights for the Dashboard list (calm copy + evidence refs).
- **Source:** Evaluate-on-read over the same in-memory Feature snapshot cache as [`get_feature_snapshot`](#get_feature_snapshot-p4-e1-t1). Host registers `knowledge_engine::register_insights_v1` once at startup, then `KnowledgeEngine::evaluate_with_pattern(&features, &signals, &pattern)` (pattern inputs may include a bounded recompute-on-read baseline series).
- **Empty:** `{ "insights": [] }` when idle / no matching rules / thin baseline history / evaluate soft-fail / engine not managed. An empty/`new()` engine without registration also yields `[]`. Dashboard shows a calm empty state (no error noise).
- **Never returns** raw Observation biometric payloads, absolute filesystem paths, or LLM text. No SQLite on the command path (UI ↛ DB).

**Pattern Discovery v1 (ADR-008):** Baseline / multi-day rules stay on this IPC. Core may recompute a **bounded** Feature series from local Observations for comparison (recompute-on-read); optional in-process memo only — **no** Feature-history SQLite table and **no** UI→DB. Thin history or low confidence → omit the Insight. Copy remains calm personal observation (not clinical). Shipped rule: `focus_vs_recent_baseline_v1` (current `FocusScore` vs mean of ≤7 prior UTC afternoon windows; category `pattern`). Dashboard lists Insights as returned and shows a calm category affordance (`Pattern` / `Focus` / `Stress`).

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
| `insights[].category` | string | e.g. `pattern`, `stress`, `focus` |
| `insights[].evidenceList[]` | object | `kind`: `"feature"` \| `"signal"`; `id`: Feature id or Signal UUID |
| `insights[].actionRecommendation` | string? | Optional gentle suggestion (thin hint — **not** L4; see ADR-009 / `get_recommendations`) |

Host contract:

```rust
let mut engine = KnowledgeEngine::new();
register_insights_v1(&mut engine)?;
// Snapshot-only rules: evaluate(&features, &signals)
// Pattern Discovery: evaluate_with_pattern(&features, &signals, &pattern)
let insights = engine.evaluate_with_pattern(&features, &signals, &pattern)?;
```

### `get_recommendations` (P9-E3-T1 / ADR-009)

- **Invoke:** `invoke("get_recommendations")`
- **Purpose:** Calm Evidence-backed Recommendations for Dashboard / Insights-adjacent surface (L4).
- **Source:** Evaluate-on-read in `knowledge-engine` after Insights on the same in-memory Feature snapshot (+ pattern baseline inputs as `get_insights`). Host registers `register_insights_v1` + `register_recommendations_v1` at startup. Empty / no-match / low confidence / soft-fail → `{ "recommendations": [] }`.
- **Never returns** raw Observation biometric payloads, absolute filesystem paths, or LLM-invented actions. No SQLite on the command path (UI ↛ DB). No Recommendation persistence (ADR-009).
- **Shipped rule:** `focus_dip_pace_hint_v1` — when pattern Insight `focus_vs_recent_baseline_v1` indicates Focus lower than recent average (and FocusScore confidence gate passes), emit a pace/pause hint with Evidence `Feature FocusScore` + `Insight <id>`.
- **Dashboard:** Suggestions section (Insights-adjacent); title/suggestion as Core returns. QA mocks: `?mockRecommendations=empty|ready|pace|error` (see `apps/desktop/README.md`).

Example (non-empty, illustrative):

```json
{
  "recommendations": [
    {
      "id": "0190…",
      "title": "A gentler pace may help",
      "suggestion": "If it fits your schedule, a short pause or slightly slower pace may help when focus looks lower than your recent average.",
      "category": "pace",
      "evidenceList": [
        { "kind": "feature", "id": "FocusScore" },
        { "kind": "insight", "id": "0190…" }
      ]
    }
  ]
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `recommendations[]` | object | Zero or more Recommendations from v1 rules |
| `recommendations[].id` | string | Recommendation UUID |
| `recommendations[].title` | string | Calm, non-clinical title |
| `recommendations[].suggestion` | string | Optional personal action hint (not medical advice) |
| `recommendations[].category` | string | e.g. `pace`, `focus` |
| `recommendations[].evidenceList[]` | object | `kind`: `"feature"` \| `"signal"` \| `"insight"`; `id` accordingly |

### report-engine (crate API, P4-E3-T1 / T2)

Offline builder + optional local LLM interpret.

```rust
use report_engine::{
    build_report, interpret_report, LocalLlmConfig, ReportDocument,
};

let doc = build_report(&features, &insights)?;
// Opt-in only — default OFF; never call on app startup.
let config = LocalLlmConfig::from_env(); // or LocalLlmConfig { enabled: true, .. }
let interpreted = interpret_report(&doc, &config).await?; // Err(LocalLlmDisabled) if off
```

| Output / API | Notes |
| :--- | :--- |
| `markdown` | Calm deterministic summary: `# BioFocus report`, Features table (sorted by id / window), Insights sections (sorted by UUID). Empty inputs → short “Nothing to summarize…” body. |
| `llm_prompt` | Same markdown wrapped with interpret-only instructions (no Feature math, non-clinical). |
| `interpret_report` / `interpret_llm_prompt` | Opt-in HTTP to OpenAI-compatible `/chat/completions` (default Ollama `http://127.0.0.1:11434/v1`). Sends prompt string only. Timeout + `thiserror`. |

Env: `BIOFOCUS_LOCAL_LLM=1`, optional `BIOFOCUS_LOCAL_LLM_BASE_URL`, `BIOFOCUS_LOCAL_LLM_MODEL`, `BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS`. Scalars render as fixed 4-decimal strings; object Feature values as compact JSON. Does not open SQLite. Builder path has no HTTP; LLM path is explicit + disabled by default.

### Prompt packs (P11-E2-T1 / ADR-011)

Named/versioned **in-process** templates in `report-engine`. No SQLite; no network from the pack builder.

```rust
use report_engine::{
    build_report_with_pack, default_prompt_pack, list_prompt_packs,
    DEFAULT_PROMPT_PACK_ID, DEFAULT_PROMPT_PACK_VERSION, ReportDocument,
};

let pack = default_prompt_pack(); // biofocus.default @ 1
assert_eq!(pack.id, DEFAULT_PROMPT_PACK_ID);
assert_eq!(pack.version, DEFAULT_PROMPT_PACK_VERSION);

let doc: ReportDocument = build_report_with_pack(
    pack.id,
    pack.version,
    &features,
    &insights,
    &recommendations,
)?;
// Unknown id/version → Err(UnknownPromptPack). Empty inputs → Ok(calm minimal).
let _ = list_prompt_packs(); // in-process registry (v1: default only)
```

| API | Notes |
| :--- | :--- |
| `build_report_with_pack(id, version, features, insights, recommendations)` | Offline `markdown` + `llm_prompt`. Default pack includes Features / Insights / Recommendations sections (soft empty when missing). No Feature / Recommendation math. |
| `DEFAULT_PROMPT_PACK_ID` / `VERSION` | `biofocus.default` / `1` — calm, non-clinical; `llm_prompt` forbids inventing metrics, Evidence, Insights, Recommendations, or actions. |
| `list_prompt_packs` / `default_prompt_pack` | In-process registry helpers. |
| `ReportEngineError::UnknownPromptPack` | Typed miss for id/version. |

Desktop host (**P11-E3-T1**) uses `build_report_with_pack` with default pack `biofocus.default` @ `1` inside `generate_report`. Phase 4 `build_report(features, insights)` remains available for Core callers that do not need Recommendations / packs.

### `get_local_llm_status` (P11-E3-T1)

- **Invoke:** `invoke("get_local_llm_status")` — config-only provider status. Safe on Dashboard open / soft refresh.
- **Purpose:** Calm `disabled` / `ready` / `error` reflecting host env (`BIOFOCUS_LOCAL_LLM*`). **No** HTTP probe, **no** interpret, **no** secrets/tokens, **no** SQLite.
- Does **not** generate a report. Interpretation still happens only via explicit `generate_report`.

```json
{
  "status": "disabled",
  "detail": "Local AI is optional and currently off.",
  "packId": "biofocus.default",
  "packVersion": "1"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `status` | string | `"disabled"` \| `"ready"` \| `"error"` |
| `detail` | string | Calm one-liner for Dashboard |
| `model` | string? | Present when `ready` (model id — not a secret) |
| `packId` / `packVersion` | string | Default pack used by `generate_report` |

### `generate_report` (P4-E3-T3 / P11-E3-T1)

- **Invoke:** `invoke("generate_report")` — **explicit user action only** (Dashboard «Generate report»). Never on app / Dashboard open or soft poll.
- **Purpose:** Offline `build_report_with_pack("biofocus.default", "1", …)` from cached Feature snapshot + evaluate-on-read Insights + Recommendations; optional `interpret_report` when `BIOFOCUS_LOCAL_LLM` is enabled on the host process.
- **UI ↛ SQLite / Core crates.** Soft-fails LLM errors into `llmStatus` so markdown still returns. When disabled: no network.

```json
{
  "markdown": "# BioFocus report\n…",
  "llmPrompt": "…",
  "llmStatus": "disabled"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `markdown` | string | Deterministic offline report (default pack) |
| `llmPrompt` | string | Prompt for optional local interpret |
| `interpretation` | string? | Present when `llmStatus == "ok"` |
| `llmStatus` | string | `"disabled"` \| `"ok"` \| `"error"` \| `"timeout"` |
| `llmError` | string? | Calm short detail when error/timeout |

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
  "bindMode": "loopback",
  "baseUrlHints": ["http://127.0.0.1:8787"],
  "fromEnv": false,
  "qrSvg": "<svg …>…</svg>"
}
```

LAN opt-in (`BIOFOCUS_INGEST_LAN=1`) example:

```json
{
  "token": "…",
  "ingestBaseUrl": "http://192.168.1.40:8787",
  "bindMode": "lan",
  "baseUrlHints": ["http://192.168.1.40:8787"],
  "fromEnv": false,
  "qrSvg": "<svg …>…</svg>"
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `token` | string | Bearer value for `Authorization` |
| `ingestBaseUrl` | string | Primary hint (`baseUrlHints[0]`); loopback when bind is local-only |
| `bindMode` | `"loopback"` \| `"lan"` | Same meaning as HTTP `/v1/status` `bind_mode` |
| `baseUrlHints` | string[] | Same as HTTP `base_url_hints` (derived on invoke; no spin) |
| `fromEnv` | bool | `true` when env override is active |
| `qrSvg` | string | SVG QR encoding the token |

Errors are short UI-safe strings (no absolute paths). Never returns Observation payloads or DB paths.

### `log_life_event` / `list_recent_life_events` (P6-E2-T1)

- **Invoke:** `invoke("log_life_event", { kind })` / `invoke("list_recent_life_events", { limit? })`
- **Purpose:** Menubar quick-log for v1 Life Event Observations (ADR-006). Same Observation store as ingest — no parallel table. UI ↛ SQLite.
- **Contract:** `data_type: "life_event"`, `provider_id: "com.biofocus.desktop"`, `payload.kind` ∈ `coffee` / `walk` / `lunch` / `workout`. Validated with `bio_spec::validate_observation_payload` before insert via `ObservationRepository`.
- **Idle-safe:** on-demand invoke only (no poll loop for logging).

`log_life_event` success:

```json
{
  "id": "0190…",
  "kind": "coffee",
  "timestamp": 1721990400,
  "providerId": "com.biofocus.desktop"
}
```

`list_recent_life_events` success (newest first; default limit 8, max 32):

```json
[
  {
    "id": "0190…",
    "kind": "coffee",
    "timestamp": 1721990400,
    "providerId": "com.biofocus.desktop"
  }
]
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `id` | string | Observation UUID |
| `kind` | string | v1 Life Event kind |
| `timestamp` | number | Unix seconds UTC |
| `providerId` | string | Usually `com.biofocus.desktop` for quick-log |

Errors are calm UI-safe strings (unknown kind, storage soft-fail). Never returns raw biometric payloads or absolute filesystem paths.

### `get_git_watched_roots` / `set_git_watched_roots` (P14-E3-T1 / ADR-014)

- **Invoke:** `invoke("get_git_watched_roots")` / `invoke("set_git_watched_roots", { roots })`
- **Purpose:** Menubar editor for personal Git watched folders. Host reads/writes `~/.biofocus/git-watched-roots.toml` (or `$BIOFOCUS_HOME/git-watched-roots.toml`). UI ↛ SQLite; UI never opens the file itself.
- **SoT:** Config **file** only for Settings get/set. Probe may still use `BIOFOCUS_GIT_WATCHED_ROOTS` when the file is **absent** (tests/CI) — that env list is not the Settings editor SoT.
- **Validation:** absolute paths or `~/…`; empty `roots` writes `roots = []` → live probe soft-fails idle (no whole-disk scan).
- **Privacy:** roots may appear on **this** IPC only so the user can edit them. Never copy roots into `git_activity` Observation payloads or default host logs.

`get_git_watched_roots` / `set_git_watched_roots` success:

```json
{
  "roots": ["/Users/you/Developer/AI Project/BioFocus"],
  "source": "file",
  "version": 1
}
```

| Field | Type | Notes |
| :--- | :--- | :--- |
| `roots` | string[] | Absolute directory paths from the config file |
| `source` | string | `file` \| `empty` (Settings path; not env) |
| `version` | number \| null | File schema version when present |

Errors are calm UI-safe strings (relative path rejected, config folder unwritable).
