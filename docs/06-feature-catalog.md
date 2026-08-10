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
| `DistractionScore` | Context fragmentation (calm; not clinical) | `browser_category` Observations (+ optional CSR) — **ADR-010 wave-1 → P10-E3** | P10 |
| `TypingRhythm` | Input cadence stability | keystrokes | P7+ |
| `ActivityBalance` | Movement vs sedentary | steps / workout Life Events | P6–P7 |
| `SustainedLoadIndicator` | Prolonged high load (calm rename of “burnout risk”) | Stress, Fatigue, schedule | P8 |
| `DeepFocusLikelihood` | Probable deep-focus window (calm rename of “flow”) | Focus, CSR, calendar gaps | P8 |

**Rules:** each shipped Feature needs formula + units + dependencies + provenance + **confidence** (ADR-007) in this doc; **explanation factors** where catalog emits them (P7-E2 — `FocusScore`; P7-E3 — `RecoveryScore`; others may omit until wired).
