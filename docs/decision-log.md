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
| ADR-011 | 2026-08-10 | L5 coaching polish v1 = **named/versioned prompt packs** in `report-engine` (templates over already-computed Features / Insights / Recommendations) + **calm Dashboard provider UX** for opt-in local LLM status/config; interpret-only; in-process packs + existing env/IPC; **no** chat-history SQLite; **no** parallel Coach Engine | Vision L5 is NL explanation only. Phase 4 shipped `build_report` / `interpret_report` / `generate_report` (env-only). Phase 11 dogfood needs selectable packs + calmer provider surface without making LLM a Feature/Recommendation engine. Local-first; idle; explicit user action; no auto-invoke on Dashboard open. | Cloud LLM by default; LLM as SoT for Features/Recommendations; auto-invoke on Dashboard open; parallel Coach Engine that bypasses Evidence; clinical/prescription coaching tone; persist chat history SQLite without need; shipping packs without ADR |
| ADR-012 | 2026-08-10 | Phase 12 v1 **primary** = ambient **Now Playing / music** Observation wave-1 (one source); **secondary** = commercial packaging as signed-build / notarization / update **runbook + process** (no default cloud sync). E2 = plugin; E3 = `AmbientMediaShare` Feature + packaging runbook. Existing `observations` store; AGPLv3 Core stays open | Vision ladder after plugins is ambient (music/weather/light) + commercial packaging ≠ secret math. macOS Now Playing dogfoods without mic / precise geo; weather/light deferred. Packaging stays open Core + distribution process | All three ambient sources in one wave; always-on ambient capture; cloud sync by default; secret/proprietary Feature formulas; ambient Features without Observations; packaging that forces UI→DB or cloud LLM; IDE/Git re-prioritized into P12; PR during freeze |
| ADR-013 | 2026-08-10 | Plugin wave-2 v1 primary = **Git activity aggregates** (not IDE in the same wave); `data_type: "git_activity"` coarse `activity_kind` (+ optional `event_count`) → existing `observations` store; opt-in `BIOFOCUS_GIT_ACTIVITY` default **off**; Capability Plugin Model via `plugin-sdk` + `macos-collector` (or thin adapter); E3 Feature = `GitActivityRate` | ADR-010 deferred IDE/Git after Browser. IDE apps already visible in `context_window` / CSR; privacy-safe IDE session kinds without paths/titles are weak on macOS collector alone. Git ops are invisible to context — additive signal via counts/cadence only (no paths / remotes / diffs). Personal self-tracking; finish plugin ladder before weather/light or App Store product | IDE+Git same wave; IDE as wave-2 when no additive privacy-safe signal; always-on capture; cloud git history sync; plugin marketplace crate; parallel plugin SQLite registry; weather/light or App Store as P13 primary; NotificationPressure as this wave’s Feature; redefining `DistractionScore`; PR during freeze |
| ADR-014 | 2026-08-10 | Git watched-roots allowlist v1 = **local config file** under `~/.biofocus/git-watched-roots.toml` (user-chosen absolute roots only); unlocks live `SystemGitActivityProbe` in Phase 14 E2; ADR-013 Observation payload unchanged (`activity_kind` + optional `event_count`); **no** SQLite allowlist table / **no** migration; opt-in `BIOFOCUS_GIT_ACTIVITY` remains the enable switch; empty/missing file → soft-fail idle | ADR-013 deferred persisted allowlist → production probe soft-fails idle; dogfood needs privacy-scoped roots without widening Observation payloads. Config file is durable for dogfood, editable by hand / future Settings without schema approve. Personal self-tracking only | Always-on whole-disk scan; env-only allowlist as sole store; SQLite allowlist table in v1; workplace / manager dashboards; cloud git history sync; IDE as Phase 14 primary while allowlist unfinished; weather/light or App Store as this phase primary; NotificationPressure without notification Observations; PR during freeze; applying migration without user approve; persisting repo paths/remotes/branch/SHA/message/diff/author into Observations |
| ADR-015 | 2026-08-11 | Phase 15 v1 **primary** = ambient **light** Observation (exactly one ambient source this phase — **not** weather); `data_type: "ambient_light"` coarse `light_kind` (+ optional bounded `level`) → existing `observations` store; opt-in `BIOFOCUS_AMBIENT_LIGHT` default **off**; Capability Plugin Model via `plugin-sdk` + `macos-collector` (or thin adapter); E3 Feature = `AmbientLightShare` | ADR-012 deferred weather/light after Now Playing; PM-GATE-POST-P14 chose weather/light track and locks **ambient light** for Local-First on-device dogfood (no cloud weather API / precise geo). Complements `AmbientMediaShare` without overlapping media inputs. Personal self-tracking only | Weather as P15 primary when cloud/geo implied; IDE as P15 primary; App Store packaging product as P15 primary; NotificationPressure without notification Observations; always-on capture; cloud light telemetry; parallel marketplace crate; parallel SQLite ambient registry; PR during freeze; applying migration without user approve; camera frames / screen contents / precise geo / always-on mic in payload |
| ADR-016 | 2026-08-11 | **Phase 15 execution supersedes ambient-light collector:** companion **HRV + autonomy** — HealthKit `heartRateVariabilitySDNN` → `hrv` Observation (`sdnn_ms`); keep `heart_rate`; event → local queue → Desktop ingest flush; Core accepts **`rmssd_ms` OR `sdnn_ms`** (prefer rmssd); **no** new SQLite; ADR-015 ambient light contract remains but E2/E3 deferred | Phase 5 companion was one-shot; dogfood needs autonomous HR+HRV without button. Apple HK exposes SDNN not RMSSD — Features must accept SDNN as HRV-proxy (non-clinical). User Priority A 2026-08-11 | Ambient light collector as active P15; sleep/steps/SpO2/ECG in same wave; always-on TCP socket; cloud relay; inventing RMSSD from SDNN without documenting proxy; clinical HRV claims; busy-loop HealthKit poll; new Observation SQLite schema; PR during freeze |
| ADR-017 | 2026-08-11 | **Sequencing:** finish **Phase 16 ambient light** first; **Phase 17** = Mi Fitness / HealthKit depth + Dashboard chart ranges (parked intent — payload contracts in follow-up ADR before build) | User chose option B (2026-08-11): do not preempt P16 again; remember wearable-max + long-range chart UI for next phase. Dogfood still needs ambient light shipped; bracelet depth + 1h/8h/12h/1d/1w charts are committed backlog, not abandoned | Preempt P16 again for wearable (option A); Mi Cloud / unofficial API; Feature history SQLite; LLM as source of chart/Features; clinical claims |
| ADR-018 | 2026-08-11 | **Phase 17 contracts:** HealthKit Observation `data_type`s (`step_count`, `active_energy`, `sleep_interval`; soft-optional `oxygen_saturation`) + keep `heart_rate`/`hrv`; Dashboard chart ranges **1h/8h/12h/1d/1w** via **recompute-on-read** IPC (`get_feature_series`); Snapshot = **latest** Features; existing `observations` store; **no** migration v1 | ADR-017 sequenced Phase 17 after P16; dogfood needs locked payloads/IPC before Companion/UI. Mi writes via Apple Health only. ADR-008 stance for long series. Personal self-tracking; L5 interpret-only | Mi Cloud / unofficial API; Feature-history SQLite for charts; UI→SQLite; clinical SpO2/sleep claims; busy-loop HK poll; workout as parallel Observation family (use Life Event); ECG/clinical labs; applying migration without approve; PR during freeze |
| ADR-019 | 2026-08-11 | **Phase 18 contracts:** opt-in notification Observation `data_type: "notification_event"` (coarse `count` + optional closed-set `category` / `interruption_level` / `app_kind` only) → existing `observations` store; **no** migration v1; Capability Plugin via `plugin-sdk` + `macos-collector`; E3 Feature = **`NotificationPressure`** | PM-GATE-POST-P17 chose interruption intensity after wearable charts; catalog named `NotificationPressure` for years without an Observation family. Local-First + privacy bar (no body/title/content). Personal self-tracking only | IDE as Phase 18 primary; weather ambient; App Store product; content capture “for accuracy”; workplace surveillance framing; `CognitiveLoad` as E3; always-on / busy-loop; parallel notification SQLite store; PR during freeze; applying migration without approve |
| ADR-020 | 2026-08-11 | **Phase 19:** unlock live `SystemNotificationEventProbe` via **hybrid** privacy-safe OS mapping (prefer public surfaces; private/undocumented OK only under hard **non-content field allowlist** + soft-fail); ADR-019 `notification_event` payload **unchanged**; **no** migration; opt-in `BIOFOCUS_NOTIFICATION_EVENTS` unchanged; existing `NotificationPressure` (no formula rewrite) | ADR-019 left production probe soft-fail idle until privacy-safe NC mapping; PM-GATE-POST-P18 chose Live NC OS mapping to dogfood shipped Feature. Pattern mirrors ADR-014 after Git soft-fail. Personal self-tracking only | Content capture / Accessibility UI text scrape; workplace monitoring; always-on busy-loop; rewriting NotificationPressure; IDE / weather / App Store / CognitiveLoad as P19 primary; parallel notification SQLite store; PR during freeze; applying migration without approve; widening payload with bundle_id / titles |
| ADR-021 | 2026-08-11 | **Phase 20:** catalog Feature **`CognitiveLoad`** = Feature-level composite of **`MeetingDensity` + `ContextSwitchRate` + `NotificationPressure`**; window **15m / 1m**; output 0–100; **renormalize** present inputs (omit only when none); ADR-007 expected slots = 3; **no** new Observation / **no** migration; calm “combined demand in this window” framing | PM-GATE-POST-P19 chose CognitiveLoad; all three input Features shipped (incl. live NC). Prefer Feature-level composition over Observation mix so catalog math stays stable and privacy bars stay with leaf Features. Personal self-tracking only | Clinical “cognitive overload” / burnout; workplace surveillance scoring; new Observation families; rewriting MeetingDensity / CSR / NotificationPressure; Observation-level mix as v1; IDE / weather / App Store as P20 primary; PR during freeze; migration without approve |
| ADR-022 | 2026-08-11 | **Phase 21:** catalog Feature **`DeepWorkScore`** = Feature-level composite of **`FocusScore` + `ContextSwitchRate`** (idle **dropped** for v1); window **15m / 1m**; output 0–100; **omit** without FocusScore; **renormalize** when CSR absent; ADR-007 expected slots = 2; **no** new Observation / **no** migration; calm “sustained focus in this window” framing | PM-GATE-POST-P20 chose DeepWorkScore after CognitiveLoad; FocusScore + CSR long shipped. Compose high Focus + low CSR — do not invent parallel FocusScore. Personal self-tracking only | Clinical “flow state” / burnout / ADHD; workplace surveillance; new Observation families; rewriting FocusScore / CSR; idle Observation invent; IDE / weather / App Store / AttentionStability / CircadianOffset as P21 primary; PR during freeze; migration without approve |
| ADR-023 | 2026-08-11 | **Phase 22:** catalog Feature **`AttentionStability`** = Feature-level **variance/stability** composite of **`FocusScore` + `ContextSwitchRate`** (distinct from DeepWorkScore intensity); window **15m / 1m**; output 0–100; **omit** without FocusScore; **renormalize** when CSR absent; Focus term from **in-window Focus range** (not Focus level); ADR-007 expected slots = 2; **no** new Observation / **no** migration; calm “focus stability in this window” framing | PM-GATE-POST-P21 chose AttentionStability after DeepWorkScore; same leaves, different question (consistency vs intensity). Personal self-tracking only | Clinical ADHD / “you can’t focus”; workplace surveillance; new Observation families; rewriting FocusScore / CSR / DeepWorkScore; using Focus level as intensity (DeepWorkScore); IDE / weather / App Store / CircadianOffset as P22 primary; PR during freeze; migration without approve |
| ADR-024 | 2026-08-11 | **Phase 23:** **Personal Context Layer** — three pillars: (**1**) reference bands **Variant B** (personal baseline primary; literature secondary, cited calm orienting ranges); (**2**) opt-in **user-declared** health context → prompt packs / report (L5); (**3**) **desk-away** from secondary signals (**no** precise GPS). Prefer local config (`~/.biofocus/…`); **no** migration v1; Features stay provider-agnostic | PM-GATE-POST-P22 chose Personal Context Layer (supersedes CircadianOffset draft). Focus-ladder Features shipped — next gap is interpretation + presence context without medical/GPS product | Precise GPS / continuous geo; clinical diagnosis engine; LLM inventing conditions; cloud health sync by default; workplace presence monitoring; rewriting Focus/Stress from disease tags; CircadianOffset / IDE / weather / App Store as P23 primary; PR during freeze; migration without approve |
| ADR-025 | 2026-08-12 | **Phase 24:** catalog Feature **`CircadianOffset`** = **Observation-level timing** composite of **sleep timing** (`sleep_interval`) vs **work/activity timing** (`keystrokes` / `context_window`, optional `step_count` / `active_energy` / workout `life_event`); Feature cadence **15m / 1m** with **24h lookback**; output **0–100** alignment (not signed hours); **omit** unless both sleep + work/activity timing present; ADR-007 expected slots = 2; **no** new Observation / **no** migration; calm “schedule alignment in this window” framing | PM-GATE-POST-P23 chose CircadianOffset after Personal Context; sleep + activity Observations already shipped (P17+). Timing alignment ≠ SleepDebt magnitude ≠ DeskAwayPresence. Personal self-tracking only | Clinical chronotype / circadian-disorder / “night owl so you fail”; workplace schedule surveillance; new Observation families; rewriting SleepDebt / EnergyScore / ActivityBalance / FocusScore; Feature-level SleepDebt/EnergyScore as timing; signed chronotype hours as primary units; IDE / weather / App Store / TypingRhythm as P24 primary; precise GPS; PR during freeze; migration without approve |

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

### ADR-011 detail — AI coaching polish: prompt packs + provider UX (v1)

**Chosen approach:** Evolve Phase 4 L5 (`report-engine` + opt-in local LLM + Dashboard `generate_report`) with (1) **named/versioned prompt packs** — in-process templates that wrap already-computed Features / Insights / Recommendations into offline `markdown` + `llm_prompt`; (2) **calm Dashboard provider UX** — status/config surface for the existing opt-in local LLM path. LLM remains **interpret-only** (Vision L5): may rephrase deterministic Evidence for display; must **not** invent scores, Features, Insights, Recommendations, or Evidence.

