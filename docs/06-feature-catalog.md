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
- **Inputs:** `hrv` Observations (`rmssd_ms` preferred; **`sdnn_ms` accepted as HRV-proxy when RMSSD absent — ADR-016**); optional `heart_rate` (`bpm`). Sleep Observations out of scope for v1.
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

### 1.10 `GitActivityRate`
- **Goal:** Calm personal **version-control cadence** in-window from coarse git activity. Self-tracking only — “VCS cadence in this window” — **not** “you commit too little”, **not** workplace / manager monitoring, and **not** a clinical claim. **Distinct from** `DistractionScore` (does not merge or redefine Browser math).
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR / DistractionScore / AmbientMediaShare.
- **Inputs (required):** `git_activity` Observations (`activity_kind` + optional `event_count`) with ≥1 **countable** kind in the window: `commit` / `checkout` / `sync` / `other`.
- **Formula Strategy (v1):** Sum of event counts over the window:
  - For each countable Observation, take `event_count` if present and ≥ 1; otherwise treat as **1**.
  - Ignore `idle` / `unknown` for the sum (they do not inflate the rate).
  - Value = sum → **events per 15-minute window** (scalar ≥ 0).
- **Omit policy:** empty window, no `git_activity`, or **only-`idle` / only-`unknown`** (no countable kinds) → **omit** Feature (soft-fail OS probe often emits nothing / unknown — Feature waits for countable kinds; works on scripted / HTTP-ingest fixtures).
- **Output:** Float (≥ 0.0).
- **Units:** coarse git events per 15-minute window.
- **Provenance:** Observation IDs of **countable** `git_activity` rows in the window.
- **Confidence (ADR-007):** single family (`git_activity`); when emitted `confidence = mean(evidence Observation.confidence)` over countable rows.
- **Explanation factors:** when ≥1 countable event — per-kind shares of summed counts: `commit` (“Commits”), `checkout` (“Checkouts”), `sync` (“Sync events”), `other` (“Other VCS events”); shares sum to 1.0.
- **DAG:** независимый узел; `feature_engine::register_git_v1` / `register_catalog_v1`.

### 1.11 `AmbientLightShare`
- **Goal:** Calm share / band context of **ambient light** in-window. Environment context only — “light context during this window” — **not** clinical lighting advice and **not** “bad lighting harms you”. **Distinct from** `AmbientMediaShare` (media presence) and `GitActivityRate` (VCS cadence).
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR / AmbientMediaShare / GitActivityRate.
- **Inputs (required):** `ambient_light` Observations (`light_kind` + optional `level`) with ≥1 **closed-set** kind in the window: `dark` / `dim` / `moderate` / `bright`.
- **Formula Strategy (v1):** Sample share over the window:
  - Count all `ambient_light` samples in the window as denominator.
  - Numerator = samples where `light_kind ∈ {dark, dim, moderate, bright}`.
  - Value = `100 × numerator / denominator`, clamped 0–100.
  - Optional `level` (0–100) is **accepted** on Observations but **not** used in v1 value / factors (stays coarse band-share — no lux / camera).
- **Omit policy:** empty window, no `ambient_light`, or **only-`unknown`** (no closed-set kinds) → **omit** Feature (soft-fail OS probe often emits nothing — Feature works on scripted / HTTP-ingest fixtures).
- **Output:** Float (0.0 — 100.0).
- **Units:** percent of window samples in closed-set light bands.
- **Provenance:** Observation IDs of `ambient_light` in the window.
- **Confidence (ADR-007):** single family (`ambient_light`); when emitted `confidence = mean(evidence Observation.confidence)` over in-window `ambient_light` samples.
- **Explanation factors:** when ≥1 closed-set sample — per-kind shares among closed-set samples: `dark` (“Dark band”), `dim` (“Dim band”), `moderate` (“Moderate band”), `bright` (“Bright band”); shares sum to 1.0.
- **DAG:** независимый узел; `feature_engine::register_ambient_light_v1` / `register_catalog_v1`.

