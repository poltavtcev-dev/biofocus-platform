# 12. Local Development Guide

> **Extending BioFocus** (where to add plugins, pipeline stages, Features, IPC): see root [`CONTRIBUTING.md`](../CONTRIBUTING.md). This file is local setup, commands, and git cadence.

## Workspace location (macOS / iCloud)

Prefer a **local** path outside iCloud Desktop/Documents (e.g. `~/Developer/AI Project/BioFocus`). iCloud “Optimize Mac Storage” can leave `dataless` stubs so `Cargo.toml` looks empty and `cargo` / `git` fail. Do not keep the active git worktree only on Desktop if Desktop syncs to iCloud.

## Prerequisites
- Rust stable (edition 2024) — `rustup` / `rust-toolchain.toml`
- Node.js >= 20.x, pnpm
- System deps for Tauri v2 on macOS (Xcode CLT)

## Commands
```bash
git clone https://github.com/poltavtcev-dev/biofocus-platform.git
cd biofocus-platform

# Core
cargo check
cargo check --workspace --exclude desktop
cargo test
cargo test -p storage
cargo test -p ingest
cargo test -p macos-collector
cargo test -p companion
cargo test -p pipeline
cargo test -p desktop

# Desktop (Epic E3 Done)
cd apps/desktop
pnpm install
pnpm tauri dev
# pnpm tauri build  # → BioFocus.app

# QA mock statuses (webview)
# http://localhost:…/?mockStatus=idle|ready|error
# http://localhost:…/?mockAlert=green|yellow|red
# combine: ?mockStatus=ready&mockAlert=yellow
```

Ingest (Phase 2+): default loopback **`127.0.0.1:8787`**, crate `crates/ingest`.
Desktop host starts ingest on launch (`IngestConfig::load` + persist worker) and stops it on exit.

**LAN opt-in (P5-E1-T1 / ADR-005):** for a physical phone on the same Wi-Fi, restart Desktop with:
```bash
export BIOFOCUS_INGEST_LAN=1
# optional override: export BIOFOCUS_INGEST_BIND_HOST=0.0.0.0   # or a specific LAN IPv4
```
Without these knobs, bind stays `127.0.0.1` (Simulator / same-machine unchanged). `POST /v1/ingest` still requires Bearer (`BIOFOCUS_INGEST_TOKEN` / `~/.biofocus/pairing_token`).

**Base URL hint (P5-E1-T2):** after LAN opt-in, read bind mode + usable URLs from:
- `GET /v1/status` → `bind_mode` / `base_url_hints` (snake_case), or
- IPC `get_pairing_token` → `bindMode` / `baseUrlHints` / `ingestBaseUrl` (primary = first hint).

```bash
curl -s http://127.0.0.1:8787/v1/status | jq '{bind_mode, base_url_hints}'
# point companion at base_url_hints[0], e.g. http://192.168.x.x:8787
```
Loopback mode always reports `http://127.0.0.1:<port>`. Hints are derived on read/startup (no busy-loop).

**Companion LAN UI (P5-E2-T1 Done):** Desktop **Companion** section shows copyable primary base URL (`ingestBaseUrl` from `get_pairing_token`), bind-mode / LAN opt-in copy, and existing token Show / Copy / QR. When `bindMode=lan` but primary is still loopback, UI surfaces a calm fallback hint (`BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>`). No pairing busy-loop (load on mount + manual Reload).

**Pipeline (Phase 3 E1):** crate `crates/pipeline`.
- **Intake (T1):** `pipeline::accept_observations(&[Observation])` → `AcceptedBatch` (`AcceptedForProcessing`). Empty = **Ok**. Helpers: `accept_owned`, `accept_iter`.
- **Dedupe (T2):** `pipeline::dedupe_observations(&mut DedupeState, &[Observation])` → `DedupedBatch` (`Deduped`); wire `dedupe_accepted`. Rule: drop if same `id` **or** same `(provider_id, data_type, timestamp, payload JSON)` already seen (first wins; in-memory seen-set). SQLite rows never rewritten.
- **Normalize (T3):** `pipeline::normalize_observations(&[Observation])` → `NormalizedBatch` (`Normalized`); wire `normalize_deduped`. Known types → canonical payload/units (`heart_rate` / `hrv` / `context_window` / `keystrokes` / `calendar_event` / `browser_category` / `now_playing` / `git_activity` / `ambient_light`); unknown → pass-through; unparseable known → skip. SQLite rows never rewritten.
- **Quality chain helper:** `pipeline::run_quality_pipeline(batch, &mut DedupeState)` → accept → dedupe → normalize.
- **Feature Worker (T4):** `runtime::spawn_feature_worker(source, hook, config)` — idle-safe poll (`recv_timeout` ≥1s when empty); desktop `feature_host::{start,stop}_feature_host` with app lifecycle. Hook stub: `NoopFeatureHook` until Feature Engine (E2). Storage cursor: `ObservationRepository::list_after_created_cursor` (no new schema; launch at DB tip).