| Layer | Role in Phase 11 v1 |
| :--- | :--- |
| `Feature` / `Insight` / `Recommendation` | Deterministic Core outputs (L2–L4) — **source of truth**; unchanged by packs / LLM |
| Prompt pack (`report-engine`) | Named + versioned template: selects tone/sections/instructions over those inputs → `ReportDocument`-shaped offline output |
| Local LLM (`interpret_report`) | Opt-in rephrase of pack `llm_prompt` only; no Feature math; no new Evidence |
| Provider UX (Dashboard) | Calm status (disabled / ready / error) + pack-aware Report flow behind **explicit** user action |
| Persistence | **No** new SQLite for packs or chat history in v1 — packs in-process; LLM knobs stay env / host config (Phase 4 stance) |

**Why packs (not a single hard-coded prompt forever)**

1. Dogfood needs named variants (default calm coaching, shorter summary, …) without forking `report-engine`.
2. Versioning lets E2 ship `id` + `version` so Dashboard / tests pin a pack without silent prompt drift.
3. Packs still consume **already-computed** Evidence — they do not replace `feature-engine` / `knowledge-engine`.

**Why provider UX (not env archaeology forever)**

1. Phase 4 enablement is env-only (`BIOFOCUS_LOCAL_LLM=…`); power users need a calm in-app read of status without turning LLM on by default.
2. Status must stay honest: disabled → no sockets; ready → host config present; error/timeout → soft-fail without losing markdown (`llmStatus` pattern).
3. Config surface may **reflect** existing env/host knobs; inventing a second cloud-provider marketplace is out of scope.

**v1 mechanics (contracts — implement in P11-E2 / P11-E3)**

1. **Prompt packs (→ P11-E2):** Public API in `report-engine`, e.g. `build_report_with_pack(pack_id, version, features, insights, recommendations) → Result<ReportDocument>` (exact names left to E2). At least one default pack (`biofocus.default` / `1`) remains calm / non-clinical and forbids inventing metrics or actions in the `llm_prompt` instructions.
2. **No Feature math in pack builder:** Packs format and instruct; scalars/objects come from Core inputs only. Empty / partial Evidence → soft empty sections, `Ok`, no panic.
3. **Provider UX (→ P11-E3):** Dashboard shows calm local-LLM status via IPC (UI ↛ SQLite). Report / coaching flow selects a pack (or default) and calls existing `generate_report` / interpret path on **explicit** user action only — never on Dashboard open, soft poll, or background timer.
4. **Idle / local-first:** Default LLM **off**; no auto-send of biometrics or prompts on app start. Prefer `127.0.0.1` Ollama-compatible endpoint (Phase 4). No cloud LLM marketplace in v1.
5. **LLM boundary (reaffirmed):** Must not invent scores, Evidence, Features, Insights, or Recommendations. Deterministic Core remains SoT for L2–L4; L5 is NL explanation only.
6. **Calm framing:** Coaching copy is optional personal interpretation — never medical advice, prescriptions, diagnosis, or employee-surveillance tone.

**Schema (v1)**

- **No new SQLite tables/columns** for prompt packs, pack registries, chat transcripts, or coaching history.
- Prefer **in-process** pack catalog + existing env / IPC (Phase 4 stance).
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): user-editable on-disk pack overrides; optional dismiss/feedback history — out of Phase 11 v1 until then.

**E2 / E3 sketch**

```text
E2 (P11-E2-T1) — report-engine packs:
  in-process registry: { id, version, build(features, insights, recommendations) }
  → offline markdown + llm_prompt (interpret-only instructions)
  default pack calm / non-clinical; unit tests: select pack, empty/partial Evidence
  no network from pack builder; no Desktop UI required; no SQLite

E3 (P11-E3-T1) — Dashboard provider UX:
  IPC status: disabled | ready | error (reflect BIOFOCUS_LOCAL_LLM / host config)
  pack-aware Generate report → existing generate_report / interpret path
  explicit user action only; soft-fail LLM → markdown still returned
  calm copy; no secrets in UI logs; UI ↛ SQLite
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/09-api.md`, `docs/10-security.md`, `docs/16-glossary.md`.
- Full pack API signatures + provider IPC DTO finalized in **P11-E2** / **P11-E3**.

**Rejected alternatives**

1. **Cloud LLM by default** — breaks local-first / privacy default; opt-in local endpoint stays the v1 path.
2. **LLM as source of truth for Features / Recommendations** — breaks determinism / Evidence; L5 interpret-only only.
3. **Auto-invoke on Dashboard open** (or soft poll) — idle / no surprise network; explicit user action only (Phase 4 `generate_report` rule).
4. **Parallel “Coach Engine” crate** that bypasses Evidence / Knowledge — prefer evolve `report-engine` + existing Report IPC (same stance as ADR-009 vs Coach Engine for L4).
5. **Clinical / prescription coaching tone** — out of product Non-Goals; calm optional interpretation only.
6. **Persist chat history in SQLite without need** — premature schema; no coaching transcript store in v1.
7. **Shipping prompt packs / provider UX without ADR** — Phase 11 boundary must be recorded before E2/E3 code.
8. **In-app cloud provider marketplace** — out of Phase 11; ambient / commercial packaging is Phase 12+.

### ADR-012 detail — Phase 12 scope: ambient Now Playing + packaging runbook (v1)

**Chosen Phase 12 v1 primary track:** **Ambient Now Playing / music** — one ambient Observation source (Capability Plugin Model).  
**Chosen Phase 12 v1 secondary track (same phase):** **Commercial packaging** as **signed-build / notarization / update-channel runbook + process** — not App Store productization; **not** default cloud sync; Core math stays AGPLv3 / open.

| Layer | Role in Phase 12 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk`) | New `BioFocusPlugin` declares Capability for Now Playing ambient; `start_stream` / `stop_stream` only |
| Observation (SQLite) | Immutable facts in existing `observations` table — **no** parallel ambient store (ADR-006 / ADR-010 stance) |
| Feature (`feature-engine`) | E3: `AmbientMediaShare` consumes `now_playing` Observations — ADR-007 confidence; calm non-clinical |
| Packaging | Docs/process: signed macOS build + notarization runbook + update-channel stance; optional sync **stance only** (off by default; **not** shipped as product in P12) |
| LLM | L5 interpret-only — must **not** invent ambient Features, media kinds, or packaging policy |

**Why Now Playing / music (not weather or light) for ambient wave-1**

1. **Vision ladder:** After plugins, ambient is music / weather / light — pick **one** for dogfood.
2. **macOS feasibility:** System Now Playing / MediaRemote-style probes can yield coarse **playing vs idle** + media kind without microphone, without song lyrics, and without precise home geolocation.
3. **Complements existing context:** Browser categories + CSR describe *what app / in-browser mix*; Now Playing adds *whether personal media is active* during the same windows — useful for calm pattern dogfood without overlapping DistractionScore inputs.
4. **Weather deferred:** Needs location or region; precise geo dumps conflict with privacy bar; coarse weather without good location UX is thin dogfood.
5. **Light deferred:** Ambient-light sensors are uneven across Mac hardware; weaker v1 probe story than Now Playing.

**Why packaging is secondary runbook (not primary code track)**

1. Vision commercial split is **packaging** (signed builds, updates, optional sync, support) — algorithms stay open-source (ADR-004 AGPLv3).
2. Dogfood value in Sprint 23–24 is higher from a real Observation → Feature loop than from shipping App Store listing mid-freeze.
3. Signed-build / notarization / update-channel **process** can land as docs without a sync product or UI→DB break.
4. Optional user-opt-in sync remains a **stance** in P12: if ever built later, default **off**; no mandatory cloud telemetry in this phase.

**Observation contract sketch (implement in P12-E2)**

| Field | Value |
| :--- | :--- |
| `data_type` | `"now_playing"` |
| `provider_id` | `com.biofocus.macos.now_playing` (host collector / thin adapter — exact probe in E2) |
| Opt-in | `BIOFOCUS_NOW_PLAYING=1` (default **off**); host starts plugin only when set |
| Poll / idle | Emit **on play-state / media-kind change** (or rare poll ≥5s); **no** busy-loop; `stop_stream` must join background work |
| Confidence | Provider-set ∈ `[0.0, 1.0]` (unknown mapping → lower confidence OK) |

**Privacy-safe payload (v1)** — coarse aggregates only:

```json
{
  "id": "…",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.now_playing",
  "data_type": "now_playing",
  "payload": {
    "media_kind": "music",
    "is_playing": true
  },
  "confidence": 0.85
}
```

| Payload field | Rule |
| :--- | :--- |
| `media_kind` | **Required.** Closed v1 set: `music` \| `podcast` \| `other` \| `none` \| `unknown` (E2 may refine labels in contracts; keep coarse) |
| `is_playing` | **Required.** Boolean — whether media is actively playing (vs paused / idle) |

**Explicitly forbidden in v1 Observations / logs**

- Song / episode titles, artists, albums, lyrics, playlist names or IDs, artwork URLs that identify content
- Always-on microphone / audio waveform capture
- Precise home geolocation dumps (weather path deferred partly for this reason)
- Employee-surveillance / workplace ambient monitoring framing — product is **personal self-tracking** only
- Always-on capture without opt-in env

**Commercial packaging boundaries (Phase 12 v1)**

| In Phase 12 v1 | Deferred (later epic / ADR + approve) |
| :--- | :--- |
| Signed macOS `.app` / `.dmg` **runbook** (Developer ID, notarization steps) | Full Mac App Store listing / review pipeline as product |
| Update-channel **stance** documented (prefer local/manual or future opt-in updater — exact tool chosen in E3 runbook) | Mandatory auto-update that phones home without consent |
| Optional sync **stance**: off by default; no sync product shipped in P12 | Cloud sync product, account system, remote Observation store |
| Reaffirm AGPLv3 Core remains open; commercial packaging ≠ closed Feature / metric formulas | Proprietary / secret Core math forks |
| Keep UI ↛ SQLite and LLM interpret-only unchanged | Packaging that forces UI→DB or cloud LLM marketplace |

**Schema (v1)**

- Prefer existing `observations` store only (ADR-006 / ADR-010 stance).
- **No new SQLite tables/columns** for ambient media, sync mirrors, or packaging registries.
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): optional sync store / outbox; ambient allowlist config table — out of Phase 12 v1 until then.

**Capability Plugin Model**

```text
BioFocusPlugin
  id: com.biofocus.macos.now_playing
  capabilities: [{ name: "now_playing", data_types: ["now_playing"] }]
  start_stream(tx) → emit Observation on change (opt-in)
  stop_stream() → join; freeze emissions
Desktop ingest_host → same bounded Observation channel → spawn_persist_worker → SQLite
UI ↛ SQLite (unchanged)
```

**E2 / E3 sketch (names locked)**

```text
E2 (P12-E2-T1) — Now Playing ambient plugin:
  plugin (macos-collector or thin adapter) + mock/scripted probe
  → ObservationSender → persist
  contracts in 07-contracts / 08-plugin-sdk finalized
  idle-safe tests; BIOFOCUS_NOW_PLAYING default off
  no Feature math; no packaging installer binary required

E3 (P12-E3-T1) — AmbientMediaShare + packaging runbook:
  Feature AmbientMediaShare:
    inputs: now_playing Observations (media_kind + is_playing)
    window: 15m / step 1m (align Focus/CSR / DistractionScore)
    formula sketch: share of window with is_playing && media_kind ∈ {music, podcast, other}
                    → 0–100; none/unknown alone → omit or low confidence
    confidence: ADR-007; thin → omit or lower
    framing: calm personal “media present during this window” — not clinical / “you listen too much”
    LLM must not invent score or media kinds
  Packaging companion (same E3):
    docs runbook: signed build + notarization + update-channel stance
    optional sync remains off-by-default stance only (no product)
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/10-security.md`, `docs/12-development.md`, `docs/16-glossary.md`, `docs/06-feature-catalog.md` (`AmbientMediaShare` → P12-E3).
- Full payload validation + probe tables + Feature formula finalized in **P12-E2** / **P12-E3**.

**Rejected alternatives**

1. **Shipping music + weather + light in one wave** — triples collector + privacy surface; one dogfood loop first.
2. **Always-on ambient capture** (no opt-in) — breaks privacy-first / Global DoD ambient rule.
3. **Cloud sync by default** — local-first; optional sync must stay opt-in and off by default; no sync product in P12.
4. **Secret / proprietary Feature formulas** as the commercial edge — AGPLv3 Core stays open; packaging ≠ closed metric math (vision).
5. **Ambient Features without Observation inputs** — catalog rule: Features need real inputs; E3 waits on E2 Observations.
6. **Packaging that forces UI→DB or cloud LLM** — UI ↛ SQLite unchanged; LLM stays L5 interpret-only / no marketplace.
7. **IDE/Git plugin wave re-prioritized into Phase 12** — remains deferred from ADR-010 unless a future ADR re-opens with rationale (not this ADR).
8. **Opening a PR during freeze** (before 2026-09-01) — local branch commits only; cluster PR after freeze.
9. **Weather or light as wave-1 v1** — deferred (feasibility / privacy); may follow after AmbientMediaShare dogfood.
10. **Mic / lyrics / playlist capture “for richer context”** — rejected; coarse media_kind + is_playing only.

### ADR-013 detail — Plugin wave-2 scope: Git activity + Observation contract (v1)

**Chosen wave-2 source:** **Git activity aggregates** (personal, opt-in). IDE is **not** wave-2 v1 (deferred again with rationale below). Exactly **one** primary source this wave.

| Layer | Role in Plugin wave-2 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk`) | New `BioFocusPlugin` declares Capability for `git_activity`; `start_stream` / `stop_stream` only |
| Observation (SQLite) | Immutable facts in existing `observations` table — **no** parallel git / plugin registry store (ADR-006 / ADR-010 / ADR-012 stance) |
| Feature (`feature-engine`) | E3: **`GitActivityRate`** consumes `git_activity` — ADR-007 confidence; calm non-clinical; **does not** redefine Browser `DistractionScore` |
| LLM | L5 interpret-only — must **not** invent payloads, activity kinds, or Feature formulas |