### 1.12 `ActivityBalance` (P17-E3 / ADR-018)
- **Goal:** Calm movement vs sedentary proxy from step counts (+ optional workout Life Event). Personal activity context — **not** fitness coaching or clinical advice.
- **Window:** 15 minutes (sliding; step from engine context — default 1m; chart series may coarsen per ADR-018).
- **Inputs (required):** `step_count` Observations (`count`; optional `window_secs`).
- **Inputs (optional):** `life_event` with `kind == "workout"`.
- **Formula Strategy (v1):** Sum `count` in-window → linear map 0 @ 0 steps → 100 @ ≥750 steps. With workout present: floor steps component at 70, then blend 85% steps / 15% workout-present (100), renormalized. Workout alone (no steps) → **omit**.
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless score (higher ≈ more movement proxy in-window).
- **Provenance:** Observation IDs of `step_count` / workout `life_event` used.
- **Confidence (ADR-007):** expected slots = 2 (steps / workout); `coverage × mean(evidence Observation.confidence)`.
- **Explanation factors:** `step_count` (“Steps”), optional `workout` (“Workout”); shares sum to 1.0.
- **DAG:** независимый узел; `feature_engine::register_wearable_v1` / `register_catalog_v1`.

### 1.13 `EnergyScore` (P17-E3 / ADR-018)
- **Goal:** Calm subjective energy proxy from active energy + optional HR + recent rest. **Not** a clinical fatigue / energy diagnosis. SpO2 unused (sparse / non-clinical).
- **Window:** 15 minutes (sliding; context step as above). Sleep rest uses a **24h lookback** ending at the window end.
- **Inputs (at least one):** `active_energy` (`kcal`); optional `heart_rate` (`bpm`); optional qualifying `sleep_interval` rest (`asleep` / `in_bed` / missing stage).
- **Formula Strategy (v1):** Weighted renormalized over present families — active energy 0.45 (0→0, ≥80 kcal→100); sleep sufficiency 0.35 (`100 × clamp(rest_secs / 8h, 0, 1)`); HR calmness vs early baseline 0.20 (`100 - clamp((mean_bpm - baseline) / 25 * 100, 0, 100)`).
- **Omit:** none of the three families present.
- **Output:** Float (0.0 — 100.0).
- **Provenance:** Observation IDs of contributing families.
- **Confidence (ADR-007):** expected slots = 3; coverage × mean evidence confidence.
- **Explanation factors:** present of `active_energy` / `sleep_interval` / `heart_rate`; shares sum to 1.0.
- **DAG:** независимый узел; `register_wearable_v1` / `register_catalog_v1`.

### 1.14 `SleepDebt` (P17-E3 / ADR-018)
- **Goal:** Calm shortfall vs an 8h personal rest target from `sleep_interval` overlap in the last 24h. **Not** a sleep diagnosis; no SpO2.
- **Window:** Feature `timeWindow` remains 15m / context step; rest overlap measured on `[end−24h, end]`.
- **Inputs:** `sleep_interval` with stage ∈ {`asleep`, `in_bed`} or missing stage. `awake` / `unknown` do not add rest.
- **Formula Strategy (v1):** `value = 100 × clamp((target − rest) / target, 0, 1)` with `target = 8h`. 0 = met/exceeded; 100 = no qualifying rest.
- **Omit:** no overlapping qualifying intervals in the lookback.
- **Output:** Float (0.0 — 100.0).
- **Provenance:** contributing `sleep_interval` Observation IDs.
- **Confidence (ADR-007):** single family; mean evidence Observation confidence.
- **Explanation factors:** `rest` / `shortfall` renormalized shares.
- **DAG:** независимый узел; `register_wearable_v1` / `register_catalog_v1`.

### 1.15 `NotificationPressure` (P18-E3 / ADR-019 — shipped)
- **Goal:** Calm **interruption intensity** proxy from notification cadence — personal observation of how often alerts arrived in a window. “Interruption intensity in this window” — **not** clinical ADHD / anxiety diagnosis; **not** workplace productivity scoring; **not** “you are overloaded” / “you should mute everything.”
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR / AmbientLightShare; series may coarsen.
- **Inputs (required):** `notification_event` Observations with usable `count` ≥ 1 (`count` required; optional closed-set `category` / `interruption_level` / `app_kind`). **No** body / title / message content (Observation contract forbids storing them).
- **Formula Strategy (v1):** Sum `count` over usable in-window events → intensity map:
  - `value = clamp(100 × sum / 20, 0, 100)` with saturation **20** deliveries in the 15m window → 100.
  - Optional explanation factors: prefer `category` count-shares; else `interruption_level`; else `app_kind` when present (shares sum to 1.0). Count-only windows emit without factors.