**Feature Engine (Phase 3 E2):** crate `crates/feature-engine`.
- **DAG skeleton (T1):** `FeatureEngine::register` + `FeatureEngine::run(&[Observation])` → `EngineOutput { features, signals }`. Nodes implement `FeatureNode` (`id` / `depends_on` / `compute`). Kahn topo; errors via `thiserror` (duplicate / unknown dep / cycle / node failed). Empty DAG/snapshot → `Ok` empty.
- **Focus catalog (T2):** `feature_engine::register_focus_v1(&mut engine)` — `ContextSwitchRate` → `FocusScore` (window 15m / step 1m; see `/docs/06-feature-catalog.md`).
- **Stress/Fatigue (T3):** `feature_engine::register_stress_v1(&mut engine)` — `StressIndex` + `FatigueIndex` (needs FocusScore already registered); contiguous StressIndex > 75 for > 5m → transient `Signal` `High_Stress` (`Severity::High`).
- **Calendar Features (P6-E3-T2):** `feature_engine::register_calendar_v1(&mut engine)` — `MeetingDensity` (busy overlap fraction 0–1) + `RecoveryBetweenMeetings` (mean free gap minutes) from `calendar_event` Observations.
- **RecoveryScore (P7-E3-T1):** `feature_engine::register_recovery_v1(&mut engine)` — short-term physiological recovery proxy from `hrv` (+ optional `heart_rate`); confidence (ADR-007) + explanation factors. Full catalog: `feature_engine::register_catalog_v1` = focus + stress + calendar + recovery + distraction + ambient + git + ambient light + wearable.
- **Alert level (E3-T1):** `feature_engine::map_alert_level(&EngineOutput) → AlertLevel` — Red if `High_Stress`; Yellow if latest StressIndex or FatigueIndex > 60; else Green (empty → Green).
- **IPC alert (E3-T2):** Desktop `get_status` includes `alertLevel` (`green`/`yellow`/`red`). Feature Worker hook runs catalog → map → shared `AlertState`. Idle/Ready/Error (`dbStatus`) unchanged. UI reads via IPC only.
- **Menubar alert UX (E3-T3):** Shell shows calm Steady/Elevated/High indicator (color + copy) from `alertLevel`; tray tooltip includes the label. Poll ~5s. QA: `?mockAlert=green|yellow|red`.
- **Feature snapshot IPC (P4-E1-T1):** `feature_engine::FeatureSnapshot` from last `EngineOutput`; desktop caches via `SnapshotState` + `invoke("get_feature_snapshot")` (pure read; **latest per featureId**). `get_status` stays lean. Contract: `docs/09-api.md`.
- **Feature confidence (P7-E1-T1 / ADR-007):** domain `Feature.confidence` ∈ `[0.0, 1.0]` = coverage × mean evidence `Observation.confidence`; catalog nodes compute it; snapshot IPC exposes `confidence`. No Feature SQLite schema.
- **Explanation factors (P7-E2-T1 / P7-E3-T1 / P10-E3-T1 / P12-E3-T1 / P13-E3-T1 / P16-E2-T1 / P17-E3-T1):** optional `Feature.factors` `{ id, label, share }` — calm input-share breakdown; includes wearable `ActivityBalance` / `EnergyScore` / `SleepDebt`; IPC omits key when empty. No ADR (additive optional field); no SQLite schema.
- **Dashboard shell (P4-E1-T2):** separate Tauri window `label: dashboard` (`index.html?view=dashboard`); Menubar **Open Dashboard** → `invoke("open_dashboard")` (show/focus; CloseRequested → hide). Calm loading/empty/error via snapshot IPC. QA mocks: `?view=dashboard&mockSnapshot=empty|ready|error` (see `apps/desktop/README.md`).
- **Recharts Feature series (P4-E1-T3 / P17-E3):** Dashboard `ChartSlot` — range picker `1h`/`8h`/`12h`/`1d`/`1w` → `invoke("get_feature_series")` recompute-on-read; LineChart for catalog scores (+ `ContextSwitchRate` secondary axis when present); Snapshot list stays on `get_feature_snapshot` (latest). Calm empty/loading. Dep: `recharts` in `apps/desktop`.
- **knowledge-engine skeleton (P4-E2-T1):** pluggable `InsightRule` + `KnowledgeEngine::evaluate(&[Feature], &[Signal]) → Result<Vec<Insight>>`; types `Insight` / `EvidenceRef` from `bio-spec`; empty/no-match → `Ok([])`.
- **Rule Insights v1 (P4-E2-T2):** `knowledge_engine::register_insights_v1` — `High_Stress` Signal + elevated `ContextSwitchRate` (≥1.0) rules with calm copy / `EvidenceRef`; host must register (empty engine still `Ok([])`). IPC/UI → T3.
- **Insights IPC + Dashboard list (P4-E2-T3):** Desktop `invoke("get_insights")` — evaluate-on-read over cached `FeatureSnapshot`; host `KnowledgeEngine::new()` + `register_insights_v1` at startup. Dashboard Insights list (evidence refs) + calm empty state; refresh on open + ~30s. Contract: `docs/09-api.md`.
- **Pattern Discovery baseline (P8-E2-T1 / ADR-008):** `focus_vs_recent_baseline_v1` — live `FocusScore` vs mean of ≤7 prior **UTC afternoon** windows (13:00–17:00 UTC; local-TZ afternoon deferred); emit when `|Δ| ≥ 10` and confidence ≥ 0.4. Host `pattern_host` loads Observations, recomputes via `feature_engine::baseline`, optional in-process memo (TTL + watermark) — not SQLite. Thin history → omit.
- **Pattern Insights UX (P8-E3-T1):** Same `get_insights` → Dashboard Insights list; calm category label for `pattern` / `focus` / `stress`; title/description as Core returns; empty/thin history stays calm. QA mocks: `?mockInsights=empty|ready|pattern|error` (see `apps/desktop/README.md`). UI ↛ SQLite.
- **Recommendations shape (P9-E1-T1 / ADR-009):** First-class `Recommendation` + `RecommendationRule` in `knowledge-engine` (evaluate-on-read after Insights); Evidence may cite Feature / Signal / Insight; **no** Recommendation SQLite store; thin `Insight.actionRecommendation` stays optional hint only. Engine → P9-E2; IPC `get_recommendations` → P9-E3. Contract sketch: `docs/09-api.md`.
- **Recommendations v1 engine (P9-E2-T1):** `bio-spec::{Recommendation, EvidenceRef::Insight}`; `register_recommendations_v1` + `focus_dip_pace_hint_v1` (Focus-below-baseline pattern Insight + FocusScore confidence ≥ 0.4 → calm pace hint); `KnowledgeEngine::evaluate_recommendations` / `evaluate_insights_and_recommendations`. IPC/UX → P9-E3.
- **Recommendations IPC / UX (P9-E3-T1):** Desktop `invoke("get_recommendations")` — host registers Recommendations alongside Insights; evaluate-on-read after Insights on Feature snapshot + pattern inputs. Dashboard **Suggestions** section (Insights-adjacent); calm empty state. QA mocks: `?mockRecommendations=empty|ready|pace|error`. UI ↛ SQLite. Contract: `docs/09-api.md`.
- **Plugin wave-1 (P10-E1-T1 / ADR-010):** Chosen source = **Browser categories** (`data_type: "browser_category"`; coarse labels only; opt-in `BIOFOCUS_BROWSER_CATEGORIES`; default off). Persist via existing `observations` store — **no** migration / plugin registry table. IDE/Git deferred. Collector → **P10-E2**; `DistractionScore` → **P10-E3**. Contracts: `docs/07-contracts.md` / `docs/08-plugin-sdk.md`.
- **Phase 12 ambient + packaging (P12-E1-T1 / ADR-012):** Primary = opt-in **Now Playing** Observations (`data_type: "now_playing"`; `BIOFOCUS_NOW_PLAYING`; default off; coarse `media_kind` + `is_playing` only). Existing `observations` store — **no** migration. Weather/light deferred. Secondary = signed-build / notarization / update **runbook** (E3 shipped); optional sync stance off by default (no sync product in P12). Collector = **P12-E2**; Feature `AmbientMediaShare` + packaging runbook = **P12-E3**. Contracts: `docs/07-contracts.md` / `docs/08-plugin-sdk.md`.
- **Now Playing collector (P12-E2-T1):** `NowPlayingPlugin` (`com.biofocus.macos.now_playing`) in `macos-collector`; Desktop `ingest_host` starts only when `BIOFOCUS_NOW_PLAYING=1`. Emit on change / poll ≥5s; `stop_stream` joins. System probe soft-fails idle (no titles). Validation: `bio_spec::validate_now_playing_payload`. Tests: `collector_integration` + bio-spec contracts. Feature companion: `AmbientMediaShare` (P12-E3 shipped).
- **AmbientMediaShare + packaging (P12-E3-T1):** `feature_engine::register_ambient_v1` / `register_catalog_v1` — share of window samples with `is_playing && media_kind ∈ {music,podcast,other}` (0–100); **omit** empty / only-`none` / only-`unknown`; ADR-007 confidence + optional kind factors. Pipeline normalizes `now_playing` (strips title/artist/album/lyrics/playlist ids). Catalog: `docs/06-feature-catalog.md` §1.9. Packaging companion: [`docs/18-packaging-runbook.md`](18-packaging-runbook.md) (signed `.app`/`.dmg`, notarization, update-channel stance; optional sync off by default; AGPLv3 Core stays open).
- **Plugin wave-2 (P13-E1-T1 / ADR-013):** Chosen source = **Git activity aggregates** (`data_type: "git_activity"`; coarse `activity_kind` + optional `event_count`; opt-in `BIOFOCUS_GIT_ACTIVITY`; default off). Existing `observations` store — **no** migration. IDE deferred (no additive privacy-safe session kind beyond `context_window` for v1). Weather/light + App Store packaging product deferred. Collector = **P13-E2**; Feature `GitActivityRate` = **P13-E3**. Contracts: `docs/07-contracts.md` / `docs/08-plugin-sdk.md`.
- **Git activity collector (P13-E2-T1):** `GitActivityPlugin` (`com.biofocus.macos.git`) in `macos-collector`; Desktop `ingest_host` starts only when `BIOFOCUS_GIT_ACTIVITY=1`. Emit on change / poll ≥5s; `stop_stream` joins. System probe soft-fails idle (no path-allowlist table in v1). Validation: `bio_spec::validate_git_activity_payload`. Pipeline normalizes `git_activity` (strips paths/remotes/branch/SHA/message/diff/author). Tests: `collector_integration` + bio-spec contracts. Feature companion: `GitActivityRate` (P13-E3).
- **GitActivityRate (P13-E3-T1):** `feature_engine::register_git_v1` / `register_catalog_v1` — sum of `event_count` (default 1) for `activity_kind ∈ {commit,checkout,sync,other}` → events per 15m window; **omit** empty / only-`idle` / only-`unknown`; ADR-007 confidence + kind factors. Distinct from `DistractionScore`. Catalog: `docs/06-feature-catalog.md` §1.10.
- **Git watched-roots allowlist (P14-E1-T1 / ADR-014):** v1 durable store = local file `~/.biofocus/git-watched-roots.toml` (absolute roots only). Observation payload unchanged (ADR-013). **No** SQLite allowlist table / **no** migration. Optional Settings/IPC / dogfood gate = **P14-E3**.
- **Git live probe + allowlist (P14-E2-T1):** `SystemGitActivityProbe` loads allowlist (file SoT; `BIOFOCUS_GIT_WATCHED_ROOTS` only if file absent); discovers nested repos under roots; emits coarse `git_activity` on HEAD/reflog/FETCH_HEAD/index change; empty allowlist → idle. Still opt-in `BIOFOCUS_GIT_ACTIVITY`. Feature `GitActivityRate` unchanged. Tests: empty allowlist no emit; fixture root commit → channel → persist; `stop_stream` freezes. Contracts: `docs/07-contracts.md` / `docs/08-plugin-sdk.md` / `docs/10-security.md`.
- **Git allowlist Settings + dogfood (P14-E3-T1):** Menubar **Git folders** — `invoke("get_git_watched_roots")` / `invoke("set_git_watched_roots")` read/write ADR-014 TOML (`BIOFOCUS_HOME` for tests). UI ↛ SQLite. No `GitActivityRate` rewrite. Browser QA: `?mockGitRoots=empty|ready|error`. Dogfood runbook: below § Git activity dogfood.
- **Phase 15 companion HRV + autonomy (P15-E1–E3 / ADR-016):** iOS Auto-sync — HealthKit HR + HRV SDNN → local queue → Desktop ingest; Core `normalize_hrv` / Features accept **rmssd_ms OR sdnn_ms**. ADR-015 ambient light resumed in **Phase 16**. Dogfood: below § Companion autonomy.
- **Phase 16 ambient light (ADR-015 / P16-E1–E2):** Opt-in `AmbientLightPlugin` (`com.biofocus.macos.ambient_light`) in `macos-collector`; Desktop `ingest_host` starts only when `BIOFOCUS_AMBIENT_LIGHT=1`. Emit on light-band change / poll ≥5s; `stop_stream` joins. System probe soft-fails idle (no camera / screen / geo / mic). Validation: `bio_spec::validate_ambient_light_payload`. Tests: `collector_integration` + bio-spec contracts. Feature: `AmbientLightShare` via `register_ambient_light_v1` / `register_catalog_v1` (P16-E2) — closed-set sample share 0–100; omit empty / only-`unknown`. Branch `phase/16-ambient-light`. **Phase 16 Done** (2026-08-11).
- **Phase 17 wearable + charts (ADR-017 · ADR-018) ✅ Done 2026-08-11:** Contracts (E1) · Companion emit (E2) · Chart path live (E3) — Desktop `invoke("get_feature_series")` recompute-on-read (ranges `1h`/`8h`/`12h`/`1d`/`1w`; default steps 60/300/300/900/3600); Snapshot = latest-per-id; catalog `register_wearable_v1` — `ActivityBalance` / `EnergyScore` / `SleepDebt` (omit when inputs absent; no clinical SpO2 Features). `FeatureEngine::run_with_step` for coarser series. Optional in-process series memo — **no** Feature-history SQLite. Branch `phase/17-wearable-charts` (cluster PR after freeze). Dogfood: § Companion autonomy (wearable depth); physical-device check recommended.
- **Phase 18 notification pressure (ADR-019) ✅ Done 2026-08-11:** Observation `data_type: "notification_event"` (coarse `count` + optional closed-set labels; **no** body/title content) → existing `observations` store; opt-in `BIOFOCUS_NOTIFICATION_EVENTS` default **off**; `NotificationPlugin` (`com.biofocus.macos.notifications`) in `macos-collector`; Desktop `ingest_host` arms only when env set. Soft-fail system probe idle; scripted probe for tests. Validation: `bio_spec::validate_notification_event_payload`; ingest `invalid_notification_event`; pipeline strips content keys. Catalog Feature **`NotificationPressure`** via `register_notification_v1` / `register_catalog_v1` — sum `count` → 0–100 (saturation 20/15m); omit empty; ADR-007 confidence. Branch `phase/18-notification-pressure` (cluster PR after freeze). **PM-GATE-POST-P18** ✅ chose Live NC OS mapping for Phase 19.
- **Phase 19 live NC OS mapping (ADR-020) ✅ Done 2026-08-11:** Live `SystemNotificationEventProbe` → **usernoted** SQLite allowlist (`delivered_date` + `app.identifier` only). Soft-fail when missing / TCC denied. ADR-019 payload unchanged; **no** Feature rewrite. Dogfood runbook: below § Notification events dogfood. Branch `phase/19-live-nc-mapping` (cluster PR after freeze). **PM-GATE-POST-P19** ✅ chose `CognitiveLoad` for Phase 20.
- **Phase 20 CognitiveLoad (ADR-021 ✅ / P20-E2 shipped):** Catalog Feature **`CognitiveLoad`** via `register_cognitive_v1` / `register_catalog_v1` — Feature-level composite of **MeetingDensity + ContextSwitchRate + NotificationPressure**; window **15m / 1m**; output 0–100; **renormalize** present inputs (omit only when none); ADR-007 expected slots = 3; explanation factors. Calm framing: “combined demand in this window”. **No** new Observation family / **no** migration; leaf Feature formulas untouched. Catalog: `docs/06-feature-catalog.md` §1.16. Branch `phase/20-cognitive-load` (cluster PR after freeze). Optional dogfood / Dashboard → **P20-E3**. Deferred: IDE · weather · App Store.
- **Browser categories collector (P10-E2-T1):** `BrowserCategoryPlugin` (`com.biofocus.macos.browser`) in `macos-collector`; Desktop `ingest_host` starts only when `BIOFOCUS_BROWSER_CATEGORIES=1`. Emit on change / poll ≥5s; `stop_stream` joins. OS probe: known browser → `unknown` + bundle (no URLs). Validation: `bio_spec::validate_browser_category_payload`. Tests: `collector_integration` + bio-spec contracts. `DistractionScore` → P10-E3.
- **DistractionScore (P10-E3-T1):** `feature_engine::register_distraction_v1` / `register_catalog_v1` — context fragmentation from `browser_category` (+ optional CSR); omit only-`unknown`/empty; ADR-007 confidence + explanation factors. Pipeline normalizes `browser_category` (strips url/title/href). Catalog: `docs/06-feature-catalog.md` §1.8.
- **report-engine builder (P4-E3-T1):** `report_engine::build_report(&[Feature], &[Insight]) → Result<ReportDocument>` — deterministic offline `markdown` + `llm_prompt` (no HTTP). Empty inputs → calm minimal report. LLM interpret-only; Feature math stays in `feature-engine`. Format: `docs/09-api.md` § report-engine.
- **report-engine prompt packs (P11-E2-T1 / ADR-011):** `build_report_with_pack(id, version, features, insights, recommendations) → Result<ReportDocument>` — in-process named/versioned packs (default `biofocus.default` @ `1`). No Feature math; no SQLite; no network from pack builder. See `docs/09-api.md` § Prompt packs.
- **Optional local LLM (P4-E3-T2):** **Off by default.** Set `BIOFOCUS_LOCAL_LLM=1` to opt in. Then `report_engine::interpret_report(&doc, &LocalLlmConfig::from_env()).await` POSTs only `ReportDocument::llm_prompt` to an OpenAI-compatible base URL (default `http://127.0.0.1:11434/v1` — local Ollama). Optional: `BIOFOCUS_LOCAL_LLM_BASE_URL`, `BIOFOCUS_LOCAL_LLM_MODEL` (default `llama3.2`), `BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS` (default `30`). HTTP timeout applies. **Never auto-called on app startup** — hosts must invoke only on explicit user action. Privacy: when disabled, no network. When enabled, the prompt text (offline report facts + interpret-only instructions) leaves the BioFocus process toward the configured base URL only — prefer localhost; a remote URL is the operator’s choice/responsibility. No Feature math in this path; no vendor cloud telemetry by default.
- **Report UX (P4-E3-T3 / P11-E3-T1):** Dashboard `ReportSlot` — calm **Local AI** provider status via `invoke("get_local_llm_status")` (config-only: `disabled` / `ready` / `error`; no HTTP probe). Explicit «Generate report» → `invoke("generate_report")`. Host assembles offline `build_report_with_pack("biofocus.default", "1", …)` from cached Feature snapshot + evaluate-on-read Insights + Recommendations; optional `interpret_report` only when `LocalLlmConfig::from_env().enabled`. Soft `llmStatus` / `llmError` on timeout or failure. Never called on Dashboard open or 30s Feature/Insights poll. Contract: `docs/09-api.md` § `get_local_llm_status` / `generate_report`. QA mocks: `?mockReport=…`, `?mockLlmStatus=disabled|ready|error`.
- **Life Events quick-log (P6-E2-T1):** Menubar Life events — `invoke("log_life_event")` / `invoke("list_recent_life_events")`. Host validates via `bio_spec` and inserts into the existing Observation store (`data_type: "life_event"`). No UI→SQLite; no life-event poll. Contract: `docs/09-api.md`. Browser QA: `?mockLifeEvents=…` (see `apps/desktop/README.md`).
```bash
cargo test -p feature-engine
cargo test -p knowledge-engine
cargo test -p report-engine
cargo test -p desktop
cd apps/desktop && pnpm exec tsc --noEmit
```

