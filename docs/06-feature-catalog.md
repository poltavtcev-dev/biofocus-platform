# 06. Feature Catalog & Derived Metrics

## 1. Feature Specifications

### 1.1 `ContextSwitchRate`
- **Goal:** Частота переключений активного приложения в окне (вход для стабильности `FocusScore`).
- **Window:** 15 минут (sliding window, шаг 1 мин) — как у `FocusScore`.
- **Inputs:** `context_window` Observations (`bundle_id`).
- **Formula Strategy (v1):** Observations окна сортируются по времени; switch = смена `bundle_id` у соседних точек; value = `switches / 15` (на номинальную минуту окна). Пустой context → Feature для шага не эмитится.
- **Output:** Float (≥ 0).
- **Provenance:** Observation IDs `context_window` в окне.
- **Confidence (ADR-007):** single family; when emitted `confidence = mean(context Observation.confidence)`. Empty → omit.
- **DAG:** независимый узел; регистрируется через `feature_engine::register_focus_v1`.

### 1.2 `FocusScore`
- **Goal:** Оценка текущей глубины погружения в работу.
- **Window:** 15 минут (sliding window, шаг 1 мин).
- **Inputs (catalog):** `ContextSnapshot.keystrokes`, `ContextSnapshot.app_category`, `BioSnapshot.hrv_ms`.
- **Inputs (v1 implementation):** `keystrokes` (`rate_per_min`), upstream `ContextSwitchRate` (вместо taxonomy `app_category`), `hrv` (RMSSD ms).
- **Formula Strategy:** Высокая стабильная скорость ввода + целевая категория приложения + нормализованный HRV без резких аномалий.
- **Formula Strategy (v1):** Weighted (renormalized if missing): typing `mean(rate_per_min)/200*100` (0.40) + stability from CSR `100 - rate*50` (0.35) + HRV comfort peak 100 @ 45 ms RMSSD (0.25). Output clamped 0–100.
- **Output:** Float (0.0 — 100.0).
- **Provenance:** union Observation IDs keystrokes / hrv / context в окне.
- **Confidence (ADR-007):** expected slots = 3 (typing / stability / HRV); `confidence = coverage × mean(evidence Observation.confidence)`. Thin windows (missing HRV/context) → lower confidence; empty → omit Feature.
- **Explanation factors (P7-E2):** when emitted, factors for present components — `typing` (“Typing activity”), `stability` (“App stability”), `hrv` (“Heart-rate variability”); `share = catalog_weight / sum(present weights)` (shares sum to 1.0). Calm input composition only — not clinical. Empty window → omit Feature (no empty-factors-only emit).
- **DAG:** зависит от `ContextSwitchRate`; `register_focus_v1`.

### 1.3 `StressIndex`
- **Goal:** Оценка мгновенного физиологического напряжения (Baevsky Stress Index / HRV Analysis).
- **Window:** 15 минут (sliding window, шаг 1 мин).
- **Inputs (catalog):** `BioSnapshot.hrv_ms` (RMSSD / SDNN / pNN50).
- **Inputs (v1 implementation):** `hrv` Observations (RMSSD required for a sample; optional SDNN + `pnn50`).
- **Formula Strategy (v1):** Linear map RMSSD 100 @ ≤15 ms → 0 @ ≥70 ms; same map for SDNN when present; `pnn50` → `100 - pnn50`; average of present components. (Full Baevsky SI / RR-interval math — later.)
- **Output:** Float (0.0 — 100.0).
- **Provenance:** Observation IDs `hrv` в окне.
- **Confidence (ADR-007):** single family (HRV); when emitted `confidence = mean(hrv Observation.confidence)`. Empty HRV → omit.
- **Trigger Threshold:** contiguous minute samples all > 75.0 with span `(last_end - first_end) > 300` s → transient `Signal` type `High_Stress`, `Severity::High` (span == 300 s does **not** emit).
- **DAG:** `feature_engine::register_stress_v1` / `register_catalog_v1`.

### 1.4 `FatigueIndex`
- **Goal:** Оценка накопленной за день усталости.
- **Window:** 15 минут (sliding window, шаг 1 мин) — как у Focus/Stress.
- **Inputs (catalog):** `FocusScore` history, total active hours, baseline HR shift.
- **Inputs (v1 implementation):** upstream `FocusScore`; active minutes since snapshot min timestamp vs 8h day (not calendar midnight); `heart_rate` rise vs early-15m baseline.
- **Formula Strategy (v1):** Weighted (renormalized if missing): `100 - FocusScore` (0.50) + active-minutes fraction of 8h (0.30) + HR rise / 20 bpm (0.20). Output clamped 0–100.
- **Output:** Float (0.0 — 100.0).
- **Provenance:** keystrokes / HR / context Observation IDs in window.
- **Confidence (ADR-007):** expected slots = 3 (Focus / active / HR); `coverage × mean(evidence Observation.confidence)` (Focus-only windows use upstream Feature.confidence as evidence mean).
- **DAG:** зависит от `FocusScore`; `register_stress_v1` (after `register_focus_v1`) or `register_catalog_v1`.