**Why Git (not IDE) for wave-2**

1. **ADR-010 deferral still holds for IDE presence:** IDE apps already appear in `context_window` / CSR. Presence alone is not additive.
2. **Privacy-safe IDE “session kind” is weak dogfood on macOS collector alone:** Coarse editing/debug/idle without file contents usually leans on Accessibility **window titles** (often leak paths) or IDE extensions. Soft-fail → mostly `unknown` — little signal beyond bundle_id.
3. **Git is additive:** Version-control ops are **invisible** to `context_window`. Cadence / coarse event kinds are a new Observation family.
4. **Privacy scoping is nameable:** Persist only closed-set `activity_kind` (+ optional small `event_count` aggregate). **No** repo paths, remotes, branch names, diffs, commit messages, SHAs, or authors.
5. **Capability Model fit:** Same shared Observation channel as Browser / Now Playing — extend `plugin-sdk` + `macos-collector` (or thin adapter), not a marketplace crate.
6. **PM lean applied:** Prefer IDE only if additive privacy-safe signal exists; ADR finds that bar unmet for v1 → choose Git.

**Observation contract sketch (implement in P13-E2)**

| Field | Value |
| :--- | :--- |
| `data_type` | `"git_activity"` |
| `provider_id` | `com.biofocus.macos.git` (host collector and/or thin adapter — exact probe in E2) |
| Opt-in | `BIOFOCUS_GIT_ACTIVITY=1` (default **off**); host starts plugin only when set |
| Poll / idle | Emit **on activity change** (or rare poll ≥5s when probing); **no** busy-loop; `stop_stream` must join background work |
| Confidence | Provider-set ∈ `[0.0, 1.0]` (unknown / soft-fail → lower confidence OK) |

**Privacy-safe payload (v1)** — coarse labels / aggregates only:

```json
{
  "id": "…",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.git",
  "data_type": "git_activity",
  "payload": {
    "activity_kind": "commit",
    "event_count": 1
  },
  "confidence": 0.9
}
```

| Payload field | Rule |
| :--- | :--- |
| `activity_kind` | **Required.** Closed v1 set: `commit` \| `checkout` \| `sync` \| `other` \| `idle` \| `unknown` (E2 may refine labels in contracts; keep coarse). `sync` = fetch/pull/push **events as kinds only** — never remotes/URLs. `idle` = opt-in probe saw no recent activity. |
| `event_count` | Optional positive integer (≥ 1) for batched aggregates since last emit; default treat as `1` when absent. **Not** a path list. |

**Explicitly forbidden in v1 Observations / logs**

- Source file paths, buffer / keystroke content, IDE window titles that embed paths
- Full repo remotes / clone URLs, branch **names**, commit SHAs / messages, diffs / patches, author emails, file-change lists
- Workplace / manager dashboards or employee-surveillance framing (product is **personal self-tracking** only)
- Always-on capture without opt-in env
- Cloud git history sync

**Schema (v1)**

- Prefer existing `observations` store only (ADR-006 / ADR-010 / ADR-012 stance).
- **No new SQLite tables/columns** for plugins, git remotes, or repo registries.
- **Do not apply** any migration for this ADR.
- Deferred allowlist: **resolved by ADR-014** (local config file `~/.biofocus/git-watched-roots.toml`; implement live probe in Phase 14 E2) — Phase 13 E2 soft-fails without it by design.

**Capability Plugin Model**

```text
BioFocusPlugin
  id: com.biofocus.macos.git
  capabilities: [{ name: "git_activity", data_types: ["git_activity"] }]
  start_stream(tx) → emit Observation on change (opt-in)
  stop_stream() → join; freeze emissions
Desktop ingest_host → same bounded Observation channel → spawn_persist_worker → SQLite
UI ↛ SQLite (unchanged)
```

**E2 / E3 sketch (names locked)**

```text
E2 (P13-E2-T1) — Git activity plugin:
  plugin (macos-collector or thin adapter) + mock/scripted probe
  → ObservationSender → persist
  contracts in 07-contracts / 08-plugin-sdk finalized
  idle-safe tests; BIOFOCUS_GIT_ACTIVITY default off
  no Feature math

E3 (P13-E3-T1) — GitActivityRate:
  inputs: git_activity Observations (activity_kind + optional event_count)
  window: 15m / step 1m (align Focus/CSR / DistractionScore)
  formula sketch: rate of non-idle closed-set events in window
                  (sum event_count for commit|checkout|sync|other)
                  → scalar ≥ 0 (events per window or per minute — finalize in E3);
                  idle/unknown-only thin windows → omit or lower confidence
  confidence: ADR-007; thin → omit or lower
  framing: calm personal “version-control cadence in this window”
           — not productivity policing / “you commit too little” / workplace monitoring
  LLM must not invent score or activity kinds
  Distinct from DistractionScore — does **not** merge or redefine Browser math
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/10-security.md`, `docs/12-development.md`, `docs/16-glossary.md`, `docs/06-feature-catalog.md`, `docs/04-storage.md`.
- Full payload validation + probe tables + Feature formula finalized in **P13-E2** / **P13-E3**.

**Rejected alternatives**

1. **IDE + Git in the same wave** — doubles collector + Feature surface; ship one dogfood loop (ADR-010 lesson).
2. **IDE as wave-2 v1 without additive privacy-safe signal** — rejected; presence already in `context_window` / CSR; title-based session kinds fail the privacy bar.
3. **Always-on capture** (no opt-in) — breaks privacy-first / Global DoD plugins rule.
4. **Cloud git history sync** — local-first; no default remote store / account system.
5. **Plugin marketplace / parallel marketplace crate** — extend `plugin-sdk` + `macos-collector` (or thin adapter) unless a future ADR justifies otherwise.
6. **Parallel plugin SQLite registry** without need — Capabilities stay in-process; Observations remain durable SoT.
7. **Weather / light ambient or App Store packaging product as Phase 13 primary** — deferred (PM-GATE-POST-P12 chose plugin wave-2).
8. **NotificationPressure as this wave’s Feature** — remains deferred; not justified as the wave-2 catalog target.
9. **Redefining Browser `DistractionScore` math** for Git — rejected; lock a distinct Feature (`GitActivityRate`).
10. **Opening a PR during freeze** (before 2026-09-01) — local branch commits only; cluster PR after freeze.
11. **Persisting repo paths / remotes / diffs “for richer context”** — rejected; coarse activity_kind + optional event_count only.

### ADR-014 detail — Git watched-roots / path-allowlist + live probe boundaries (v1)

**Chosen v1 storage stance:** **local config file** — `~/.biofocus/git-watched-roots.toml` (create dir if missing). Exactly **one** durable allowlist store for Phase 14 v1.

| Layer | Role in Phase 14 v1 |
| :--- | :--- |
| Allowlist config (file) | User-chosen **absolute directory roots** the live probe may watch — configuration only, not Observation history |
| Capability Plugin (`plugin-sdk` + `macos-collector`) | Existing `com.biofocus.macos.git` / `GitActivityPlugin`; E2 wires live `SystemGitActivityProbe` to read allowlist |
| Observation (SQLite) | Unchanged ADR-013 rows in `observations` (`data_type = 'git_activity'`) — **no** new tables/columns |
| Feature (`feature-engine`) | Existing **`GitActivityRate`** — **no** formula rewrite in Phase 14 unless a later ADR finds a justified confidence tweak (none for v1) |
| LLM | L5 interpret-only — must **not** invent allowlist paths, activity kinds, or Feature scores |

**Why config file (not env-only, not SQLite)**

1. **Closes ADR-013 soft-fail gap:** Production `SystemGitActivityProbe` intentionally emits nothing without a watched-roots allowlist. Dogfood needs real VCS cadence without whole-disk scanning.
2. **Durable enough for dogfood:** Survives shell sessions / app restarts — better than an env list as the sole store.
3. **No schema approve / migration:** Prefer non-SQLite for v1 (PM lean). Settings / IPC in optional E3 can read/write the same file later without a table.
4. **Privacy boundary stays nameable:** Roots live only in local config; Observation payloads stay coarse (`activity_kind` + optional `event_count`).
5. **Capability Model fit:** Extend existing git plugin + host — no parallel marketplace crate, no second Observation bus.
6. **Personal self-tracking only:** User explicitly lists personal project roots — not workplace / fleet monitoring.

**Config sketch (implement in P14-E2)**

```toml
# ~/.biofocus/git-watched-roots.toml
# Absolute directory roots only. Empty / missing file → probe soft-fails idle (no emit).
version = 1
roots = [
  "/Users/me/Developer/AI Project/BioFocus",
]
```

| Rule | Detail |
| :--- | :--- |
| Path | `~/.biofocus/git-watched-roots.toml` (expand home at runtime) |
| Contents | `version` (u32, start at `1`) + `roots` (list of absolute directory paths) |
| Semantics | Probe may only consider git activity under these roots (or nested repos discovered **under** a listed root). Outside roots → ignore |
| Missing / empty / unreadable | Soft-fail idle — **no** Observation emit (same safe default as ADR-013 without allowlist) |
| Enable switch | Unchanged: host starts plugin only when `BIOFOCUS_GIT_ACTIVITY=1` (default **off**) |
| Not in this file | Remotes, branch names, SHAs, messages, diffs, authors, per-repo metadata |

**Optional E2 dogfood helper (non-durable):** a one-shot env override (e.g. `BIOFOCUS_GIT_WATCHED_ROOTS` colon/comma-separated) **may** merge or substitute for tests/CI only if implemented — it is **not** the v1 durable store. If both exist, E2 must document precedence; prefer file as SoT when present.

**Privacy contract**

| May store / appear | Must **never** |
| :--- | :--- |
| User-chosen root paths in **local config file only** | Repo paths / remotes / clone URLs / branch names / SHAs / commit messages / diffs / patches / authors / file-change lists inside `git_activity` Observation **payloads** |
| Observation `id`, `timestamp`, `provider_id`, `data_type`, coarse `activity_kind`, optional `event_count`, `confidence` | Whole-disk / home-directory scans without an explicit allowlist |
| Logs: Observation `id`, emit/drop **counts**, allowlist load **ok/empty/error** (no path dump by default) | Logging full allowlist roots or discovered repo paths at info/default levels (prefer debug-gated or omit) |

Pipeline normalize (already shipped) continues to **strip** forbidden keys if a buggy probe ever emits them.

**Schema (v1)**

- **No SQLite allowlist / watched-roots table.**
- **No migration** under ADR-014 — **do not apply** any SQL.
- Sketch only if a future ADR revisits CRUD+Settings: optional `git_watched_roots(id, path, created_at)` — **not** proposed for Phase 14 v1; would require **user approve** before apply.
- Observations remain the only durable git **facts** store.

**Capability / live probe sketch**

```text
BIOFOCUS_GIT_ACTIVITY=1
  → Desktop ingest_host starts GitActivityPlugin
  → SystemGitActivityProbe loads ~/.biofocus/git-watched-roots.toml
  → if roots empty/missing → soft-fail idle (no emit)
  → else watch / poll under allowlisted roots only (≥5s or on change; no busy-loop)
  → emit ADR-013 Observations → bounded channel → persist
  → existing GitActivityRate consumes rows (no Feature rewrite)
UI ↛ SQLite (unchanged); optional E3 Settings edits config file via IPC only
```

**E2 / E3 sketch (names locked)**

```text
E2 (P14-E2-T1) — Allowlist + live probe:
  load/validate git-watched-roots.toml
  live SystemGitActivityProbe under allowlisted roots
  → existing Observation channel → persist
  keep ScriptedGitActivityProbe for tests
  no Feature formula rewrite; no SQLite migration

E3 (P14-E3-T1) — optional companion:
  calm Settings / IPC to edit allowlist file, and/or dogfood gate notes
  still no Observation payload widening; still no Feature math rewrite
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/10-security.md`, `docs/12-development.md`, `docs/04-storage.md`, `docs/16-glossary.md`.
- Live probe behavior + file parse/validation finalized in **P14-E2**. Optional Settings/IPC / dogfood gate in **P14-E3**.

**Rejected alternatives**

1. **Always-on whole-disk / whole-home scan** — privacy bar; Personal self-tracking requires explicit roots.
2. **Env-only allowlist as the sole durable store** — fragile for multi-root dogfood across restarts; rejected as *only* store (optional CI override OK).
3. **SQLite allowlist table in Phase 14 v1** — unnecessary for dogfood; triggers schema approve; defer until proven CRUD need.
4. **Workplace / manager dashboards** / employee git surveillance framing — product is personal self-tracking only.
5. **Cloud git history sync** — local-first; no default remote store.
6. **IDE wave as Phase 14 primary while allowlist unfinished** — PM-GATE-POST-P13 chose Git allowlist first; unlock shipped Feature before new collectors.
7. **Weather / light ambient or App Store packaging product as this phase primary** — deferred (same gate).
8. **NotificationPressure without notification Observations** — remains deferred; not this phase.
9. **Opening a PR during freeze** (before 2026-09-01) — local branch commits only; cluster PR after freeze.
10. **Applying a migration without user approve** — forbidden; this ADR chooses no migration.
11. **Widening `git_activity` payloads with paths/remotes/diffs “for richer Features”** — rejected; ADR-013 contract stands.

### ADR-015 detail — Phase 15 scope: ambient light Observation contract (v1)

> **Supersession (2026-08-11):** Phase **15 execution** moved to **ADR-016** (companion HRV + autonomy). ADR-015 contract remains valid.  
> **Resume (2026-08-11 / PM-GATE-POST-P15):** Phase **16** resumes ambient light collector + `AmbientLightShare` under ADR-015 SoT (no contract change). Implement in **P16-E1** / Feature in **P16-E2**.

**Chosen Phase 15 v1 primary track (original):** **Ambient light** — exactly **one** ambient Observation source this phase (Capability Plugin Model).  
**Not in this wave:** weather ambient (deferred again — cloud / geo implications), IDE plugin, App Store packaging product, NotificationPressure.