### Pipeline E2E (P3-E2-T4)

Integration suite (`docs/11-testing.md` §5): fixture / mock Observation stream → `pipeline::run_quality_pipeline` (accept → dedupe → normalize) → `feature_engine::register_catalog_v1` + `FeatureEngine::run` → asserted Features / Signals (in-memory; no SQLite schema, no UI).

```bash
# Full pipeline crate (unit + E2E)
cargo test -p pipeline

# E2E only
cargo test -p pipeline --test pipeline_e2e

# Feature Engine units (catalog) — optional companion
cargo test -p feature-engine
```

Coverage (`crates/pipeline/tests/pipeline_e2e.rs`):
- **Happy path** — fixture `tests/fixtures/e2e_happy_path.json` (alias payloads, duplicate id, unknown `custom.debug` pass-through) → normalized Observations → `ContextSwitchRate` / `FocusScore` / `StressIndex` / `FatigueIndex` + `High_Stress` Signal.
- **Edge** — empty batch → idle-friendly empty output; high RMSSD → StressIndex below threshold, no `High_Stress`.

**Active window collector (P2-E2-T1):** on macOS, Desktop also starts `ActiveWindowPlugin` (poll ≥1s → `context_window` Observations on the same channel → persist worker). No Accessibility permission. Payload: `bundle_id` + `app_name` only.