### 1.5 `MeetingDensity`
- **Goal:** Доля окна, занятая busy-встречами (спокойная метрика нагрузки расписания; не клинический диагноз).
- **Window:** 15 минут (sliding window, шаг 1 мин) — как у Focus/Stress.
- **Inputs:** `calendar_event` Observations (`uid` / `start` / `end`; optional `busy` / `all_day`). Titles не требуются.
- **Formula Strategy (v1):** Timed events with `busy != false` (missing `busy` ⇒ busy); `all_day == true` skipped. Merge overlapping busy intervals clipped to the window; value = `merged_overlap_secs / 900`, clamped to `[0.0, 1.0]`. Malformed payloads skipped (partial calendars OK).
- **Output:** Float (0.0 — 1.0), fraction of window booked.
- **Units:** dimensionless fraction (1.0 = fully booked).
- **Provenance:** Observation IDs of busy meetings overlapping the window.
- **Confidence (ADR-007):** single family (calendar); when emitted `confidence = mean(meeting Observation.confidence)`.
- **DAG:** независимый узел; `feature_engine::register_calendar_v1` / `register_catalog_v1`.

### 1.6 `RecoveryBetweenMeetings`
- **Goal:** Средний свободный промежуток между consecutive busy-встречами (качество пауз в расписании; не физиологический recovery score).
- **Window:** 15 минут (sliding window, шаг 1 мин) — как у Focus/Stress.
- **Inputs:** те же busy `calendar_event` meetings, что и у `MeetingDensity`.
- **Formula Strategy (v1):** Sort busy meetings by `start`; gap = `(prev.end → next.start)` when positive. Mean gap length in **minutes** for gaps that intersect the Feature window. Back-to-back / overlapping → no gap. No intersecting gaps → Feature for that step is not emitted.
- **Output:** Float (≥ 0.0), mean gap minutes.
- **Units:** minutes.
- **Provenance:** Observation IDs of meetings bounding counted gaps.
- **Confidence (ADR-007):** single family; when emitted `confidence = mean(bounding meeting Observation.confidence)`.
- **DAG:** независимый узел (не зависит от `MeetingDensity`); `register_calendar_v1` / `register_catalog_v1`.

### 1.7 `RecoveryScore`
- **Goal:** Short-term physiological recovery proxy from recent HRV (and optional heart rate). Calm data-quality metric — **not** a clinical recovery diagnosis; no sleep required for v1.
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus/Stress.
- **Inputs:** `hrv` Observations (`rmssd_ms` required to emit); optional `heart_rate` (`bpm`). Sleep Observations out of scope for v1.
- **Formula Strategy (v1):** Weighted (renormalized if HR missing): HRV recovery map RMSSD 0 @ ≤15 ms → 100 @ ≥70 ms (0.70; inverse anchors of `StressIndex` RMSSD map) + HR calmness vs early-15m baseline BPM `100 - clamp((mean_bpm - baseline) / 20 * 100, 0, 100)` (0.30). Output clamped 0–100. HR alone (no usable HRV) → omit Feature.
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless score (higher ≈ more recovered proxy in-window).
- **Provenance:** Observation IDs of `hrv` / `heart_rate` in the window.
- **Confidence (ADR-007):** expected slots = 2 (HRV / HR); `confidence = coverage × mean(evidence Observation.confidence)`. Thin (HRV-only) → lower confidence; empty / no HRV → omit.
- **Explanation factors (P7-E3):** when emitted, factors for present components — `hrv` (“Heart-rate variability”), `heart_rate` (“Heart rate”); `share = catalog_weight / sum(present weights)` (shares sum to 1.0). Calm input composition only.
- **DAG:** независимый узел; `feature_engine::register_recovery_v1` / `register_catalog_v1`. Distinct from schedule `RecoveryBetweenMeetings`.

### 1.8 `DistractionScore`
- **Goal:** Calm proxy for **context fragmentation** from in-browser category mix (and optional app switching). Personal observation of category churn / mix — **not** a clinical ADHD diagnosis and **not** a “you are distracted” judgment.
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus/CSR.
- **Inputs (required):** `browser_category` Observations with ≥1 **non-`unknown`** coarse label in the window (`work` / `communication` / `entertainment` / `reference` / `shopping`).
- **Inputs (optional):** upstream `ContextSwitchRate` for the same window — documented as improving the signal when app switching co-occurs with browser fragmentation (CSR alone is insufficient for in-browser mix).
- **Formula Strategy (v1):** Weighted (renormalized if CSR absent):
  - **Category mix (0.55):** equal mean of label weights — `work` 10, `reference` 20, `communication` 45, `shopping` 75, `entertainment` 90 (`unknown` excluded from mix).
  - **Category churn (0.25):** consecutive label changes (incl. `unknown`) → `min(100, switches × 25)`.
  - **CSR (0.20, optional):** `min(100, ContextSwitchRate × 50)`.
  - Output clamped 0–100. Higher ≈ more fragmented browser/context mix in-window.
