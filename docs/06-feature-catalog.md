# 06. Feature Catalog & Derived Metrics

## 1. Feature Specifications

### 1.1 `FocusScore`
- **Goal:** Оценка текущей глубины погружения в работу.
- **Window:** 15 минут (sliding window, шаг 1 мин).
- **Inputs:** `ContextSnapshot.keystrokes`, `ContextSnapshot.app_category`, `BioSnapshot.hrv_ms`.
- **Formula Strategy:** Высокая стабильная скорость ввода + целевая категория приложения + нормализованный HRV без резких аномалий.
- **Output:** Float (0.0 — 100.0).

### 1.2 `StressIndex`
- **Goal:** Оценка мгнологического физиологического напряжения (Baevsky Stress Index / HRV Analysis).
- **Inputs:** `BioSnapshot.hrv_ms` (RMSSD / SDNN / pNN50).
- **Output:** Float (0.0 — 100.0).
- **Trigger Threshold:** > 75.0 в течение > 5 минут вызывает `Signal(High_Stress)`.

### 1.3 `FatigueIndex`
- **Goal:** Оценка накопленной за день усталости.
- **Inputs:** `FocusScore` history, total active hours, baseline HR shift.
- **Output:** Float (0.0 — 100.0).