**Input aggregates (P2-E2-T2, opt-in):** `export BIOFOCUS_INPUT_AGGREGATES=1` then restart Desktop. Requires Accessibility. Emits `keystrokes` Observations (`count` / `window_secs` / `rate_per_min`) on the same channel. Default off.

**Local Calendar (P6-E3-T1, opt-in):** export a Calendar (or fixture) to `.ics`, then:
```bash
export BIOFOCUS_CALENDAR=1
export BIOFOCUS_CALENDAR_ICS="$HOME/Desktop/biofocus-dogfood.ics"
# restart Desktop
```
Emits `calendar_event` Observations (`uid` / `start` / `end` / `all_day` / `busy`) via rare poll ≥60s on the same channel → persist. No Google/Outlook OAuth. Titles/bodies are not stored or logged. Default off.

**Browser categories (P10-E2-T1, opt-in):**
```bash
export BIOFOCUS_BROWSER_CATEGORIES=1
# restart Desktop
```
Emits `browser_category` Observations (`category` + optional `browser_bundle_id`) on change / poll ≥5s → same channel → persist. **Never** stores URLs or page titles. OS probe v1 emits `unknown` for known browsers (no URL mapping). Default off.

**Now Playing ambient (P12-E2-T1, opt-in):**
```bash
export BIOFOCUS_NOW_PLAYING=1
# restart Desktop
```
Arms `NowPlayingPlugin` (poll ≥5s; emit on change). System probe soft-fails idle without titles; use scripted probes in tests. Default off. Catalog Feature: `AmbientMediaShare` (see packaging bullet above / `docs/06-feature-catalog.md` §1.9).