| Layer | Role in Phase 15 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk`) | New `BioFocusPlugin` declares Capability for ambient light; `start_stream` / `stop_stream` only |
| Observation (SQLite) | Immutable facts in existing `observations` table — **no** parallel ambient-light / weather store (ADR-006 / ADR-010 / ADR-012 stance) |
| Feature (`feature-engine`) | E3: **`AmbientLightShare`** consumes `ambient_light` — ADR-007 confidence; calm non-clinical; **distinct from** `AmbientMediaShare` / `GitActivityRate` |
| LLM | L5 interpret-only — must **not** invent light kinds, levels, or Feature formulas |

**Why ambient light (not weather) for Phase 15**

1. **ADR-012 deferral:** Wave-1 ambient shipped Now Playing; weather / light were deferred. PM-GATE-POST-P14 reopened the weather/light track and locks **light** as v1 primary within that track.
2. **Local-First:** Prefer **on-device** ambient-light / display-brightness style probes over a **network weather API** that implies region or precise geo.
3. **Privacy bar:** Coarse light bands dogfood without camera frames, screen pixel dumps, mic, or home geolocation.
4. **Complements existing ambient:** `AmbientMediaShare` describes *personal media presence*; ambient light adds *environment brightness context* in the same windows — useful for calm pattern dogfood without overlapping media inputs.
5. **Weather deferred (same phase):** Cloud weather + coarse region UX is a later ambient slice; not parallel with light in P15.

**Observation contract sketch (implement in P15-E2)**

| Field | Value |
| :--- | :--- |
| `data_type` | `"ambient_light"` |
| `provider_id` | `com.biofocus.macos.ambient_light` (host collector / thin adapter — exact probe in E2) |
| Opt-in | `BIOFOCUS_AMBIENT_LIGHT=1` (default **off**); host starts plugin only when set |
| Poll / idle | Emit **on light-band change** (or rare poll ≥5s); **no** busy-loop; `stop_stream` must join background work |
| Confidence | Provider-set ∈ `[0.0, 1.0]` (unknown / unavailable mapping → lower confidence or soft-fail idle OK) |

**Privacy-safe payload (v1)** — coarse aggregates only:

```json
{
  "id": "…",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.ambient_light",
  "data_type": "ambient_light",
  "payload": {
    "light_kind": "dim",
    "level": 25
  },
  "confidence": 0.8
}
```

| Payload field | Rule |
| :--- | :--- |
| `light_kind` | **Required.** Closed v1 set: `dark` \| `dim` \| `moderate` \| `bright` \| `unknown` (E2 may refine labels in contracts; keep coarse closed-set) |
| `level` | **Optional.** Bounded integer **0–100** relative brightness band (not raw identifying sensor dumps; not continuous camera lux streams) |

**Explicitly forbidden in v1 Observations / logs**

- Camera frames, images, video, or scene-understanding embeddings
- Screen contents / screenshots / pixel buffers
- Precise geolocation / home address / region dumps (weather path stays deferred partly for this reason)
- Always-on microphone / audio waveform capture
- Cloud light / weather telemetry phones-home
- Employee-surveillance / workplace ambient monitoring framing — product is **personal self-tracking** only
- Always-on capture without opt-in env

**Schema (v1)**

- Prefer existing `observations` store only (ADR-006 / ADR-010 / ADR-012 stance).
- **No new SQLite tables/columns** for ambient light, weather mirrors, or ambient registries.
- **Do not apply** any migration for this ADR.
- Deferred (future ADR + user approve only): weather Observation family; ambient allowlist / calibration table — out of Phase 15 v1 until then.

**Capability Plugin Model**

```text
BioFocusPlugin
  id: com.biofocus.macos.ambient_light
  capabilities: [{ name: "ambient_light", data_types: ["ambient_light"] }]
  start_stream(tx) → emit Observation on change (opt-in)
  stop_stream() → join; freeze emissions
Desktop ingest_host → same bounded Observation channel → spawn_persist_worker → SQLite
UI ↛ SQLite (unchanged)
```

**E2 / E3 sketch (names locked)**

```text
E2 (P15-E2-T1) — Ambient light plugin:
  plugin (macos-collector or thin adapter) + soft-fail OS probe + scripted probe for tests
  → ObservationSender → persist
  contracts in 07-contracts / 08-plugin-sdk finalized
  idle-safe tests; BIOFOCUS_AMBIENT_LIGHT default off
  soft-fail idle when mapping unavailable (no busy-loop)
  no Feature math

E3 (P15-E3-T1) — AmbientLightShare:
  Feature AmbientLightShare:
    inputs: ambient_light Observations (light_kind + optional level)
    window: 15m / step 1m (align Focus/CSR / AmbientMediaShare / GitActivityRate)
    formula sketch: share of window samples with light_kind ∈ {dark, dim, moderate, bright}
                    (or calm band-share — exact formula in E3); unknown alone → omit or low confidence
    confidence: ADR-007; thin → omit or lower
    framing: calm personal “light context during this window” — not clinical / “bad lighting harms you”
    distinct from AmbientMediaShare (media) and GitActivityRate (VCS cadence)
    LLM must not invent score or light kinds
```

**Docs policy for this ADR**

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/10-security.md`, `docs/12-development.md`, `docs/16-glossary.md`, `docs/06-feature-catalog.md` (`AmbientLightShare` → P15-E3), `docs/04-storage.md`.
- Full payload validation + probe tables + Feature formula finalized in **P15-E2** / **P15-E3**.

**Rejected alternatives**

1. **Weather as Phase 15 primary when cloud/geo implied** — Local-First prefers on-device light; weather deferred to a later ambient slice.
2. **IDE as Phase 15 primary** — still no additive privacy-safe signal beyond `context_window` without a new contract (PM-GATE-POST-P14).
3. **App Store packaging product as Phase 15 primary** — commercial track beyond P12 runbook; not an Observation unlock.
4. **NotificationPressure without notification Observations** — remains deferred; Features need real inputs.
5. **Always-on capture** (no opt-in) — breaks privacy-first / Global DoD ambient rule.
6. **Cloud light telemetry** — local-first; no phones-home ambient sensor stream.
7. **Parallel marketplace crate** — Capability Plugin Model via existing `plugin-sdk` + `macos-collector` / host.
8. **Parallel SQLite ambient registry without need** — existing `observations` store only.
9. **Opening a PR during freeze** (before 2026-09-01) — local branch commits only; cluster PR after freeze.
10. **Applying a migration without user approve** — forbidden; this ADR chooses no migration.
11. **Camera / screen / precise geo / always-on mic “for richer light context”** — rejected; coarse `light_kind` + optional bounded `level` only.

### ADR-016 detail — Phase 15 execution: companion HRV + autonomy (v1)

**Chosen Phase 15 execution track (2026-08-11):** **iOS companion HRV + autonomous sync** (supersedes ambient-light collector for active Sprint 29–30).

| Layer | Role |
| :--- | :--- |
| iOS Companion | HealthKit HR + HRV (SDNN) → local durable queue → `POST /v1/ingest` when Desktop reachable |
| Observation | Existing `heart_rate` + `hrv` contracts — **no** new `data_type` / **no** SQLite migration |
| Pipeline | `normalize_hrv` accepts **`rmssd_ms` OR `sdnn_ms`** (keep both when present) |
| Features | Prefer mean `rmssd_ms`, else mean `sdnn_ms` as HRV-proxy ms (Focus / Recovery); Stress already accepts SDNN |
| LLM | L5 interpret-only — must **not** invent HRV values or clinical claims |

**Why companion autonomy (not ambient light collector) now**

1. **Dogfood gap:** Phase 5 companion is one-shot button; wearable north-star needs continuous facts without manual taps.
2. **Feature unlock:** `StressIndex` / `RecoveryScore` / Focus HRV slot need `hrv` Observations — currently almost never present.
3. **Apple reality:** HealthKit exposes `heartRateVariabilitySDNN` (ms). Catalog historically required `rmssd_ms`. v1 treats SDNN as **non-clinical HRV-proxy** with the same numeric maps when RMSSD absent — documented, not “fake RMSSD”.
4. **Local-First:** LAN ingest + Bearer; queue on device; no cloud relay / always-on TCP.
5. **ADR-015 parked:** Ambient light contract stays; collector deferred.

**Autonomy mechanics (v1)**

```text
HealthKit sample / HKObserver wake
  → encode Observation(s) (heart_rate and/or hrv)
  → durable local queue (dedupe by sample identity)
  → flush POST /v1/ingest when base URL reachable
  → on failure keep queued (no busy-loop retry spin)
```

- Auto-sync **opt-in** in Companion UI (default on after user enables once is OK; document).
- Manual send remains for dogfood.
- Idle-safe: event / background delivery only — **no** tight HealthKit poll loop.

**Schema**

- **No** new SQLite tables/columns.
- Existing `observations` store only.

**Rejected alternatives**

1. **Keep ambient light collector as active Phase 15** while companion stays one-shot — rejected by user Priority A.
2. **Sleep / steps / SpO2 / ECG in same wave** — new contracts / clinical risk; deferred.
3. **Always-on TCP / cloud relay** — privacy / battery; queue+flush only.
4. **Silently writing SDNN into `rmssd_ms`** without ADR — pollutes semantics; prefer explicit `sdnn_ms` + Feature prefer path.
5. **Clinical HRV / recovery diagnosis copy** — Non-Goals.
6. **Busy-loop polling HealthKit** — Global DoD idle footprint.
7. **New Observation SQLite schema / migration** — unnecessary; `hrv` already contracted.
8. **Opening a PR during freeze** — local branch only until 2026-09-01.

### ADR-017 detail — After P16: wearable depth + chart ranges (sequencing)

**Sequencing (locked 2026-08-11):** Complete **Phase 16** (ADR-015 ambient light plugin → `AmbientLightShare`). Do **not** reopen Priority-A preemption of ambient light.

**Status (2026-08-11):** Phase 16 **Done**. Phase 17 **opened**. Payload / chart-range **contracts locked in ADR-018** (this ADR remains sequencing SoT only).

**Phase 17 intent** (contracts → **ADR-018**; implement E2/E3):

1. **Wearable depth (Mi Fitness via HealthKit / Companion)** — maximize Observations Mi actually writes into Apple Health. Soft-optional HRV SDNN. Same autonomy path: observer → queue → Desktop ingest. **No** Mi Cloud / unofficial API.
2. **Dashboard chart range UI** — picker **1h / 8h / 12h / 1d / 1w**; longer ranges = **recompute-on-read** (ADR-008); Snapshot list = **latest**. UI ↛ SQLite.
3. **Analysis ladder** — Features / Insights / Recommendations **without LLM** first; L5 report/LLM remains interpret-only.

**Rejected now**

1. Option A — start wearable Phase before finishing P16.
2. Persisting Feature time-series SQLite for chart history in v1.
3. LLM inventing bracelet metrics or chart values.

### ADR-018 detail — Phase 17 contracts: HealthKit depth + chart ranges (v1)

> **Relationship:** ADR-017 = sequencing unlock. **ADR-018** = buildable Observation + IPC lock for Phase 17. Implement Companion emits in **P17-E2**; chart ranges + Features in **P17-E3**.

| Layer | Role in Phase 17 v1 |
| :--- | :--- |
| iOS Companion | HealthKit samples Mi Fitness writes → local queue → `POST /v1/ingest` (ADR-016 autonomy path) |
| Observation | Existing `heart_rate` + soft-optional `hrv`; **new** `step_count` / `active_energy` / `sleep_interval`; soft-optional `oxygen_saturation` when present |
| Storage | Existing `observations` table only — **no** SQLite migration in v1 |
| Charts / IPC | Range picker → `get_feature_series` recompute-on-read; `get_feature_snapshot` stays **latest**-oriented for lists |
| Features | Deterministic catalog from new + existing Observations (E3); no LLM required |
| LLM | L5 interpret-only — must **not** invent bracelet values, SpO2 clinical meaning, or chart points |

#### A. Wearable Observation contracts (locked names)

**Keep (unchanged):**

| `data_type` | Notes |
| :--- | :--- |
| `heart_rate` | Required `bpm`; existing companion path |
| `hrv` | Soft-optional; `rmssd_ms` **or** `sdnn_ms` (ADR-016); Mi often omits |

**New required family (E2 must emit when HK samples exist):**

##### `step_count`

| Field | Rule |
| :--- | :--- |
| `data_type` | always `"step_count"` |
| `provider_id` | e.g. `com.biofocus.applehealth` (Companion) |
| `payload.count` | **Required.** Non-negative integer step count for the sample / interval |
| `payload.window_secs` | **Optional.** Aggregation window length in seconds when the sample is an interval sum (≥ 1) |

```json
{
  "provider_id": "com.biofocus.applehealth",
  "data_type": "step_count",
  "payload": { "count": 2400, "window_secs": 3600 },
  "confidence": 0.9
}
```

##### `active_energy`

| Field | Rule |
| :--- | :--- |
| `data_type` | always `"active_energy"` |
| `payload.kcal` | **Required.** Non-negative active energy in **kilocalories** (canonicalize aliases in E2 normalize) |

```json
{
  "provider_id": "com.biofocus.applehealth",
  "data_type": "active_energy",
  "payload": { "kcal": 185.5 },
  "confidence": 0.9
}
```

##### `sleep_interval`

| Field | Rule |
| :--- | :--- |
| `data_type` | always `"sleep_interval"` |
| `payload.start` | **Required.** Unix seconds UTC — interval start |
| `payload.end` | **Required.** Unix seconds UTC — interval end (`end` ≥ `start`) |
| `payload.stage` | **Optional.** Closed set: `asleep` \| `in_bed` \| `awake` \| `unknown` |

```json
{
  "provider_id": "com.biofocus.applehealth",
  "data_type": "sleep_interval",
  "payload": {
    "start": 1721952000,
    "end": 1721977200,
    "stage": "asleep"
  },
  "confidence": 0.85
}
```

**Soft-optional (emit only when HealthKit has samples; never invent):**

##### `oxygen_saturation`

| Field | Rule |
| :--- | :--- |
| `data_type` | always `"oxygen_saturation"` |
| `payload.spo2_percent` | **Required when emitted.** Integer/float **0–100** (percent SpO2). Calm non-clinical framing only |

**Explicitly deferred / rejected as Phase 17 Observation families**

- **Workout session Observation** — use existing Life Event `workout` + `step_count` / `active_energy`; avoid a parallel workout store.
- ECG / clinical labs / blood pressure as v1 required types.
- Mi Cloud / unofficial Xiaomi API payloads.
- Inventing SpO2 or sleep stages when HK has none.