- **Omit policy:** empty window / no usable `notification_event` → **omit** Feature (soft-fail OS probe often emits nothing — Feature works on scripted / HTTP-ingest fixtures).
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless interruption-intensity score.
- **Provenance:** Observation IDs of usable `notification_event` in the window.
- **Confidence (ADR-007):** single family (`notification_event`); when emitted `confidence = mean(evidence Observation.confidence)`.
- **Explanation factors:** optional closed-set label shares as above (calm labels only).
- **DAG:** независимый узел; `feature_engine::register_notification_v1` / `register_catalog_v1`.
- **Privacy:** Feature must never surface notification body/title text.

### 1.16 `CognitiveLoad` (P20-E2 / ADR-021 — shipped)

- **Goal:** Calm **combined demand** proxy for a window from schedule density + app switching + interruption intensity. “Combined demand in this window” — **not** clinical cognitive overload / ADHD / burnout diagnosis; **not** workplace productivity scoring; **not** “you are overloaded.”
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR / NotificationPressure; series may coarsen.
- **Inputs (Feature-level):** upstream **`MeetingDensity`**, **`ContextSwitchRate`**, **`NotificationPressure`** for the same window (ADR-021). **Not** a raw Observation mix in v1.
- **Formula Strategy (v1):** Normalize present inputs to 0–100 (`MeetingDensity × 100`; `clamp(CSR × 50, 0, 100)`; `NotificationPressure` as-is). Equal weights (⅓ each). **Missing-input policy:** if **none** present → **omit**; if **one or more** present → **renormalize** weights over present components (not omit-unless-all-three). Output clamped 0–100.
- **Omit policy:** empty step (no upstream Features) → omit. Partial windows emit with lower ADR-007 confidence.
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless combined-demand score.
- **Provenance:** union of Observation IDs from present upstream Features.
- **Confidence (ADR-007):** expected slots = 3; `confidence = coverage × mean(upstream Feature.confidence)`.
- **Explanation factors:** when emitted — `meeting` (“Schedule demand”), `switches` (“App switching”), `notifications` (“Interruption intensity”); `share = catalog_weight / sum(present weights)` (shares sum to 1.0). Calm composition only.
- **DAG:** depends on MeetingDensity + ContextSwitchRate + NotificationPressure; `feature_engine::register_cognitive_v1` / `register_catalog_v1` (after calendar / focus / notification nodes).
- **Schema:** **no** new Observation `data_type`; **no** migration; **do not** rewrite leaf Feature formulas.

### 1.17 `DeepWorkScore` (P21-E2 / ADR-022 — shipped)

- **Goal:** Calm **sustained-focus** intensity for a window from Focus depth + low app-switching. “Sustained focus in this window” — **not** clinical flow state / ADHD / burnout diagnosis; **not** workplace productivity scoring; **not** “you are in flow.”
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR; series may coarsen.
- **Inputs (Feature-level):** upstream **`FocusScore`** (**required**) + **`ContextSwitchRate`** (**optional** stability term) for the same window (ADR-022). Backlog “idle” **dropped** for v1. **Not** a raw Observation mix; **not** a parallel FocusScore.
- **Formula Strategy (v1):** `focus = FocusScore`; `stability = clamp(100 - CSR × 50, 0, 100)`. Weights Focus 0.60 / stability 0.40. **Missing-input policy:** omit without FocusScore; if Focus present and CSR absent → **renormalize** (Focus-only). Output clamped 0–100.
- **Omit policy:** no FocusScore for the step → omit (windows driven from FocusScore ends — CSR-only cannot emit). Focus-only windows emit with lower ADR-007 confidence.
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless sustained-focus score.
- **Provenance:** union of Observation IDs from present upstream Features.
- **Confidence (ADR-007):** expected slots = 2; `confidence = coverage × mean(upstream Feature.confidence)`.
- **Explanation factors:** when emitted — `focus` (“Focus depth”), `stability` (“App stability”) when CSR present; `share = catalog_weight / sum(present weights)` (shares sum to 1.0). Calm composition only.
- **DAG:** depends on FocusScore + ContextSwitchRate; `feature_engine::register_deep_work_v1` / `register_catalog_v1` (after focus nodes).
- **Schema:** **no** new Observation `data_type`; **no** migration; **do not** rewrite leaf Feature formulas.

### 1.18 `AttentionStability` (P22-E2 / ADR-023 — shipped)

