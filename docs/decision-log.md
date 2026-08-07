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
| ADR-008 | 2026-08-07 | Pattern Discovery v1 = **recompute-on-read** multi-window Features from local Observations (+ optional in-process memo); evolve `knowledge-engine`; **no** Feature/baseline history SQLite table in v1 | North star needs multi-day / baseline Knowledge without a second persistence model or busy background jobs. Observations remain source of truth; Features stay derived (ADR-007). Evaluate-on-read matches existing `KnowledgeEngine::evaluate` + Insights IPC. Local-only; idle-safe. | Persisted Feature/baseline history store for v1; always-on recompute worker / busy-loop; cloud sync of patterns; ML model training; parallel “Correlation Engine” crate |

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
- **No SQLite schema change** for Feature confidence in v1 (in-memory / IPC only). Explanation factor breakdown → P7-E2 (additive optional `Feature.factors`; **no** separate ADR — see note below).

**Rejected alternatives**

1. UI-only opacity/heuristics without a Core `Feature.confidence` field.
2. A second Feature registry or parallel confidence table.
3. Treating Feature confidence as identical to Observation.confidence.

### Note — Explanation factors (P7-E2, no ADR)

Additive optional `Feature.factors: Vec<ExplanationFactor>` `{ id, label, share }` on domain + snapshot IPC. Catalog nodes that emit factors use renormalized shares summing to `1.0` over present components. Empty factors omit the JSON key (`skip_serializing_if`). Calm labels only (inputs/weights — not clinical). **No** SQLite schema; **no** Explainability Engine crate. First emitter: `FocusScore`.

### ADR-008 detail — Pattern Discovery history / recompute (v1)

**Chosen approach:** **recompute-on-read** (evaluate-on-read), not a persisted Feature/baseline history store.

| Layer | Role in Pattern Discovery v1 |
| :--- | :--- |
| `Observation` (SQLite) | Immutable local facts — only durable history |
| `Feature` (in-memory / IPC) | Current window from Feature Worker cache; baseline series **recomputed** from Observations via `feature-engine` when a pattern rule needs multi-window inputs |
| `Insight` (`knowledge-engine`) | Deterministic rules compare current Feature(s) vs recomputed baseline; Evidence refs stay Feature/Signal ids |

**v1 mechanics**

1. Live snapshot path unchanged: `get_feature_snapshot` / current Insights rules read the in-memory Feature Worker cache (no SQLite on that hot path).
2. Pattern / baseline rules (→ **P8-E2**): Core builds a **bounded** multi-window Feature series by loading Observations for a capped lookback (e.g. ≤7 days) and running catalog nodes for discrete windows — then `InsightRule::evaluate` compares current vs baseline.
3. **Idle / privacy:** no always-on recompute worker; no busy-loop. Empty / thin history → omit baseline Insight (`Ok([])`). All compute local-only; no cloud sync of patterns or biometrics.
4. **Optional in-process memo:** process-local cache of the last bounded baseline series (TTL / invalidate on new Observation watermark) is allowed to avoid repeating DAG work within a session — **not** SQLite, not a second registry.
5. **Calm framing:** Insights describe personal observations (“Focus looks higher than your recent afternoon average”), never clinical diagnosis.

**Schema (v1)**

- **No new SQLite tables/columns** for Feature history or baselines.
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): optional daily Feature rollup table if recompute cost becomes a proven dogfood issue — out of Phase 8 v1 scope until then.

**Knowledge consumption sketch (contracts — implement in P8-E2)**

Example rule id: `focus_vs_recent_baseline_v1`

```text
inputs:
  current  = FocusScore from live Feature snapshot (confidence ≥ gate)
  series   = recompute FocusScore for N prior comparable windows
             (same time-of-day bucket or daily afternoon mean; N ≤ 7)
  baseline = mean(series values with confidence ≥ gate)
emit when:
  |current - baseline| ≥ δ (catalog/rule constant, e.g. 10 points)
  and both sides pass confidence gate (ADR-007)
Insight (calm):
  title: "Focus relative to your recent average"
  description: "Focus looks higher / lower than your recent afternoon average."
  category: "focus" | "pattern"
  evidenceList: [ Feature FocusScore (current), … ]
  actionRecommendation: optional gentle, non-clinical
IPC: still `get_insights` → evaluate-on-read; UI ↛ SQLite
```

**Rejected alternatives**

1. Persisted Feature / baseline history SQLite store for Pattern Discovery v1 (conflicts with ADR-007 persistence stance; premature schema).
2. Always-on background recompute worker or polling busy-loop.
3. Cloud sync of patterns / biometrics.
4. ML model training for pattern discovery.
5. Parallel “Correlation Engine” crate / second registry — evolve `knowledge-engine` + existing Feature path instead.