**Privacy / idle**

- Personal self-tracking only — not employer wearable monitoring.
- HealthKit event / observer → queue → flush; **no** busy-loop HK poll.
- No raw HK sample dumps in default logs; prefer Observation `id` / `data_type` counts.
- No clinical diagnosis copy (“hypoxia”, “sleep apnea”, “you are unhealthy”).

**Schema**

- Prefer existing `observations` store only.
- **No new SQLite tables/columns** for wearable depth or chart history.
- **Do not apply** any migration for this ADR.
- Future Feature-history rollup or wearable registry → **new ADR + user approve**.

#### B. Dashboard chart ranges (locked IPC stance)

| Item | Value |
| :--- | :--- |
| Ranges | Closed set: `1h` \| `8h` \| `12h` \| `1d` \| `1w` |
| Analysis path | **Recompute-on-read** Feature series from local Observations (ADR-008); optional in-process memo OK |
| Snapshot list | Show **latest** Feature per `featureId` (not every window end) |
| Chart | Holds the **series** for the selected range |
| UI | IPC only — **UI ↛ SQLite** |
| LLM | Must not invent series points |

**Default recompute step (v1 sketch — E3 may tune within same ranges):**

| Range | Approx span | Default `stepSecs` |
| :--- | :--- | :--- |
| `1h` | 3600 | 60 (1m — align catalog `STEP_SECS`) |
| `8h` | 28800 | 300 (5m) |
| `12h` | 43200 | 300 (5m) |
| `1d` | 86400 | 900 (15m) |
| `1w` | 604800 | 3600 (1h) |

**IPC sketches (implement P17-E3)**

1. Keep `get_feature_snapshot` as **latest-oriented** cache read for lists / Menubar-adjacent surfaces (host may collapse to latest-per-id in E3).
2. Add `get_feature_series`:

```text
invoke("get_feature_series", { range: "1h"|"8h"|"12h"|"1d"|"1w", featureIds?: string[] })
→ {
    range: string,
    stepSecs: number,
    window: { start: number, end: number },
    features: Feature[]   // multi-window series for chart; same Feature wire shape
  }
```

- Empty / thin history → `{ features: [] }` (calm).
- Never returns raw Observation biometric payloads or filesystem paths.
- Host loads Observations for the span, runs `FeatureEngine` with the chosen step, returns series — **no** Feature-history SQLite.

#### C. Epic split (names locked)

```text
P17-E1 — Contracts (this ADR + docs) ← Done 2026-08-11 (QA Pass)
P17-E2 — Companion HealthKit expand (emit locked data_types via queue→ingest) ← Done 2026-08-11 (QA Pass)
P17-E3 — Dashboard ranges IPC/UI + catalog Features from new Observations ← Done 2026-08-11 (QA Pass)
```
**Phase 17 closed.** Next slice → **PM-GATE-POST-P17**.


#### D. Analysis ladder

1. Deterministic Features / Insights / Recommendations from Observations (no LLM required).
2. Optional L5 report / local LLM = interpret-only over Evidence.

#### Rejected alternatives

1. **Mi Cloud / unofficial bracelet API** — privacy + ToS; HealthKit only.
2. **Feature history SQLite for chart ranges in v1** — ADR-008 / ADR-017 reject; recompute-on-read.
3. **UI reading SQLite for charts** — Core / IPC only.
4. **Clinical SpO2 / sleep / recovery claims** — Non-Goals.
5. **Busy-loop HealthKit polling** — Global DoD idle footprint.
6. **Required workout Observation family** — Life Event + steps/energy cover dogfood.
7. **ECG / BP as Phase 17 required types** — clinical risk; deferred.
8. **Opening a PR during freeze** — local branch only until 2026-09-01.
9. **Applying a migration without user approve** — this ADR chooses no migration.
10. **LLM inventing bracelet metrics or chart values** — L5 interpret-only.

### ADR-019 detail — Phase 18 contracts: notification Observation + NotificationPressure (v1)

> **Relationship:** **PM-GATE-POST-P17** chose NotificationPressure over IDE / weather / App Store. **ADR-019** = buildable Observation + Feature scope lock for Phase 18. Implement collector in **P18-E2**; Feature math in **P18-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 18 (E1 this ADR · E2 collector · E3 Feature).

| Layer | Role in Phase 18 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk`) | New `BioFocusPlugin` declares Capability for `notification_event`; `start_stream` / `stop_stream` only |
| Observation (SQLite) | Immutable facts in existing `observations` table — **no** parallel notification store / migration (ADR-006 stance) |
| Feature (`feature-engine`) | E3: `NotificationPressure` from `notification_event` — ADR-007 confidence; calm non-clinical |
| LLM | L5 interpret-only — must **not** invent notification payloads, titles/bodies, or Features |

#### Why this source now

1. **Catalog readiness:** `NotificationPressure` has been named in the backlog for interruption intensity; every prior gate deferred it for lack of a notification Observation family.
2. **Local-First fit:** macOS notification delivery metadata can be coarsened without cloud weather APIs or workplace IDE telemetry.
3. **Privacy bar is enforceable:** counts / cadence / closed-set labels only — content capture is an explicit Non-Goal.
4. **Deferred tracks stay deferred:** IDE still lacks an additive privacy-safe signal beyond `context_window`; weather implies cloud/geo; App Store is commercial packaging, not Core Features.

#### Locked Observation family

**Chosen `data_type` (closed set for Phase 18 v1):** **`notification_event`**

> Not `notification_burst` as a separate family — coalesced arrivals in one sample use `payload.count` ≥ 1. One Observation family keeps the catalog / ingest surface small.

| Field | Value |
| :--- | :--- |
| `data_type` | always `"notification_event"` |
| `provider_id` | `com.biofocus.macos.notifications` (collector; exact probe in E2) |
| Opt-in | `BIOFOCUS_NOTIFICATION_EVENTS=1` (default **off**); Desktop `ingest_host` starts plugin only when set |
| Emit cadence | On notification delivery / coalesced change, or rare poll ≥5s; **no** busy-loop; `stop_stream` must join |
| Soft-fail | OS APIs unavailable / permission denied → idle (no emit); injectable scripted probe for tests |
| Confidence | Provider-set ∈ `[0.0, 1.0]` (unknown mapping → lower confidence OK) |

**Privacy-safe payload (v1)** — coarse counts / labels only:

```json
{
  "id": "…",
  "timestamp": 1721990400,
  "provider_id": "com.biofocus.macos.notifications",
  "data_type": "notification_event",
  "payload": {
    "count": 1,
    "category": "communication",
    "interruption_level": "active",
    "app_kind": "messaging"
  },
  "confidence": 0.85
}
```

| Payload field | Rule |
| :--- | :--- |
| `count` | **Required.** Integer ≥ 1 — notifications represented by this sample (usually `1`; >1 when OS coalesces) |
| `category` | **Optional.** Closed set: `communication` \| `calendar` \| `system` \| `media` \| `social` \| `other` \| `unknown` |
| `interruption_level` | **Optional.** Closed set: `passive` \| `active` \| `time_sensitive` \| `critical` \| `unknown` (maps to OS interruption bands when available — not a clinical urgency claim) |
| `app_kind` | **Optional.** Closed set: `messaging` \| `mail` \| `calendar` \| `social` \| `system` \| `other` \| `unknown` |

**Explicitly forbidden in v1 Observations / default logs**

- Notification **body**, **title**, **subtitle**, message text, thread IDs, deep-link URLs, or userInfo dumps
- Screenshots, attachment bytes, contact names, email subjects, chat previews
- Free-form app display names or always-on content dumps “for accuracy”
- Workplace / employer surveillance framing (product is **personal self-tracking** only)
- Always-on capture without the opt-in env

**Schema (v1)**

- Prefer existing `observations` store only.
- **No new SQLite tables/columns** for notifications, NC registries, or Feature history.
- **Do not apply** any migration for this ADR.
- Future allowlists / richer app taxonomy → **new ADR + user approve**.

#### Capability Plugin Model

```text
BioFocusPlugin
  id: com.biofocus.macos.notifications
  capabilities: [{ name: "notification_events", data_types: ["notification_event"] }]
  start_stream(tx) → emit Observation on delivery / rare poll (opt-in)
  stop_stream() → join; freeze emissions
Desktop ingest_host → same bounded Observation channel → spawn_persist_worker → SQLite
UI ↛ SQLite (unchanged)
```

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 18)

```text
E1 (P18-E1-T1) — THIS ADR ← Done 2026-08-11 (QA Pass)
  Lock data_type + privacy bar + NotificationPressure scope in docs

E2 (P18-E2-T1) ← Done 2026-08-11 (QA Pass)
  macos-collector plugin + mock/scripted probe
  bio-spec validate_notification_event_payload (+ ingest reject code)
  pipeline normalize strips any accidental content keys
  Host wire: BIOFOCUS_NOTIFICATION_EVENTS=1 only
  System probe soft-fails idle until privacy-safe OS mapping

E3 (P18-E3-T1) ← Done 2026-08-11 (QA Pass)
  Catalog Feature NotificationPressure — sum count → 0–100 (saturation 20/15m)
  Window / step aligned with Focus catalog (15m / 1m default; series may coarsen)
  Inputs: notification_event in window
  Omit when empty / no usable events
  ADR-007: single family (notification_event); confidence = mean evidence when emitted
  Optional explanation factors: category / interruption_level / app_kind shares when present
  Calm copy only — not “you are overloaded” / clinical ADHD / workplace productivity scoring
```
**Phase 18 closed.** Next slice → **PM-GATE-POST-P18**.


#### Rejected alternatives

1. **IDE as Phase 18 primary** — still no additive privacy-safe session kind beyond `context_window` / CSR.
2. **Weather ambient** — implies cloud API and/or geo; deferred.
3. **App Store packaging product** — commercial track, not Core Feature unlock.
4. **Content capture “for accuracy”** — Non-Goal; forbid title/body/message.
5. **Workplace surveillance framing** — personal self-tracking only.
6. **`CognitiveLoad` as E3 target** — needs more inputs (meetings + CSR + notifications); later phase.
7. **Separate `notification_burst` family** — coalescing via `count` on `notification_event` is enough for v1.
8. **Parallel notification SQLite store / migration** — ADR-006 stance; observations only.
9. **Always-on / busy-loop poll** — Global DoD idle footprint; event or rare ≥5s.
10. **Opening a PR during freeze** — local branch `phase/18-notification-pressure` until 2026-09-01.
11. **Applying a migration without user approve** — this ADR chooses no migration.
12. **LLM inventing notification payloads or Features** — L5 interpret-only.

### ADR-020 detail — Phase 19: privacy-safe Notification Center OS mapping + live probe boundaries (v1)

> **Relationship:** **ADR-019** shipped `notification_event` + `NotificationPlugin` + **`NotificationPressure`**, but production `SystemNotificationEventProbe` intentionally soft-fails idle until a privacy-safe OS mapping exists. **PM-GATE-POST-P18** chose Live NC OS mapping as Phase 19 primary. **ADR-020** = unlock that probe without widening the Observation privacy bar. Pattern mirrors **ADR-014** (allowlist unlock after Git soft-fail). Implement live mapping in **P19-E2**; optional dogfood/status notes in **P19-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 19.

| Layer | Role in Phase 19 v1 |
| :--- | :--- |
| Capability Plugin (`plugin-sdk` + `macos-collector`) | Existing `com.biofocus.macos.notifications` / `NotificationPlugin`; E2 wires live `SystemNotificationEventProbe` |
| OS mapping (this ADR) | **Hybrid** privacy-safe NC metadata → ADR-019 payload fields only |
| Observation (SQLite) | Unchanged ADR-019 rows in `observations` (`data_type = 'notification_event'`) — **no** new tables/columns |
| Feature (`feature-engine`) | Existing **`NotificationPressure`** — **no** formula rewrite in Phase 19 unless a later ADR finds a justified confidence tweak (none for v1) |
| LLM | L5 interpret-only — must **not** invent notification content, labels, or Feature scores |

#### Chosen v1 OS/API stance: **hybrid + soft-fail**

Exactly **one** stance for Phase 19 v1:

1. **Prefer public macOS APIs** when they can supply delivery **counts** and/or **non-content** classification (interruption band, poster identity coarse enough to map to closed-set `app_kind`) **without** title/body/subtitle/message/userInfo/attachments.
2. **Honest gap:** as of this ADR, there is **no** supported public API that lets a third-party Desktop app observe *other apps'* Notification Center deliveries with useful metadata while staying content-free. Claiming “public-only forever” would strand dogfood on soft-fail idle.
3. Therefore E2 **may** use a **private / undocumented / unstable** OS surface (daemon metadata, NC persistence, or equivalent) **only** under a hard **field allowlist**:
   - May read: delivery timestamps, coalesced delivery counts, poster **bundle id** / process identity (for **in-memory** closed-set mapping only), OS interruption/importance **class codes** when available without text.
   - Must **never** read, select, deserialize, OCR, or log: title, body, subtitle, message, thread/preview text, attachment bytes, screenshots, contact names, deep-link URLs, or userInfo/dictionary dumps — even transiently “for classification”.
4. Map in-memory signals → ADR-019 closed-set labels only (`category` / `interruption_level` / `app_kind`) + required `count` ≥ 1. **Do not** persist raw bundle ids, display names, or free-form strings in Observation payloads (zero schema change; no justified closed-set extension in E1).
5. **Soft-fail idle** (`Ok(None)` / no emit) when: opt-in off; mapping unavailable; entitlement / TCC / schema drift; only content-bearing fields would be available; or mapping would require forbidden reads.
6. Emit cadence unchanged: on delivery / coalesced identity change, or rare poll ≥5s; **no** busy-loop; `stop_stream` joins.
7. Opt-in unchanged: `BIOFOCUS_NOTIFICATION_EVENTS=1` (default **off**); Desktop `ingest_host` starts plugin only when set.
8. Observation `confidence`: coarse / incomplete label mapping → lower confidence OK (ADR-019 already allows).

**Why hybrid (not public-only, not unrestricted private)**

1. **Closes ADR-019 soft-fail gap** for personal dogfood of shipped `NotificationPressure`.
2. **Privacy bar stays enforceable** via field allowlist — same Non-Goal as ADR-019 (no content capture).
3. **Capability Model fit:** extend existing notifications plugin + host — no parallel marketplace crate, no second Observation bus.
4. **Personal self-tracking only** — not workplace / employer notification monitoring.

#### Privacy contract (reaffirmed)

| May store / appear | Must **never** |
| :--- | :--- |
| Observation `id`, `timestamp`, `provider_id`, `data_type`, required `count`, optional closed-set `category` / `interruption_level` / `app_kind`, `confidence` | Notification **body**, **title**, **subtitle**, message text, thread IDs, deep-link URLs, userInfo dumps |
| Logs (prefer): Observation `id`, emit/drop **counts**, closed-set labels, mapping **ok / unavailable / soft-fail** reason codes | Screenshots, attachment bytes, contact names, email subjects, chat previews |
| In-memory only: bundle id → closed-set `app_kind` map (never written to payload / default logs) | Free-form app display names or always-on content dumps “for accuracy” |
| Accessibility / UI scrape of banner text as a classification source | Workplace / employer surveillance framing |

Pipeline normalize (already shipped) continues to **strip** accidental content keys (`title` / `body` / `subtitle` / `message` / `screenshot` / `userInfo`).

#### Schema (v1)

- Prefer existing `observations` store only.
- **ADR-019 payload unchanged** — **no** closed-set label extension in this ADR.
- **No new SQLite tables/columns** for NC registries, allowlists, or Feature history.
- **Do not apply** any migration for this ADR.
- Future richer taxonomy / bundle allowlists → **new ADR + user approve**.

#### Capability / live probe sketch

```text
BIOFOCUS_NOTIFICATION_EVENTS=1
  → Desktop ingest_host starts NotificationPlugin
  → SystemNotificationEventProbe attempts hybrid OS mapping
  → if mapping unavailable / would need content fields → soft-fail idle (no emit)
  → else emit ADR-019 Observations (count + optional closed-set labels only)
  → bounded Observation channel → spawn_persist_worker → SQLite
  → existing NotificationPressure consumes rows (no Feature rewrite)
