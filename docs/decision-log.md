# Decision Log (Architecture Decision Records)

| ID | Date | Decision | Reason | Rejected Alternatives |
| :--- | :--- | :--- | :--- | :--- |
| ADR-001 | 2026-07-27 | Modular Monolith in Rust | Низкое потребление ОЗУ (<30MB), безопасность памяти, скорость. | Node.js, Go, Python |
| ADR-002 | 2026-07-27 | SQLite с режимом WAL | Local-first концепция, отсутствие сложных внешних серверов. | PostgreSQL, MongoDB |
| ADR-003 | 2026-07-27 | Tauri v2 для Desktop UI | Компактный размер дистрибутива, минимальная нагрузка на CPU/RAM. | Electron |
| ADR-004 | 2026-07-27 | Лицензия AGPLv3 | Защита открытого кода от создания закрытых коммерческих форков корпорациями. | MIT, Apache 2.0 |
| ADR-005 | 2026-08-05 | Opt-in LAN ingest bind for companion dogfood; default remains loopback (`127.0.0.1`) | Physical iPhone on Wi-Fi cannot reach loopback-only Desktop; LAN must be explicit so local-only stays the safe default. Knobs: `BIOFOCUS_INGEST_LAN=1` → `0.0.0.0`, optional `BIOFOCUS_INGEST_BIND_HOST`. Bearer pairing token still required. | Always-on `0.0.0.0`; mDNS/TLS in Phase 5; cloud relay |
| ADR-006 | 2026-08-05 | Life Events as Observation kinds (`data_type: "life_event"`, `payload.kind`); no parallel Life Events table/store; default local-only | Vision rule 6 + Phase 6 dogfood: Coffee / Walk / Lunch / Workout must feed ActivityBalance / schedule Features without a second persistence model. Reuse existing Observation SQLite store + ingest. v1 kinds: `coffee`, `walk`, `lunch`, `workout`. Payload: required `kind`; optional `note` (string), `duration_secs` (≥ 0). Validation in `bio-spec`; ingest rejects malformed Life Events with `400 invalid_life_event`. | Parallel `life_events` SQLite table; separate event bus; cloud sync of Life Events; one `data_type` per kind without shared schema |
| ADR-007 | 2026-08-06 | Feature-level `confidence` ∈ `[0.0, 1.0]` on domain `Feature` (+ snapshot IPC); computed by catalog nodes; **no** new SQLite schema / second Feature registry | Observation.confidence trusts a single fact; Feature values still looked equally trustworthy on thin windows. Trust layer needs a derived quality score so Insights/UI can down-weight incomplete evidence. Confidence is **data quality**, not a clinical claim. | UI-only heuristics without Core field; parallel Feature confidence store/registry; persisting Feature rows in SQLite for v1 |

### ADR-007 detail — Feature confidence (v1)

**Relationship**

| Layer | Meaning |
| :--- | :--- |
| `Observation.confidence` | Provider trust in one immutable fact |
| `Feature.confidence` | Trust in the **derived** windowed metric |

Feature confidence is **not** a copy of any single Observation confidence. It combines input coverage with the mean confidence of evidence Observations.

**v1 formula**

```text
coverage = present_input_slots / expected_input_slots   # catalog-declared families
mean_obs = mean(Observation.confidence of provenance evidence)
Feature.confidence = clamp(coverage × mean_obs, 0.0, 1.0)
```

- **Expected slots** are Feature-specific (e.g. FocusScore = 3: typing / stability / HRV; StressIndex = 1: HRV; ContextSwitchRate / calendar Features = 1).
- **Missing-input policy:** empty windows that cannot compute a value → **omit** the Feature for that step (unchanged). Partial inputs that still yield a value → **emit with lower confidence** (coverage < 1). Idle / no evidence → empty snapshot (no busy-loop).
- **No SQLite schema change** for Feature confidence in v1 (in-memory / IPC only). Explanation factor breakdown → P7-E2.

**Rejected alternatives**

1. UI-only opacity/heuristics without a Core `Feature.confidence` field.
2. A second Feature registry or parallel confidence table.
3. Treating Feature confidence as identical to Observation.confidence.