**Ambient light (P16-E1-T1, opt-in):**
```bash
export BIOFOCUS_AMBIENT_LIGHT=1
# restart Desktop
```
Arms `AmbientLightPlugin` (poll ≥5s; emit on light-band change). System probe soft-fails idle without camera/screen/geo/mic; use scripted probes in tests. Default off. Catalog Feature: `AmbientLightShare` (`docs/06-feature-catalog.md` §1.11).

### Collector test suite (P2-E2-T3)

```bash
# Unit + integration (mock probes; no Accessibility / NSWorkspace required)
cargo test -p macos-collector

# Integration only: emit Observation → channel → persist → SQLite + stop/idle
cargo test -p macos-collector --test collector_integration

# Broader Phase 2 ingest path (HTTP + persist), optional:
cargo test -p ingest
```

Integration coverage (`tests/collector_integration.rs`): `context_window`, `keystrokes`, synthetic / ICS `calendar_event`, mock `browser_category` / `now_playing` / `git_activity` / `ambient_light` land in storage via `spawn_persist_worker`; after `stop_stream`, probe call counts freeze (no busy-loop). Pause for collectors = `stop_stream` (plugin trait has no separate pause API).

### Companion sample path (P2-E3-T1)

```bash
cargo test -p companion

# Live Desktop ingest (token from file or env):
export BIOFOCUS_INGEST_TOKEN="$(cat ~/.biofocus/pairing_token)"
cargo run -p companion --bin biofocus-companion-sample -- 74
```