UI ↛ SQLite (unchanged)
ScriptedNotificationEventProbe remains for tests / CI
```

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 19)

```text
E1 (P19-E1-T1) — THIS ADR ← Done 2026-08-11 (QA Pass)
  Lock hybrid OS stance + privacy field allowlist + soft-fail boundaries
  ADR-019 payload unchanged; no migration

E2 (P19-E2-T1) — Live probe ← Done 2026-08-11 (QA Pass)
  SystemNotificationEventProbe → usernoted NC SQLite (allowlisted columns only)
  → existing Observation channel → persist
  Keep ScriptedNotificationEventProbe for tests
  No Feature formula rewrite; no SQLite migration
  Soft-fail idle remains OK when mapping truly unavailable

E3 (P19-E3-T1) — companion ← Done 2026-08-11 (QA Pass)
  Dogfood runbook in docs/12-development.md (§ Notification events dogfood)
  Calm soft-fail / FDA note; verify NotificationPressure via snapshot IPC / fixtures
  Still no Observation payload widening; still no NotificationPressure math rewrite
```
**Phase 19 closed.** Next slice → **PM-GATE-POST-P19**.

#### Docs policy for this ADR

- Planned / ADR notes land now in `docs/08-plugin-sdk.md`, `docs/07-contracts.md`, `docs/10-security.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Live probe implementation + exact OS surface chosen in **P19-E2** (must still obey field allowlist). Optional dogfood/status in **P19-E3**.

#### Rejected alternatives

1. **Content capture “for accuracy”** (title/body/subtitle/message/screenshots/userInfo) — Non-Goal; forbid even transient reads for classification.
2. **Accessibility / UI banner text scrape** — content capture by another name.
3. **Workplace / employer monitoring** framing or manager dashboards — personal self-tracking only.
4. **Always-on / busy-loop poll** — Global DoD idle footprint; event or rare ≥5s.
5. **Rewriting `NotificationPressure` formula** in Phase 19 — Feature stays; unlock inputs only.
6. **IDE / weather ambient / App Store packaging / `CognitiveLoad` as Phase 19 primary** — deferred (PM-GATE-POST-P18).
7. **Parallel notification SQLite store / migration** — ADR-006 / ADR-019 stance; observations only.
8. **Public-APIs-only forever** (with no private fallback) — would leave production probe soft-fail idle indefinitely given current public API surface.
9. **Unrestricted private API** without field allowlist — rejects privacy bar.
10. **Widening Observation payload** with `bundle_id` / display names in v1 — prefer zero schema change; map to closed-set in probe.
11. **Opening a PR during freeze** — local branch `phase/19-live-nc-mapping` until 2026-09-01.
12. **Applying a migration without user approve** — this ADR chooses no migration.
13. **LLM inventing notification payloads or Features** — L5 interpret-only.

### ADR-021 detail — Phase 20: CognitiveLoad Feature scope (v1)

> **Relationship:** Catalog backlog named **`CognitiveLoad`** for years as a combined demand proxy from meetings + context switches + notifications. **ADR-019 / ADR-020** shipped `notification_event` + live NC mapping + **`NotificationPressure`**. **MeetingDensity** and **ContextSwitchRate** already ship. **PM-GATE-POST-P19** chose CognitiveLoad as Phase 20 primary. **ADR-021** = lock Feature-level inputs, formula stance, omit policy, and framing **before** math lands in `feature-engine`. Implement in **P20-E2**; optional dogfood / calm Dashboard surface in **P20-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 20.

| Layer | Role in Phase 20 v1 |
| :--- | :--- |
| Inputs (Feature-level) | Upstream catalog Features: **`MeetingDensity`**, **`ContextSwitchRate`**, **`NotificationPressure`** for the **same** window |
| Feature (`feature-engine`) | E2: new catalog node **`CognitiveLoad`** — composite demand proxy 0–100; ADR-007 confidence; optional explanation factors |
| Observation (SQLite) | **Unchanged** — reuse existing calendar / context / notification families via the three leaf Features; **no** new `data_type` |
| Leaf Features | **Do not** rewrite MeetingDensity / CSR / NotificationPressure formulas in this phase |
| LLM | L5 interpret-only — must **not** invent CognitiveLoad scores or clinical overload claims |

#### Chosen v1 input set: Feature-level (not Observation mix)

Exactly **one** input stance for Phase 20 v1:

1. **Prefer Feature-level:** `CognitiveLoad(window)` reads the three already-computed Features for that window (DAG depends on them).
2. **Why not Observation-level mix:** leaf Features already own omit policy, provenance, ADR-007 confidence, and privacy stripping (especially notifications — no body/title). Re-deriving from raw Observations would duplicate catalog math and risk bypassing leaf omit/privacy rules.
3. **Hard gap check:** no missing Observation family for this composite — calendar, context_window, and notification_event already feed the three leaves. **Zero** new Observation `data_type` in v1.

#### Formula stance sketch (for P20-E2)

```text
Window / step: 15 minutes / 1 minute (align Focus / CSR / NotificationPressure; series may coarsen)

Normalize each present input to [0, 100]:
  meeting   = MeetingDensity × 100                    # fraction → intensity
  switches  = clamp(ContextSwitchRate × 50, 0, 100)   # same CSR map as DistractionScore optional term
  notify    = NotificationPressure                    # already 0–100

Catalog weights (equal thirds):
  w_meeting = 1/3, w_switches = 1/3, w_notify = 1/3

Missing-input policy (LOCKED — pick one):
  • If NONE of the three Features are present for the step → OMIT CognitiveLoad
  • If ONE OR MORE present → EMIT with RENORMALIZED weights over present components
    value = Σ (component × w_i) / Σ w_present ; clamp to [0, 100]
  (Do NOT force all-three-required omit — NotificationPressure is often empty when opt-in is off.)

Confidence (ADR-007 sketch):
  expected_slots = 3
  present_slots  = count of present input Features
  coverage       = present_slots / 3
  mean_evidence  = mean(upstream Feature.confidence of present inputs)
  Feature.confidence = clamp(coverage × mean_evidence, 0.0, 1.0)

Provenance: union of Observation IDs from present upstream Features (or Feature ids if IPC already carries them — E2 follows existing composite patterns, e.g. FatigueIndex / DistractionScore).

Optional explanation factors (when emitted):
  id/label for present components — meeting (“Schedule demand”), switches (“App switching”),
  notifications (“Interruption intensity”); share = catalog_weight / sum(present weights).
  Calm composition only — not clinical.

Framing / copy:
  “combined demand in this window” — NOT “you are overloaded” / ADHD / burnout / cognitive overload diagnosis.
```

#### Schema (v1)

- Prefer existing `observations` store + existing Feature DAG only.
- **No new SQLite tables/columns**; **no** Feature-history store.
- **Do not apply** any migration for this ADR.
- Future alternate inputs / Observation-level variants → **new ADR + user approve**.

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 20)

```text
E1 (P20-E1-T1) — THIS ADR
  Lock CognitiveLoad Feature scope: Feature-level inputs, 15m/1m, renormalize-partial / omit-none,
  ADR-007 slots=3, calm framing, rejected alts; no migration

E2 (P20-E2-T1)
  Ship CognitiveLoad in feature-engine + catalog §1 finalize
  register_* / register_catalog_v1 after calendar + focus + notification nodes
  Unit tests: rich emit (all three) / partial renormalize / omit empty / confidence / factors
  Do NOT rewrite MeetingDensity / CSR / NotificationPressure

E3 (optional)
  Dogfood / calm Dashboard surface (snapshot series) — still non-clinical copy
```

#### Docs policy for this ADR

- Planned → ADR sketch notes land now in `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Full §1 formula finalize + DAG registration names → **P20-E2** (must still obey this ADR).

#### Rejected alternatives

1. **Clinical “cognitive overload” / burnout / ADHD diagnosis claims** — Non-Goal; calm personal observation only.
2. **Workplace / employer surveillance scoring** or manager dashboards — personal self-tracking only.
3. **Inventing new Observation families** this phase — all required inputs already ship via leaf Features.
4. **Rewriting MeetingDensity / ContextSwitchRate / NotificationPressure** formulas — compose, do not reopen leaf math.
5. **Observation-level mix as v1 primary** — rejected in favor of Feature-level composition (see above); revisit only with a new ADR if a hard gap appears.
6. **Omit-unless-all-three** as the missing-input policy — too sparse when notifications opt-in is off; renormalize-partial is locked instead.
7. **IDE plugin / weather ambient / App Store packaging as Phase 20 primary** — deferred (PM-GATE-POST-P19).
8. **Opening a PR during freeze** — local branch `phase/20-cognitive-load` until 2026-09-01.
9. **Applying a migration without user approve** — this ADR chooses no migration.
10. **LLM inventing CognitiveLoad scores or overload diagnoses** — L5 interpret-only.
11. **Parallel “Load Engine” crate** — extend `feature-engine` catalog only.

### ADR-022 detail — Phase 21: DeepWorkScore Feature scope (v1)

> **Relationship:** Catalog backlog named **`DeepWorkScore`** (P7-era) as sustained-focus windows from **FocusScore** + **ContextSwitchRate** (+ optional “idle”). **FocusScore** and **CSR** have long shipped. Phase 20 shipped demand-side **`CognitiveLoad`**. **PM-GATE-POST-P20** chose DeepWorkScore as Phase 21 primary (complement: demand vs sustained-focus quality). **ADR-022** = lock Feature-level inputs, formula stance, omit policy, and framing **before** math lands in `feature-engine`. Implement in **P21-E2**; optional dogfood / calm Dashboard surface in **P21-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 21.

| Layer | Role in Phase 21 v1 |
| :--- | :--- |
| Inputs (Feature-level) | Upstream catalog Features: **`FocusScore`** (required) + **`ContextSwitchRate`** (optional stability term) for the **same** window |
| Feature (`feature-engine`) | E2: new catalog node **`DeepWorkScore`** — sustained-focus intensity 0–100; ADR-007 confidence; optional explanation factors |
| Observation (SQLite) | **Unchanged** — reuse existing keystrokes / context / HRV via FocusScore + CSR; **no** new `data_type` |
| Leaf Features | **Do not** rewrite FocusScore / ContextSwitchRate formulas; **do not** invent a parallel FocusScore |
| LLM | L5 interpret-only — must **not** invent DeepWorkScore scores or “flow state” claims |

#### Chosen v1 input set: Feature-level Focus + CSR (idle dropped)

Exactly **one** input stance for Phase 21 v1:

1. **Prefer Feature-level:** `DeepWorkScore(window)` reads upstream **`FocusScore`** and (when present) **`ContextSwitchRate`** for that window (DAG depends on both nodes).
2. **Idle dropped for v1:** backlog listed “idle”, but there is **no** shipped idle Feature / dedicated idle Observation family. Mapping “idle” onto thin heuristics (e.g. inventing gaps from raw context) would reopen Observation math and risk a parallel FocusScore. Idle / gap proxies → later ADR if a clear leaf exists.
3. **Why not Observation-level mix:** FocusScore already owns typing / CSR-stability / HRV composition + ADR-007. Re-deriving from Observations would duplicate Focus math — Non-Goal.
4. **Hard gap check:** FocusScore + CSR already ship. **Zero** new Observation `data_type` in v1.

#### Formula stance sketch (for P21-E2)

```text
Window / step: 15 minutes / 1 minute (align Focus / CSR; series may coarsen)

Compose high Focus + low CSR into sustained-focus intensity (0–100):
  focus      = FocusScore                                    # already 0–100
  stability  = clamp(100 - ContextSwitchRate × 50, 0, 100) # same CSR map as FocusScore stability term
                                                    # (high when switches are low)

Catalog weights:
  w_focus = 0.60, w_stability = 0.40
  (Focus is the primary sustained-focus signal; CSR refines — not a second FocusScore.)

Missing-input policy (LOCKED):
  • If FocusScore is ABSENT for the step → OMIT DeepWorkScore
  • If FocusScore present and CSR absent → EMIT with RENORMALIZED weights (Focus-only)
  • If both present → EMIT with catalog weights
  (Do NOT invent Focus from Observations; do NOT emit CSR-only as “deep work”.)

