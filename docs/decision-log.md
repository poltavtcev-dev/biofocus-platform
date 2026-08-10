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
| ADR-009 | 2026-08-08 | Recommendations v1 = first-class `Recommendation` + `RecommendationRule` in `knowledge-engine` (evaluate-on-read); keep thin `Insight.actionRecommendation` as optional hint only; **no** Recommendation SQLite store; **no** parallel Coach Engine crate | Vision L4 needs Evidence-backed suggested actions, not only a string on Insight. Evolve existing Knowledge path (ADR-008); local-only; idle-safe; LLM stays L5 interpret-only. | LLM as source of truth for actions; clinical/prescription framing; parallel Coach Engine without Evidence; cloud sync of recommendations; persist Recommendation history in v1; replace Insights with Recommendations |
| ADR-010 | 2026-08-10 | Plugin wave-1 source = **Browser categories** (not IDE/Git in v1); `data_type: "browser_category"` coarse labels only → existing `observations` store; opt-in env default **off**; Capability Plugin Model via `plugin-sdk` + `macos-collector` (or thin adapter); E3 Feature = `DistractionScore` | Vision source priority after Calendar is IDE/Git/**Browser**; catalog already names `DistractionScore` (browser categories). Complements `context_window` / CSR (browser is one app — categories reveal in-browser fragmentation). Privacy-first: no full URLs / titles / keystroke content; personal self-tracking only. | Both IDE+Browser in same wave; always-on capture; cloud history sync; plugin marketplace crate; parallel plugin SQLite registry; ambient music/weather in P10; IDE/Git as wave-1 v1 (deferred) |

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

### ADR-009 detail — Recommendations domain / engine shape (v1)

**Chosen approach:** First-class domain `Recommendation` (Evidence-backed) evaluated by pluggable `RecommendationRule`s hosted in **`knowledge-engine`** (same crate as `InsightRule`). **Evaluate-on-read** only — no Recommendation persistence in v1. Thin optional `Insight.actionRecommendation` remains for Insights UX / reports but is **not** the L4 contract.

| Layer | Role in Recommendations v1 |
| :--- | :--- |
| `Observation` (SQLite) | Immutable local facts — only durable history (unchanged) |
| `Feature` / `Signal` | Derived inputs; confidence gates (ADR-007) still apply |
| `Insight` (`knowledge-engine`) | L3 Knowledge — patterns / conclusions with Evidence; may include optional thin `actionRecommendation` string |
| `Recommendation` (`knowledge-engine` + `bio-spec`) | L4 — calm, optional suggested action with its **own** Evidence (Feature / Signal / Insight ids) |

**Why not “thin text only”:** Vision L4 requires deterministic suggestions **with Evidence**. A single optional string on `Insight` cannot carry independent provenance, emit without a matching Insight title, or be listed/filtered as a first-class surface. Phase 9 elevates L4 without discarding existing thin hints.

**v1 mechanics**

1. **Types (→ P9-E2):** `bio-spec::Recommendation` — at minimum `{ id, title, suggestion, category, evidence_list }`. Calm non-clinical copy in `suggestion` (and title). Extend `EvidenceRef` with an `Insight(InsightId)` variant (or equivalent) so Recommendations can cite Knowledge as Evidence alongside Feature/Signal.
2. **Engine host:** `RecommendationRule` trait + registry inside `knowledge-engine` (mirror `InsightRule`). Host evaluates Insights first (existing path), then Recommendation rules with inputs: current Features, Signals, and the just-evaluated Insights. Empty / no-match → `Ok([])`.
3. **Evaluate-on-read:** No always-on recommendation worker; no busy-loop. Compute on IPC / Core request (same idle posture as Insights + ADR-008). Optional process-local memo of last evaluation is allowed — **not** SQLite.
4. **Idle / privacy:** Local-only; no cloud sync of recommendations or biometrics. Thin / low-confidence inputs → omit Recommendation.
5. **LLM boundary:** L5 (`report-engine` / opt-in LLM) may **rephrase** an already-computed Recommendation for display only — must **not** invent actions, Evidence, or scores. Deterministic Core remains source of truth for L4.
6. **Calm framing:** Optional personal hints (“If it fits your schedule…”), never medical advice, prescriptions, or diagnosis.

**Schema (v1)**

- **No new SQLite tables/columns** for Recommendations or recommendation history.
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): optional user feedback / dismiss history — out of Phase 9 v1 until then.

**IPC / surface (contracts — implement in P9-E3)**

- Prefer a dedicated evaluate-on-read path (e.g. `get_recommendations`) returning `Recommendation[]`, UI ↛ SQLite.
- Evolving Insights DTO alone is insufficient once L4 is first-class; thin `actionRecommendation` may still appear on Insights for backward compatibility.
- Mock/dev fixtures allowed for Dashboard without rich history (same pattern as Insights).

**E2 consumption sketch (implement in P9-E2)**

Example rule id: `focus_dip_pace_hint_v1`

```text
inputs:
  insights = evaluate-on-read Knowledge Insights (existing path)
  pattern  = Insight rule_id / category "pattern" for focus vs recent baseline
             (e.g. focus_vs_recent_baseline_v1) when Focus looks lower than baseline
  features = live FocusScore (confidence ≥ gate, ADR-007)
emit when:
  matching pattern Insight is present
  and FocusScore confidence ≥ gate
  and (optional) |current - baseline| already satisfied by that Insight
Recommendation (calm):
  title: "A gentler pace may help"
  suggestion: "If it fits your schedule, a short pause or slightly slower pace
               may help when focus looks lower than your recent average."
  category: "pace" | "focus"
  evidence_list: [ Feature FocusScore, Insight <pattern insight id> ]
IPC: get_recommendations → evaluate-on-read; UI ↛ SQLite
```

Legacy: existing Insight rules may keep a short `actionRecommendation` string; L4 product surface and tests target `Recommendation`, not that string alone.

**Rejected alternatives**

1. **LLM as source of truth for actions** — breaks determinism / Evidence; LLM stays L5 interpret-only.
2. **Clinical / prescription framing** — out of product Non-Goals; calm optional hints only.
3. **Parallel “Coach Engine” crate** that bypasses Evidence / Knowledge — prefer evolve `knowledge-engine` + `bio-spec` (same stance as ADR-008 vs Correlation Engine).
4. **Cloud sync of recommendations** — local-first; no default remote store.
5. **Persist Recommendation history in v1** — premature schema; Observations remain durable SoT.
6. **Evolve only `Insight.actionRecommendation` (no first-class type)** — insufficient for Evidence-backed L4 listing / gates.
7. **Replace Insights with Recommendations** — L3 Knowledge and L4 actions stay distinct layers.

### ADR-010 detail — Plugin wave-1 source + Observation contract (v1)

**Chosen wave-1 source:** **Browser categories** (personal, opt-in). IDE/Git is deferred to a later sprint / second wave — not Phase 10 v1.

| Layer | Role in Plugin wave-1 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk`) | New `BioFocusPlugin` declares Capability for `browser_category`; `start_stream` / `stop_stream` only |
| Observation (SQLite) | Immutable facts in existing `observations` table — **no** new plugin registry / parallel store (ADR-006 stance) |
| Feature (`feature-engine`) | E3: `DistractionScore` consumes `browser_category` (+ may combine with CSR) — ADR-007 confidence; calm non-clinical |
| LLM | L5 interpret-only — must **not** define plugin payloads, categories, or Features |

**Why Browser (not IDE/Git) for wave-1**

1. **Catalog readiness:** `docs/06-feature-catalog.md` § Planned already lists `DistractionScore` with browser categories as likely inputs — clear E3 path without inventing a new Feature name mid-ADR.
2. **Complements existing dogfood:** `context_window` + `ContextSwitchRate` treat Safari/Chrome/etc. as **one** app. Coarse browser categories add in-browser fragmentation signals CSR cannot see.
3. **Capability Model fit:** New Capability + `data_type` on the same shared Observation channel as macOS context / input / calendar — extend `plugin-sdk` + `macos-collector` (or thin adapter), not a marketplace crate.
4. **IDE/Git deferral rationale:** IDE apps already appear in `context_window` / CSR; Git history needs careful path / remote privacy scoping. Ship Browser first; revisit IDE/Git after DistractionScore dogfood.

**Observation contract sketch (implement in P10-E2)**

| Field | Value |
| :--- | :--- |
| `data_type` | `"browser_category"` |
| `provider_id` | `com.biofocus.macos.browser` (host collector and/or local extension adapter — exact probe in E2) |
| Opt-in | `BIOFOCUS_BROWSER_CATEGORIES=1` (default **off**); host starts plugin only when set |
| Poll / idle | Emit **on category change** (or rare poll ≥5s); **no** busy-loop; `stop_stream` must join background work |
| Confidence | Provider-set ∈ `[0.0, 1.0]` (unknown mapping → lower confidence OK) |

**Privacy-safe payload (v1)** — coarse labels only:

```json
{
  "id": "…",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.browser",
  "data_type": "browser_category",
  "payload": {
    "category": "work",
    "browser_bundle_id": "com.apple.Safari"
  },
  "confidence": 0.9
}
```

| Payload field | Rule |
| :--- | :--- |
| `category` | **Required.** Closed v1 set: `work` \| `communication` \| `entertainment` \| `reference` \| `shopping` \| `unknown` (E2 may refine labels in contracts; keep coarse) |
| `browser_bundle_id` | Optional frontmost browser app id (same privacy bar as `context_window.bundle_id`) |

**Explicitly forbidden in v1 Observations / logs**

- Full URLs, query strings, or hostnames that identify a specific site beyond category mapping internals (mapping tables stay in-process; **not** persisted as Observation fields)
- Page titles, form content, keystroke / content capture, screenshots, clipboard
- Employee-surveillance framing (product is **personal self-tracking** only — vision Non-Goal)
- Always-on capture without opt-in env

**Schema (v1)**

- Prefer existing `observations` store only (ADR-006 stance).
- **No new SQLite tables/columns** for plugins, browser history, or category registries.
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): optional host allowlist config table — out of Phase 10 v1 until then.

**Capability Plugin Model**

```text
BioFocusPlugin
  id: com.biofocus.macos.browser
  capabilities: [{ name: "browser_categories", data_types: ["browser_category"] }]
  start_stream(tx) → emit Observation on change (opt-in)
  stop_stream() → join; freeze emissions
Desktop ingest_host → same bounded Observation channel → spawn_persist_worker → SQLite
UI ↛ SQLite (unchanged)
```

**E2 / E3 sketch**

```text
E2 (P10-E2-T1):
  plugin (macos-collector or thin adapter) + mock probe
  → ObservationSender → persist
  contracts in 07-contracts / 08-plugin-sdk finalized
  idle-safe tests; default off

E3 (P10-E3-T1) — DistractionScore:
  inputs: browser_category Observations (+ optional ContextSwitchRate)
  window: 15m / step 1m (align Focus/CSR)
  formula sketch: higher share of entertainment/shopping / rapid category churn
                  → higher DistractionScore (0–100); work/reference lower
  confidence: ADR-007 (coverage × mean obs confidence); thin → omit or lower
  framing: calm personal context fragmentation — not clinical ADHD / “you are distracted”
  LLM must not invent score or categories
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/16-glossary.md`, `docs/04-storage.md`, `docs/06-feature-catalog.md` (DistractionScore → wave-1).
- Full payload validation + probe tables finalized in **P10-E2** (same pattern as calendar dogfood).

**Rejected alternatives**

1. **Both IDE+Browser in the same wave** — doubles collector + Feature surface; ship one dogfood loop first.
2. **Always-on capture** (no opt-in) — breaks privacy-first / Global DoD plugins rule.
3. **Cloud browsing / history sync** — local-first; no default remote store.
4. **Plugin marketplace / parallel marketplace crate** — extend `plugin-sdk` + `macos-collector` (or thin adapter) unless a future ADR justifies otherwise.
5. **Parallel plugin SQLite registry** without need — Capabilities stay in-process; Observations remain durable SoT.
6. **Ambient music / weather / light in P10** — Phase 12+ per vision ladder.
7. **IDE/Git as wave-1 v1** — deferred (see rationale); may reuse `context_window` more heavily later.
8. **Full URL / title logging “for accuracy”** — rejected; coarse categories only.
9. **NotificationPressure in this ADR** — out of Phase 10 unless a later task folds a thin path; not chosen here.