Rust client + CLI in `apps/companion`; iOS HealthKit stub under `apps/companion/ios/` (runnable target → **P5-E3-T1**). Same-machine / Simulator: `http://127.0.0.1:8787`. Physical phone on LAN: enable `BIOFOCUS_INGEST_LAN=1` on Desktop, then copy the **Base URL** from Companion UI (or `base_url_hints[0]` from `/v1/status` / pairing IPC).

### Pairing UX (P2-E3-T2 + P5-E2-T1)

In the Desktop shell (**Companion** section):

1. **Base URL** shows the primary ingest URL (LAN hint when available, else loopback) with **Copy URL** / **Reload**.
2. **Show** reveals the local pairing token (masked by default).
3. **Copy** puts the token on the clipboard for paste into companion / CLI / iOS stub.
4. **Show QR** displays a QR of the token (scan with the phone camera → copy text).

IPC: `invoke("get_pairing_token")` — see `docs/09-api.md`. No cloud account; UI does not open `~/.biofocus` itself.

**HTTP status (companion/debug):** `curl -s http://127.0.0.1:8787/v1/status` → `version` + `db_status` (no Observation payload). Menubar still uses IPC `get_status`.

**Pairing token**
- Default file: `~/.biofocus/pairing_token` (created on first `IngestConfig::load`)
- Override: `export BIOFOCUS_INGEST_TOKEN=…` (skips file; shell shows “env override”)
- Tests / custom root: `export BIOFOCUS_HOME=/tmp/biofocus-test` → `$BIOFOCUS_HOME/pairing_token`
- Share path: Desktop Companion UI (copy / QR) — do not commit the token
- Details: `docs/10-security.md`