Confidence (ADR-007 sketch):
  expected_slots = 2   # Focus / CSR
  present_slots  = 1 if Focus-only; 2 if Focus+CSR
  coverage       = present_slots / 2
  mean_evidence  = mean(upstream Feature.confidence of present inputs)
  Feature.confidence = clamp(coverage × mean_evidence, 0.0, 1.0)

Provenance: union of Observation IDs from present upstream Features
  (follow existing composite patterns, e.g. FatigueIndex / CognitiveLoad).

Optional explanation factors (when emitted):
  focus (“Focus depth”), stability (“App stability”) when CSR present;
  share = catalog_weight / sum(present weights). Calm composition only — not clinical.

Framing / copy:
  “sustained focus in this window” — NOT “you are in flow” / burnout / ADHD / flow-state diagnosis.
```

#### Schema (v1)

- Prefer existing `observations` store + existing Feature DAG only.
- **No new SQLite tables/columns**; **no** Feature-history store.
- **Do not apply** any migration for this ADR.
- Future idle leaf / Observation-level variants → **new ADR + user approve**.

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 21)

```text
E1 (P21-E1-T1) — THIS ADR
  Lock DeepWorkScore Feature scope: Focus required + optional CSR; idle dropped;
  15m/1m; omit-without-Focus / renormalize-without-CSR; ADR-007 slots=2;
  calm framing; rejected alts; no migration

E2 (P21-E2-T1)
  Ship DeepWorkScore in feature-engine + catalog §1 finalize
  register_* / register_catalog_v1 after focus nodes (CSR + FocusScore)
  Unit tests: rich Focus+CSR / Focus-only renormalize / omit without Focus /
  confidence / factors
  Do NOT rewrite FocusScore / ContextSwitchRate

E3 (optional)
  Dogfood / calm Dashboard surface (snapshot series) — still non-clinical copy
```

#### Docs policy for this ADR

- Planned → ADR sketch notes land now in `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Full §1 formula finalize + DAG registration names → **P21-E2** (must still obey this ADR).

#### Rejected alternatives

1. **Clinical “flow state” / burnout / ADHD diagnosis claims** — Non-Goal; calm personal observation only.
2. **Workplace / employer surveillance scoring** or manager dashboards — personal self-tracking only.
3. **Inventing new Observation families** this phase (including a dedicated idle family) — Focus + CSR already ship.
4. **Rewriting FocusScore / ContextSwitchRate** formulas or inventing a parallel FocusScore — compose, do not reopen leaf math.
5. **Keeping backlog “idle” as a required v1 input** — dropped (see above); revisit with a new ADR when a real idle leaf exists.
6. **CSR-only emit as DeepWorkScore** — without Focus there is no sustained-focus intensity signal under this ADR.
7. **IDE plugin / weather ambient / App Store packaging as Phase 21 primary** — deferred (PM-GATE-POST-P20).
8. **AttentionStability or CircadianOffset as this-phase primary** — deferred siblings (PM-GATE-POST-P20).
9. **Opening a PR during freeze** — local branch `phase/21-deep-work-score` until 2026-09-01.
10. **Applying a migration without user approve** — this ADR chooses no migration.
11. **LLM inventing DeepWorkScore scores or flow diagnoses** — L5 interpret-only.
12. **Parallel “Deep Work Engine” crate** — extend `feature-engine` catalog only.

### ADR-023 detail — Phase 22: AttentionStability Feature scope (v1)

> **Relationship:** Catalog backlog named **`AttentionStability`** (P7-era) as variance of focus / switches from **FocusScore** + **ContextSwitchRate**. Phase 21 shipped sibling **`DeepWorkScore`** — sustained-focus **intensity** on the same leaves. **PM-GATE-POST-P21** chose AttentionStability as Phase 22 primary (complement: consistency/stability vs intensity). **ADR-023** = lock Feature-level inputs, formula stance, omit policy, and framing **before** math lands in `feature-engine`. Implement in **P22-E2**; optional dogfood / calm Dashboard surface in **P22-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 22.

| Layer | Role in Phase 22 v1 |
| :--- | :--- |
| Inputs (Feature-level) | Upstream catalog Features: **`FocusScore`** (required) + **`ContextSwitchRate`** (optional switch-stability term) for the **same** window |
| Feature (`feature-engine`) | E2: new catalog node **`AttentionStability`** — focus/switch stability 0–100; ADR-007 confidence; optional explanation factors |
| Observation (SQLite) | **Unchanged** — reuse existing keystrokes / context / HRV via FocusScore + CSR; **no** new `data_type` |
| Leaf Features | **Do not** rewrite FocusScore / ContextSwitchRate / DeepWorkScore formulas; **do not** invent a parallel FocusScore |
| Sibling | **Distinct from DeepWorkScore:** stability/variance ≠ sustained-focus intensity |
| LLM | L5 interpret-only — must **not** invent AttentionStability scores or ADHD / “can’t focus” claims |

#### Chosen v1 input set: Feature-level Focus + CSR (variance framing)

Exactly **one** input stance for Phase 22 v1:

1. **Prefer Feature-level:** `AttentionStability(window)` reads upstream **`FocusScore`** sample(s) and (when present) **`ContextSwitchRate`** for that window (DAG depends on both nodes).
2. **Distinct from DeepWorkScore:** DeepWorkScore composes **Focus level** + low CSR into intensity. AttentionStability composes **Focus consistency** (low in-window Focus **range**) + low CSR into stability — **do not** reuse DeepWorkScore’s Focus-level × CSR intensity math.
3. **Why not Observation-level mix:** FocusScore / CSR already own leaf math + ADR-007. Re-deriving from Observations would duplicate Focus math — Non-Goal.
4. **Hard gap check:** FocusScore + CSR already ship. **Zero** new Observation `data_type` in v1.

#### Formula stance sketch (for P22-E2)

```text
Window / step: 15 minutes / 1 minute (align Focus / CSR; series may coarsen)

Compose LOW variance / HIGH stability (0–100) — not Focus intensity:
  # Focus consistency from in-window FocusScore samples (Feature ends in window)
  focus_samples = FocusScore values with ends in [window_start, window_end]
  if |focus_samples| >= 2:
    focus_range = max(focus_samples) - min(focus_samples)   # 0–100 scale
    focus_stability = clamp(100 - focus_range, 0, 100)      # low range → high stability
  else:  # exactly one Focus sample in window
    focus_stability = 100   # no swing observed yet — not a DeepWorkScore Focus-level term

  # Switch stability (same CSR map as FocusScore stability / DeepWorkScore CSR term)
  switch_stability = clamp(100 - ContextSwitchRate × 50, 0, 100)

Catalog weights:
  w_focus_stab = 0.50, w_switch_stab = 0.50
  (Equal thirds avoided — two slots; neither is DeepWorkScore intensity.)

Missing-input policy (LOCKED):
  • If FocusScore is ABSENT for the step → OMIT AttentionStability
  • If FocusScore present and CSR absent → EMIT with RENORMALIZED weights (Focus-stability only)
  • If both present → EMIT with catalog weights
  (Do NOT invent Focus from Observations; do NOT emit CSR-only as “attention stability”;
   do NOT copy DeepWorkScore Focus-level intensity formula.)

Confidence (ADR-007 sketch):
  expected_slots = 2   # Focus / CSR
  present_slots  = 1 if Focus-only; 2 if Focus+CSR
  coverage       = present_slots / 2
  mean_evidence  = mean(upstream Feature.confidence of present inputs)
  Feature.confidence = clamp(coverage × mean_evidence, 0.0, 1.0)

Provenance: union of Observation IDs from present upstream Features
  (follow existing composite patterns, e.g. DeepWorkScore / CognitiveLoad).

Optional explanation factors (when emitted):
  focus_stability (“Focus consistency”), switch_stability (“Switch steadiness”) when CSR present;
  share = catalog_weight / sum(present weights). Calm composition only — not clinical.

Framing / copy:
  “focus stability in this window” — NOT “you have ADHD” / “you can’t focus” / burnout /
  attention-deficit diagnosis. Distinct from DeepWorkScore “sustained focus in this window”.
```

#### Schema (v1)

- Prefer existing `observations` store + existing Feature DAG only.
- **No new SQLite tables/columns**; **no** Feature-history store (in-window Focus samples come from the same DAG batch / recompute-on-read Feature set — not a new persistence model).
- **Do not apply** any migration for this ADR.
- Future Observation-level / longer-horizon variance store → **new ADR + user approve**.

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 22)

```text
E1 (P22-E1-T1) — THIS ADR
  Lock AttentionStability Feature scope: Focus required + optional CSR;
  Focus range (not level) + CSR switch stability; 15m/1m;
  omit-without-Focus / renormalize-without-CSR; ADR-007 slots=2;
  distinct from DeepWorkScore; calm framing; rejected alts; no migration

E2 (P22-E2-T1)
  Ship AttentionStability in feature-engine + catalog §1 finalize
  register_* / register_catalog_v1 after focus / DeepWork nodes as appropriate
  Unit tests: rich Focus+CSR / Focus-only / single-vs-multi Focus range /
  omit without Focus / confidence / factors
  Do NOT rewrite FocusScore / ContextSwitchRate / DeepWorkScore

E3 (optional)
  Dogfood / calm Dashboard surface (snapshot series) — still non-clinical copy
```

#### Docs policy for this ADR

- Planned → ADR sketch notes land now in `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Full §1 formula finalize + DAG registration names → **P22-E2** (must still obey this ADR).

#### Rejected alternatives

1. **Clinical ADHD / attention-deficit / “you can’t focus” claims** — Non-Goal; calm personal observation only.
2. **Workplace / employer surveillance scoring** or manager dashboards — personal self-tracking only.
3. **Inventing new Observation families** this phase — Focus + CSR already ship.
4. **Rewriting FocusScore / ContextSwitchRate / DeepWorkScore** formulas or inventing a parallel FocusScore — compose, do not reopen leaf / sibling math.
5. **Using FocusScore level as intensity** (DeepWorkScore math) — rejected for this Feature; AttentionStability is variance/stability.
6. **CSR-only emit as AttentionStability** — without Focus there is no focus-consistency signal under this ADR.
7. **IDE plugin / weather ambient / App Store packaging as Phase 22 primary** — deferred (PM-GATE-POST-P21).
8. **CircadianOffset as this-phase primary** — deferred (PM-GATE-POST-P21).
9. **Opening a PR during freeze** — local branch `phase/22-attention-stability` until 2026-09-01.
10. **Applying a migration without user approve** — this ADR chooses no migration.
11. **LLM inventing AttentionStability scores or ADHD diagnoses** — L5 interpret-only.
12. **Parallel “Attention Engine” crate** — extend `feature-engine` catalog only.


### ADR-024 detail — Phase 23: Personal Context Layer (v1)

> **Relationship:** Focus-ladder catalog Features (`FocusScore`, `DeepWorkScore`, `AttentionStability`, `CognitiveLoad`, …) are shipped. The next product gap is **interpretation context**: how we orient numbers (personal + literature), optional user-declared health framing for L5 reports, and whether the person left the desk — **without** becoming a medical device or a GPS tracker. **PM-GATE-POST-P22** chose **Personal Context Layer** (supersedes same-day CircadianOffset draft). **ADR-024** locks three pillars, layer relationships, E2 ship order, and rejected alts **before** implementation. Epic split: `docs/SPRINT_ROADMAP.md` Phase 23.

#### Three locked pillars

| Pillar | v1 lock |
| :--- | :--- |
| **1. Reference bands = Variant B** | **Personal baseline primary** (existing ADR-008 recompute-on-read patterns). **Literature bands secondary** — cited, calm **orienting ranges** only (“often discussed around …”); **never** “нельзя / diagnosis / clinical cutoff” tone. |
| **2. Health context** | Opt-in **user-declared** curated conditions the user already knows. v1 **consumer = prompt packs / report** (L5 interpret-only). May sketch later Insight soft-framing with Evidence. **Must not** invent disease from HR/HRV. **Must not** rewrite Feature formulas from disease labels in v1. |
| **3. Desk-away / presence** | Infer leave-desk / likely break-or-walk from **secondary signals** (prefer existing Observations). **Reject precise GPS / continuous geo** for v1. Calm copy: “away from desk in this window” — **not** workplace surveillance. |

#### Layer relationships (how pieces connect)

| Layer | Role in Personal Context Layer v1 |
| :--- | :--- |
| `Observation` (SQLite) | Immutable facts — keystrokes, `context_window`, `step_count`, `life_event` (`walk`), … Unchanged store; **no** GPS Observation family in v1. |
| `Feature` (`feature-engine`) | Windowed metrics stay provider-agnostic (consume Observation **contracts**, not Apple-only APIs). New candidate: **`DeskAwayPresence`** (E2). Existing Focus/Stress/… formulas **not** rewritten from health tags. |
| `Insight` (`knowledge-engine`) | Variant B reference-band Insights via a **case catalog** (one rule family per Feature/context — not one mega-rule). Personal-baseline cases first; literature-orienting cases thin / later. Soft health framing only with Evidence + declared context — never diagnose. |
| `Recommendation` | Unchanged L4 contract; may later reference desk-away / baseline Insights — **not** medical prescriptions. |
| Prompt packs / report (`report-engine`) | **Primary v1 consumer of health context** — inject declared conditions into interpret-only `llm_prompt` / offline markdown as **user-supplied context**, not as diagnoses. LLM remains L5. |
| Local profile config | Prefer **`~/.biofocus/…` files** (same pattern as Git watched roots ADR-014) for health context + optional literature-cite prefs. **No** new SQLite profile schema / **no** migration in v1 unless a future ADR + user approve. |
| UI | IPC only; UI ↛ SQLite. Optional calm surface for desk-away / health declaration → **E3**. |

#### Provider universality (product stance)

- **Core path is universal:** any source posting valid `Observation` JSON to local ingest works (`provider_id` + `data_type` contracts).
- **Dogfood path today:** Mac Desktop + iOS Companion + Apple Health (Watch; Mi Band via Health OK).
- **Not architectural lock-in:** Features / Insights / prompt packs **must not** hard-code “only Apple Watch” or Apple-only APIs. New platforms = new Capability plugins / companions behind the same contracts.

#### Formula / product stance sketches (for P23-E2)

##### A. Reference bands (Variant B) — case catalog

```text
Approach: case-catalog Insight rules — NOT one mega-rule for all Features.