- **Omit policy:** empty window, no `browser_category`, or **only-`unknown`** thin windows → **omit** Feature (OS probe without URL mapping often emits `unknown` — Feature waits for closed-set labels / richer mapping).
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless score.
- **Provenance:** Observation IDs of `browser_category` in the window.
- **Confidence (ADR-007):** expected slots = 2 (browser / CSR); `confidence = coverage × mean(evidence Observation.confidence)`. Browser-only → lower confidence (coverage 0.5).
- **Explanation factors:** when emitted — `browser_mix` (“Browser category mix”), `category_churn` (“Category changes”), optional `app_switches` (“App switching”); shares sum to 1.0.
- **DAG:** depends on `ContextSwitchRate`; `feature_engine::register_distraction_v1` / `register_catalog_v1` (after `register_focus_v1`).

### 1.9 `AmbientMediaShare`
- **Goal:** Calm share of the window with **active personal media** playing. Ambient context only — “media present during this window” — **not** “you listen too much” and **not** a clinical / attention diagnosis.
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR / DistractionScore.
- **Inputs (required):** `now_playing` Observations (`media_kind` + `is_playing`) with ≥1 **closed-set** kind in the window: `music` / `podcast` / `other` (playing or paused).
- **Formula Strategy (v1):** Sample share over the window:
  - Count all `now_playing` samples in the window as denominator.
  - Numerator = samples where `is_playing == true` **and** `media_kind ∈ {music, podcast, other}`.
  - Value = `100 × numerator / denominator`, clamped 0–100.
  - Paused closed-set samples (e.g. music with `is_playing: false`) count in the denominator but not the numerator → may emit **0**.
- **Omit policy:** empty window, no `now_playing`, or **only-`none` / only-`unknown`** (no closed-set kinds) → **omit** Feature (soft-fail OS probe often emits `unknown` / idle `none` — Feature waits for closed-set kinds).
- **Output:** Float (0.0 — 100.0).
- **Units:** percent of window samples with active closed-set media.
- **Provenance:** Observation IDs of `now_playing` in the window.
- **Confidence (ADR-007):** single family (`now_playing`); when emitted `confidence = mean(evidence Observation.confidence)`.
- **Explanation factors:** when ≥1 active-playing sample — per-kind shares among playing samples: `music` (“Music playing”), `podcast` (“Podcast playing”), `other` (“Other media playing”); shares sum to 1.0. Paused-only closed-set → value 0, empty factors.
- **DAG:** независимый узел; `feature_engine::register_ambient_v1` / `register_catalog_v1`.

## 2. Planned backlog (not sprint-Ready)

Accepted vision (`/docs/00-vision.md`): keep a catalog backlog; **implement only when Observation inputs exist**. Calm, non-clinical names (Global DoD). No burnout/clinical diagnosis claims.

| Working name | Intent | Likely inputs (later) | Earliest phase |
| :--- | :--- | :--- | :--- |
| `EnergyScore` | Subjective energy proxy from bio + activity | HR, activity, sleep | P7 |
| `DeepWorkScore` | Sustained focus windows | FocusScore, CSR, idle | P7 |
| `AttentionStability` | Variance of focus / switches | FocusScore, CSR | P7 |
| `CognitiveLoad` | Combined demand proxy | MeetingDensity, CSR, notifications | P7–P8 |
| `SleepDebt` | Sleep shortfall vs baseline | Sleep Observations | P7 |
| `CircadianOffset` | Alignment of work vs chronotype proxy | sleep + activity timing | P8 |
| `NotificationPressure` | Interruption intensity | notification Observations | P10 (deferred — not ADR-010 wave-1) |
| `GitActivityRate` | Personal VCS cadence in-window (coarse git events) | `git_activity` Observations (ADR-013) | P13-E3 |
| `TypingRhythm` | Input cadence stability | keystrokes | P7+ |
| `ActivityBalance` | Movement vs sedentary | steps / workout Life Events | P6–P7 |
| `SustainedLoadIndicator` | Prolonged high load (calm rename of “burnout risk”) | Stress, Fatigue, schedule | P8 |
| `DeepFocusLikelihood` | Probable deep-focus window (calm rename of “flow”) | Focus, CSR, calendar gaps | P8 |

**Rules:** each shipped Feature needs formula + units + dependencies + provenance + **confidence** (ADR-007) in this doc; **explanation factors** where catalog emits them (P7-E2 — `FocusScore`; P7-E3 — `RecoveryScore`; P10-E3 — `DistractionScore`; P12-E3 — `AmbientMediaShare`; others may omit until wired).