### Companion autonomy dogfood (P15-E3 / ADR-016 · P17-E2 / ADR-018)

1. Desktop LAN: `export BIOFOCUS_INGEST_LAN=1` (and `BIOFOCUS_INGEST_BIND_HOST` if needed) → `pnpm tauri dev`.
2. Confirm `curl -s http://<lan-ip>:8787/v1/status` shows `"bind_mode":"lan"`.
3. Rebuild/run iOS companion from Xcode; paste Base URL + token; leave **Auto-sync** on.
4. Tap **Send latest wearable samples now** once (authorizes HealthKit reads for HR/HRV/steps/energy/sleep/SpO2).
5. Verify rows:
```bash
sqlite3 ~/.biofocus/data/biofocus_main.db \
  "SELECT data_type, COUNT(*) FROM observations
   WHERE provider_id='com.biofocus.applehealth' GROUP BY 1;"
```
6. Expect `heart_rate` always when present; `hrv` / `oxygen_saturation` / sleep may be sparse (Mi via Apple Health) — soft-omit, calm/expected; never invent SpO2. Chart ranges + Features shipped in P17-E3. Not clinical.

### Git activity dogfood (P14-E3-T1 / ADR-014)

Personal self-tracking only — not workplace monitoring.

1. **Enable collector:** `export BIOFOCUS_GIT_ACTIVITY=1` (default off).
2. **Allowlist (required for live emit):** non-empty absolute roots in `~/.biofocus/git-watched-roots.toml`, **or** use Menubar **Git folders** (`get_git_watched_roots` / `set_git_watched_roots`) to save the same file.
3. **Expect:** coarse `git_activity` Observations (`activity_kind` + optional `event_count`) → existing Feature `GitActivityRate`. Empty / missing allowlist → soft-fail idle (no whole-disk scan).
4. **CI / tests without a home file:** when the config file is **absent**, optional `BIOFOCUS_GIT_WATCHED_ROOTS=/abs/a:/abs/b` (colon or comma). File remains SoT when present (no env merge). Tests may set `BIOFOCUS_HOME=/tmp/…` so the TOML lands under `$BIOFOCUS_HOME/git-watched-roots.toml`.
5. **Privacy:** roots stay in config / Settings IPC only — **never** in Observation payloads or default logs. See `docs/08-plugin-sdk.md` §7.1 / `docs/10-security.md`.

Example file:

```toml
version = 1
roots = [
  "/Users/you/Developer/AI Project/BioFocus",
]
```

### Notification events dogfood (P19-E3-T1 / ADR-019 · ADR-020)

Personal self-tracking only — **not** workplace / employer notification monitoring.

1. **Enable collector:** `export BIOFOCUS_NOTIFICATION_EVENTS=1` (default **off**). Restart Desktop so `ingest_host` arms `NotificationPlugin`.
2. **Full Disk Access (often required):** macOS may block read of  
   `~/Library/Group Containers/group.com.apple.usernoted/db2/db`  
   without **System Settings → Privacy & Security → Full Disk Access** for BioFocus (or your terminal when dogfooding CLI). Without access the live probe **soft-fails idle** (no emit) — expected, not a crash.
3. **Privacy bar:** Observations are coarse `count` + optional closed-set `category` / `interruption_level` / `app_kind` only. The probe **never** `SELECT`s `record.data` / title / body / subtitle / message / userInfo / attachments. Do not grant Accessibility “for richer text” — that is out of scope.
4. **Expect live path:** after a new Notification Center delivery (and ≥5s poll / identity change), SQLite gains `data_type = 'notification_event'` rows from `com.biofocus.macos.notifications`. Existing catalog Feature **`NotificationPressure`** (0–100; omit when empty) consumes those rows — **no** formula rewrite in Phase 19.
5. **Verify Feature (no UI→SQLite):**
   - **Live:** Menubar / Dashboard via IPC `get_feature_snapshot` (and optional `get_feature_series`) — look for `NotificationPressure` after live emits exist.
   - **Counts only (debug):**  
     `sqlite3 ~/.biofocus/data/biofocus_main.db "SELECT COUNT(*) FROM observations WHERE data_type='notification_event';"`  
     (operator check; product UI must not open SQLite.)
   - **Fixture / CI path:** scripted probe / `write_fixture_nc_db` + `SystemNotificationEventProbe::with_db_path` in `macos-collector` tests — proves emit→channel→persist and Feature registration without FDA.