- **Goal:** Calm **focus stability** proxy for a window from Focus consistency (low in-window Focus range) + low app-switching. “Focus stability in this window” — **not** ADHD / “you can’t focus” / burnout diagnosis; **not** workplace productivity scoring; **not** DeepWorkScore “sustained focus” intensity.
- **Window:** 15 minutes (sliding window, шаг 1 мин) — как у Focus / CSR; series may coarsen.
- **Inputs (Feature-level):** upstream **`FocusScore`** (**required**) + **`ContextSwitchRate`** (**optional**) for the same window (ADR-023). **Not** a raw Observation mix; **not** a parallel FocusScore; **do not** redefine DeepWorkScore.
- **Formula Strategy (v1):** `focus_samples` = FocusScore scalars whose Feature ends lie in `[window_start, window_end]`. If ≥2 samples: `focus_stability = clamp(100 - (max−min), 0, 100)`; if exactly one: `focus_stability = 100` (no swing — **not** Focus-level intensity). `switch_stability = clamp(100 - CSR × 50, 0, 100)`. Weights focus_stability 0.50 / switch_stability 0.50. **Missing-input policy:** omit without FocusScore; if Focus present and CSR absent → **renormalize** (Focus-stability only). Output clamped 0–100.
- **Omit policy:** no FocusScore for the step → omit (windows driven from FocusScore ends — CSR-only cannot emit). Focus-only windows emit with lower ADR-007 confidence.
- **Output:** Float (0.0 — 100.0).
- **Units:** dimensionless focus-stability score.
- **Provenance:** union of Observation IDs from present upstream Features (all in-window Focus samples + CSR when present).
- **Confidence (ADR-007):** expected slots = 2; `confidence = coverage × mean(upstream Feature.confidence)` (Focus-slot conf = mean of in-window Focus samples).
- **Explanation factors:** when emitted — `focus_stability` (“Focus consistency”), `switch_stability` (“Switch steadiness”) when CSR present; `share = catalog_weight / sum(present weights)` (shares sum to 1.0). Calm composition only.
- **DAG:** depends on FocusScore + ContextSwitchRate; `feature_engine::register_attention_stability_v1` / `register_catalog_v1` (after focus / DeepWork nodes).
- **Schema:** **no** new Observation `data_type`; **no** migration; **do not** rewrite leaf / DeepWorkScore formulas.

## 2. Planned backlog (not sprint-Ready)

Accepted vision (`/docs/00-vision.md`): keep a catalog backlog; **implement only when Observation inputs exist**. Calm, non-clinical names (Global DoD). No burnout/clinical diagnosis claims.

| Working name | Intent | Likely inputs (later) | Earliest phase |
| :--- | :--- | :--- | :--- |
| `CircadianOffset` | Alignment of work vs chronotype proxy | sleep + activity timing | P8 / later |
| `TypingRhythm` | Input cadence stability | keystrokes | P7+ |
| `SustainedLoadIndicator` | Prolonged high load (calm rename of “burnout risk”) | Stress, Fatigue, schedule | P8 |
| `DeepFocusLikelihood` | Probable deep-focus window (calm rename of “flow”) | Focus, CSR, calendar gaps | P8 |

> **Phase 18 note (ADR-019):** `notification_event` Observation family + collector + catalog Feature **`NotificationPressure`** are **shipped** (P18-E1–E3). No body/title content; personal self-tracking only.
>
> **Phase 20 note (ADR-021 / P20-E2):** Catalog Feature **`CognitiveLoad`** **shipped** — Feature-level composite of MeetingDensity + CSR + NotificationPressure; §1.16 above.
>
> **Phase 21 note (ADR-022 / P21-E2):** Catalog Feature **`DeepWorkScore`** **shipped** — Feature-level FocusScore + optional CSR (idle dropped); §1.17 above.
>
> **Phase 22 note (ADR-023 / P22-E2):** Catalog Feature **`AttentionStability`** **shipped** — Feature-level Focus range + optional CSR; distinct from DeepWorkScore; §1.18 above.

**Rules:** each shipped Feature needs formula + units + dependencies + provenance + **confidence** (ADR-007) in this doc; **explanation factors** where catalog emits them (P7-E2 — `FocusScore`; P7-E3 — `RecoveryScore`; P10-E3 — `DistractionScore`; P12-E3 — `AmbientMediaShare`; P13-E3 — `GitActivityRate`; P16-E2 — `AmbientLightShare`; P17-E3 — `ActivityBalance` / `EnergyScore` / `SleepDebt`; P18-E3 — `NotificationPressure`; P20-E2 — `CognitiveLoad`; P21-E2 — `DeepWorkScore`; P22-E2 — `AttentionStability`; others may omit until wired).