Per case (example shape):
  • id: focus_vs_personal_afternoon_v1   (personal baseline — already patterned ADR-008)
  • id: focus_vs_literature_orient_v1    (optional later) — cites source; calm copy only
  • Compare current Feature vs personal baseline FIRST
  • Literature band (if present) is SECONDARY orienting note — never “you are unhealthy”
  • Thin history / missing baseline → omit Insight (Ok([]))
  • Large literature library is OUT of E2 first slice
```

##### B. Health context — local config shape

```text
Path (v1): ~/.biofocus/health-context.toml   # mirror Git folders pattern
Enable: opt-in (missing/empty file = no health context injected)

Sketch:
  [health_context]
  # Closed-set curated ids the user already knows (examples — finalize in E2):
  # conditions = ["sleep_sensitive", "migraine_prone", "caffeine_sensitive"]
  conditions = []
  # Optional free-text, local-only, never synced by default:
  # note = "I already know I am sensitive to late caffeine"
  note = ""

Rules:
  • User-declared only — Core never invents conditions from biometrics
  • Prompt packs may include declared ids + note as interpret-only context
  • Feature math MUST NOT branch on condition ids in v1
  • No cloud health-records sync by default; no SQLite migration
```

##### C. Desk-away — Feature candidate `DeskAwayPresence`

```text
Working name: DeskAwayPresence
Goal: calm likelihood the person is away from the desk / on a break-or-walk in the window
Framing: “away from desk in this window” — NOT employer presence monitoring / GPS tracking

Window / step: prefer 15 minutes / 1 minute (align catalog)

Inputs (prefer EXISTING Observations — Feature or Observation-level compose OK if documented in E2):
  • Quiet / absent keystrokes in window
  • Quiet / stable or absent context_window (no active desktop focus)
  • Optional: step_count cadence rise (walk-like)
  • Optional: life_event kind = walk
  • Optional later (not required for E2): screen-lock / idle OS signal — only if privacy-safe + ADR note

Omit policy (LOCKED sketch):
  • Insufficient evidence to distinguish “idle at desk” vs “away” → OMIT (no false presence claims)
  • Do NOT emit from precise geo
  • Do NOT invent a GPS / continuous-location Observation family in v1

Output: Float 0–100 (away-from-desk likelihood) OR omit
Confidence: ADR-007 expected slots sketched in E2 from present evidence families
Optional factors: calm ids only (e.g. input_quiet / steps / walk_event)

Schema: prefer NO new Observation data_type; NO migration.
  Thin new Observation only if E2 proves an unavoidable gap — requires ADR amend + approve.
```

#### E2 / E3 sketch (LOCKED ship order)

```text
E1 (P23-E1-T1) — THIS ADR
  Lock Personal Context Layer: Variant B + health context + desk-away;
  layer relationships; config preference; universality; rejected alts; no migration

E2 (P23-E2-T1) — FIRST IMPLEMENTATION SLICE (order LOCKED)
  1. PRIMARY: DeskAwayPresence Feature path from existing secondary signals
     (register_* / catalog stub → formula; unit tests; omit policy)
  2. SECONDARY (same epic if capacity, else immediate follow-on under E2):
     health-context local config + prompt-pack / report injection (L5 consume only)
  3. DEFER: large literature-band library — at most thin Variant B case stub(s);
     personal-baseline Insights already exist (ADR-008) — do not rebuild them

E3 (optional)
  Dogfood notes + optional calm UI (desk-away series / health-context declare surface)
  Still non-clinical; UI ↛ SQLite; no GPS
```

#### Schema / persistence (v1)

- Prefer existing `observations` store + `feature-engine` / `knowledge-engine` / `report-engine`.
- Health profile: **`~/.biofocus/health-context.toml`** (or equivalent file under `~/.biofocus/`) — **not** SQLite.
- **Do not apply** any migration for this ADR.
- Precise geo / location tables → **rejected**. Future schema → **new ADR + user approve**.

#### Docs policy for this ADR

- Planned names / stubs land now in `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Full Feature math + pack wiring → **P23-E2** (must still obey this ADR).

#### Rejected alternatives

1. **Precise GPS / continuous geo** for desk-away or walk tracking in v1 — Local-First privacy bar; secondary signals first.
2. **Clinical diagnosis engine** or “нельзя / diagnosis” reference-band tone — Non-Goal; Variant B only.
3. **LLM inventing health conditions** from biometrics — L5 interpret-only; user-declared only.
4. **Cloud health-records sync by default** — local config; opt-in sync would need a future ADR.
5. **Workplace / employer presence monitoring** dashboards — personal self-tracking only.
6. **Rewriting Focus / Stress / Recovery Feature formulas from disease tags** in v1 — context may frame reports; math stays disease-agnostic.
7. **CircadianOffset / IDE / weather / App Store as Phase 23 primary** — deferred (PM-GATE-POST-P22).
8. **Opening a PR during freeze** — local branch `phase/23-personal-context` until 2026-09-01.
9. **Applying a migration without user approve** — this ADR chooses no migration.
10. **Hard-coding Apple-only APIs into Features** — contracts stay provider-agnostic.
11. **One mega Insight rule** for all reference bands — case catalog only.
12. **Large literature library as E2 primary** — deferred behind desk-away + health→prompt.


### ADR-025 detail — Phase 24: CircadianOffset Feature scope (v1)

> **Relationship:** Catalog backlog named **`CircadianOffset`** as alignment of work vs chronotype proxy from **sleep + activity timing**, deferred until after Personal Context. Phase 23 shipped presence/context (`DeskAwayPresence` + health→prompt); P17 shipped `SleepDebt` / `EnergyScore` / `ActivityBalance` and sleep/activity Observations. **PM-GATE-POST-P23** chose CircadianOffset as Phase 24 primary. **ADR-025** locks Observation-level timing inputs, formula stance, omit policy, and calm framing **before** math lands in `feature-engine`. Implement in **P24-E2**; optional dogfood / calm Dashboard surface in **P24-E3**. Epic split: `docs/SPRINT_ROADMAP.md` Phase 24.

| Layer | Role in Phase 24 v1 |
| :--- | :--- |
| Inputs (Observation-level timing) | Already-shipped families: **`sleep_interval`** (sleep timing) + **work/activity timing** from **`keystrokes` / `context_window`** (preferred) with optional reinforcement **`step_count` / `active_energy` / workout `life_event`** |
| Feature (`feature-engine`) | E2: new catalog node **`CircadianOffset`** — schedule-alignment score 0–100; ADR-007 confidence; optional explanation factors |
| Observation (SQLite) | **Unchanged** — reuse existing sleep + desktop/wearable families; **no** new `data_type` |
| Leaf / sibling Features | **Do not** rewrite SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence; **do not** treat those magnitudes as timing proxies |
| Siblings | **Distinct from SleepDebt** (debt magnitude ≠ timing alignment) and **DeskAwayPresence** (away-from-desk ≠ circadian offset) |
| LLM | L5 interpret-only — must **not** invent CircadianOffset scores or chronotype / sleep-disorder claims |

#### Chosen v1 input set: Observation-level sleep timing + work/activity timing

Exactly **one** input stance for Phase 24 v1:

1. **Prefer Observation-level timing** (not Feature-level SleepDebt / EnergyScore / ActivityBalance): those Features measure **magnitude / volume / sufficiency**, not clock alignment of sleep midpoint vs work/activity midpoint.
2. **Sleep timing:** qualifying `sleep_interval` Observations (same rest stages as SleepDebt: `asleep` / `in_bed` / missing stage; `awake` / `unknown` do not contribute). Derive a lookback **sleep midpoint** from overlapping intervals.
3. **Work/activity timing:** preferred desktop work evidence — timestamps of active **`keystrokes`** and/or **`context_window`** in the lookback. Optional reinforcement when desk signals are thin: **`step_count`**, **`active_energy`**, or workout **`life_event`** as activity timing (still timing centroids — not ActivityBalance volume math).
4. **Why not Feature-level composite:** no shipped Feature exposes sleep midpoint or work midpoint; composing SleepDebt×ActivityBalance would invent a false “alignment” from debt × steps.
5. **Hard gap check:** sleep + keystrokes / context / steps / energy already ship (P3 / P17). **Zero** new Observation `data_type` in v1.

#### Formula stance sketch (for P24-E2)

```text
Feature cadence (window / step): 15 minutes / 1 minute (align catalog DAG; series may coarsen)
Lookback for timing math: 24 hours ending at window end
  (Longer lookback preferred — sleep + work schedule needs a day; mirrors SleepDebt rest lookback.
   Do NOT invent multi-day chronotype profiling in v1.)

Compose HIGH alignment (0–100) — NOT signed chronotype hours; NOT clinical labels:

  sleep_intervals = qualifying sleep_interval overlaps in [end−24h, end]
  if empty → OMIT

  sleep_mid = midpoint of union(sleep_intervals)   # UTC instant

  work_evidence = active keystrokes and/or context_window timestamps in lookback
  if work_evidence thin:
    may reinforce with step_count / active_energy / workout life_event timestamps
  if still empty → OMIT

  work_mid = centroid (time-weighted midpoint) of work/activity evidence

  # Expected mid-wake relative to recent sleep (calm schedule heuristic — not chronotype class)
  expected_wake_mid = sleep_mid + 12h   (circular day)
  offset_hours = circular_hours(|work_mid − expected_wake_mid|)   # 0..12

  # Map offset → alignment: 0h → 100; saturates toward 0 by ~6h
  value = clamp(100 × (1 − offset_hours / 6.0), 0, 100)

Units (LOCKED): dimensionless 0–100 alignment score.
  Rejected as primary units: signed hours / “owl vs lark” labels —
  signed offsets invite clinical / judgmental chronotype copy; 0–100 matches catalog Features.

Missing-input policy (LOCKED):
  • If sleep timing ABSENT → OMIT CircadianOffset
  • If work/activity timing ABSENT → OMIT CircadianOffset
  • Do NOT renormalize a single slot into “alignment”
  • Do NOT invent sleep from SleepDebt Feature or work from FocusScore level
  (Optional reinforcement inside the work/activity slot is OK when desk signals thin —
   that is within-slot evidence, not cross-slot renormalize.)

Confidence (ADR-007 sketch):
  expected_slots = 2   # sleep_timing / work_activity_timing
  present_slots  = 2 only when both present (else omit — no 1-slot emit)
  coverage       = present_slots / 2   # = 1.0 when emitted
  mean_evidence  = mean(Observation.confidence of contributing evidence)
  Feature.confidence = clamp(coverage × mean_evidence, 0.0, 1.0)
  (If E2 distinguishes rich desk work vs activity-only reinforcement, may down-weight
   mean_evidence — still obey omit-without-both-slots.)

Provenance: union of Observation IDs from present sleep + work/activity evidence.

Optional explanation factors (when emitted):
  sleep_timing (“Sleep timing”), work_timing (“Work timing”);
  optional activity_timing (“Activity timing”) when reinforcement used;
  shares sum to 1.0. Calm composition only — not clinical.

Framing / copy:
  “schedule alignment in this window” — NOT “wrong chronotype” / “you are a night owl
  so you fail” / circadian-disorder / sleep-disorder / burnout diagnosis.
  Distinct from SleepDebt (“rest shortfall”) and DeskAwayPresence (“away from desk”).
```

#### Schema (v1)

- Prefer existing `observations` store + existing Feature DAG only.
- **No new SQLite tables/columns**; **no** Feature-history store (24h lookback recompute-on-read from Observations — ADR-008 stance).
- **Do not apply** any migration for this ADR.
- Future multi-day chronotype profile / new Observation family → **new ADR + user approve**.

#### E2 / E3 sketch (aligned with SPRINT_ROADMAP Phase 24)

```text
E1 (P24-E1-T1) — THIS ADR
  Lock CircadianOffset Feature scope: Observation-level sleep + work/activity timing;
  15m/1m cadence + 24h lookback; 0–100 alignment (not signed hours);
  omit unless both slots present; ADR-007 slots=2;
  distinct from SleepDebt / DeskAwayPresence; calm framing; rejected alts; no migration

E2 (P24-E2-T1)
  Ship CircadianOffset in feature-engine + catalog §1 finalize
  register_* / register_catalog_v1 after wearable / focus nodes as appropriate
  Unit tests: rich sleep+work / sleep+activity-only / omit without sleep /
  omit without work/activity / confidence / factors
  Do NOT rewrite SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence

E3 (optional)
  Dogfood / calm Dashboard surface (snapshot series) — still non-clinical copy
  Suggested calm chart label: Schedule alignment
```

#### Docs policy for this ADR

- Planned → ADR sketch notes land now in `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`.
- Full §1 formula finalize + DAG registration names → **P24-E2** (must still obey this ADR).

#### Rejected alternatives

1. **Clinical chronotype / circadian-disorder / “you are a night owl so you fail” claims** — Non-Goal; calm personal schedule alignment only.
2. **Workplace / employer schedule surveillance** or manager dashboards — personal self-tracking only.
3. **Inventing new Observation families** this phase — sleep + work/activity inputs already ship.
4. **Rewriting SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence** leaf or sibling formulas — compose timing from Observations; do not reopen magnitude Features.
5. **Using SleepDebt / EnergyScore / ActivityBalance as timing proxies** (Feature-level composite) — rejected; those are not clock alignment.
6. **Signed offset hours / owl-lark labels as primary Feature units** — rejected; 0–100 alignment score only.
7. **Renormalize / emit with only sleep or only work** — without both slots there is no alignment signal under this ADR.
8. **IDE plugin / weather ambient / App Store packaging / TypingRhythm as Phase 24 primary** — deferred (PM-GATE-POST-P23).
9. **Precise GPS** for schedule / location chronobiology — rejected (ADR-024 bar; no reopen).
10. **Opening a PR during freeze** — local branch `phase/24-circadian-offset` until 2026-09-01.
11. **Applying a migration without user approve** — this ADR chooses no migration.
12. **LLM inventing CircadianOffset scores or chronotype diagnoses** — L5 interpret-only.
13. **Parallel “Circadian Engine” crate** — extend `feature-engine` catalog only.