6. **Calm soft-fail note:** if the env is on but no `notification_event` rows appear, treat mapping as **unavailable** (FDA / missing DB / schema) — idle is correct. Re-check Full Disk Access; keep personal framing (“interruption intensity in this window”), never clinical / workplace copy.
7. Contracts: `docs/07-contracts.md` / `docs/08-plugin-sdk.md` / `docs/10-security.md`.

## CI

GitHub Actions (no CD): `.github/workflows/ci.yml`

| Job | Runner | When |
| :--- | :--- | :--- |
| **Detect paths** | ubuntu | Always (cheap); decides what to run |
| **rust-core** | ubuntu | `crates/**`, `apps/companion/**`, `Cargo.toml` / `Cargo.lock`, `rust-toolchain.toml`, this workflow |
| **desktop** | macos (×10 min) | PR: `apps/desktop/**` or workspace meta / this workflow. **Push to `main`:** also when rust paths change (merge gate for crate→desktop breaks) |
| **CI gate** | ubuntu | Always — success if heavy jobs succeeded or were skipped (docs/handoffs-only → green without burning minutes) |

- **rust-core:** `cargo check --workspace --exclude desktop`, `cargo test -p bio-spec -p runtime -p storage -p ingest -p pipeline -p feature-engine -p plugin-sdk -p macos-collector -p companion`
- **desktop:** `cargo test -p desktop`, `pnpm install` + `pnpm build`, UI↛DB boundary grep
- **Manual full suite:** Actions → CI → **Run workflow** (`workflow_dispatch`) — only while auto-CI is off, or anytime for a full check
- **Local before ship** (required while Actions auto-run is off): `cargo test` on touched crates; if IPC/Tauri/`apps/desktop` touched → `cargo test -p desktop` + `pnpm build`

### TEMP — Actions auto-run OFF (billing limit, 2026-08-06)

`push` / `pull_request` triggers are **commented out** in `ci.yml`. Workflow starts only via **Run workflow**.  
Until minutes reset: **local tests = gate**; agents may **commit + push straight to `main`** (no PR). When limit returns: restore triggers in `ci.yml` and resume classic PR flow below.

## Git workflow (related work → PR)

**Default (when Actions minutes available):** не пушить напрямую в `main`. Агенты: `.cursor/rules/06-git-agent-policy.mdc`.

### PR freeze (active until 2026-09-01)

До **2026-09-01 включительно** агенты **не открывают PR** и **не мержат в `main` через PR**.  
Работаем на feature-ветках; handoffs + локальные коммиты — ок. Классический flow ниже — **после** этой даты (или если пользователь явно снял freeze).

Классика (после freeze): **связанный код** → одна ветка → **мало коммитов** → **один PR** на код-кластер.  
Не коммит/PR на каждый handoff. **Docs / roadmap / canvas** — отдельно позже или в следующий code PR.

| Уровень | Правило |
| :--- | :--- |
| **Freeze** | Нет PR / merge в `main` до **2026-09-01** (если не сняли раньше). |
| **Ветка** | Кластер связанного кода: `phase/N-…`, `epic/P?-E?-…`, `feat/…`. |
| **Commit** | Когда код-единица готова (batch Task IDs ок). Handoffs на диске — не триггерят PR. |
| **Push / PR** | Только **после freeze** + substantive code vs `main` **и** (кластер готов **или** явный «PR»). Один PR на ветку. Squash preferred. |
| **Не PR** | до 2026-09-01; handoffs-only; roadmap/canvas-only; второй PR на тот же tip. |
| **Спринт** | Лучше мало содержательных PR, чем много пустых. |

```bash
git checkout main && git pull
git checkout -b epic/p3-e1-pipeline   # or feat/…

# … Dev → QA → PM on related tasks; handoffs on disk …
git add … && git commit -m "P3-E1: …"

git push -u origin HEAD
gh pr create --base main --title "…" --body "## Summary
- …
## Test plan
- [ ] CI green (rust-core + desktop)
"
# prefer: gh pr merge --squash (after CI Pass)
```

Naming: `phase/N-short-slug`, `epic/…`, `feat/…`.  
Phase 1 landed as direct push to `main` (foundation exception); do not repeat.

## Status

**Status (2026-08-05):** Phase 1–4 **Done** (Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24); Phase 4 E1–E3 on `phase/4-dashboard-ai`, cluster PR pending). **Phase 5 active** — Wearable dogfood; **P5-E1 + P5-E2 Done** (LAN bind ADR-005 + advertise hints + Companion LAN UI); Ready **P5-E3-T1** (runnable iOS HealthKit companion). Branch: `phase/5-wearable-dogfood`. Brief: `docs/handoffs/P5-E3-T1-pm-brief.md`. Platform vision (L1–L5, Personal Pattern Discovery, horizon P6–P12+) accepted in `/docs/00-vision.md`.  
**Git policy:** **PR freeze until 2026-09-01** (no agent PRs); then few **code** PRs; commit messages describe the change only — no personal device inventories.
