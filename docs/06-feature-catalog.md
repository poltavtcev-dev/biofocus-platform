# 06. Feature Catalog & Derived Metrics

## 1. Feature Specifications

### 1.1 `ContextSwitchRate`
- **Goal:** Частота переключений активного приложения в окне (вход для стабильности `FocusScore`).
- **Window:** 15 минут (sliding window, шаг 1 мин) — как у `FocusScore`.
- **Inputs:** `context_window` Observations (`bundle_id`).
- **Formula Strategy (v1):** Observations окна сортируются по времени; switch = смена `bundle_id` у соседних точек; value = `switches / 15` (на номинальную минуту окна). Пустой context → Feature для шага не эмитится.
- **Output:** Float (≥ 0).
- **Provenance:** Observation IDs `context_window` в окне.
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
- **DAG:** зависит от `ContextSwitchRate`; `register_focus_v1`.

### 1.3 `StressIndex`
- **Goal:** Оценка мгновенного физиологического напряжения (Baevsky Stress Index / HRV Analysis).
- **Window:** 15 минут (sliding window, шаг 1 мин).
- **Inputs (catalog):** `BioSnapshot.hrv_ms` (RMSSD / SDNN / pNN50).
- **Inputs (v1 implementation):** `hrv` Observations (RMSSD required for a sample; optional SDNN + `pnn50`).
- **Formula Strategy (v1):** Linear map RMSSD 100 @ ≤15 ms → 0 @ ≥70 ms; same map for SDNN when present; `pnn50` → `100 - pnn50`; average of present components. (Full Baevsky SI / RR-interval math — later.)
- **Output:** Float (0.0 — 100.0).
- **Provenance:** Observation IDs `hrv` в окне.
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
- **DAG:** зависит от `FocusScore`; `register_stress_v1` (after `register_focus_v1`) or `register_catalog_v1`.

## 2. Planned backlog (not sprint-Ready)

Accepted vision (`/docs/00-vision.md`): keep a catalog backlog; **implement only when Observation inputs exist**. Calm, non-clinical names (Global DoD). No burnout/clinical diagnosis claims.

| Working name | Intent | Likely inputs (later) | Earliest phase |
| :--- | :--- | :--- | :--- |
| `MeetingDensity` | Meeting load in window | Calendar Observations | P6 |
| `RecoveryBetweenMeetings` | Gap quality between meetings | Calendar | P6 |
| `RecoveryScore` | Short-term physiological recovery | HRV / sleep / HR | P7 |
| `EnergyScore` | Subjective energy proxy from bio + activity | HR, activity, sleep | P7 |
| `DeepWorkScore` | Sustained focus windows | FocusScore, CSR, idle | P7 |
| `AttentionStability` | Variance of focus / switches | FocusScore, CSR | P7 |
| `CognitiveLoad` | Combined demand proxy | MeetingDensity, CSR, notifications | P7–P8 |
| `SleepDebt` | Sleep shortfall vs baseline | Sleep Observations | P7 |
| `CircadianOffset` | Alignment of work vs chronotype proxy | sleep + activity timing | P8 |
| `NotificationPressure` | Interruption intensity | notification Observations | P10 |
| `DistractionScore` | Context fragmentation | CSR, browser categories | P10 |
| `TypingRhythm` | Input cadence stability | keystrokes | P7+ |
| `ActivityBalance` | Movement vs sedentary | steps / workout Life Events | P6–P7 |
| `SustainedLoadIndicator` | Prolonged high load (calm rename of “burnout risk”) | Stress, Fatigue, schedule | P8 |
| `DeepFocusLikelihood` | Probable deep-focus window (calm rename of “flow”) | Focus, CSR, calendar gaps | P8 |

**Rules:** each shipped Feature needs formula + units + dependencies + provenance in this doc; Feature-level **confidence** and **explanation factors** → Phase 7 (ADR if IPC grows).
