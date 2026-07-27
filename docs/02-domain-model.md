---

### `docs/02-domain-model.md`

```markdown
# 02. Domain Model & Core Entities

## 1. Domain Entities Hierarchy

### 1.1 Observation (Факты)
- **Description:** Неизменяемый атомарный факт биометрии или контекста.
- **Properties:** `id` (UUIDv7), `timestamp` (Unix UTC), `provider_id`, `data_type`, `payload` (JSON), `confidence` (0.0–1.0).
- **Invariant:** Immutable. После записи в DB не редактируется.

### 1.2 Signal (События)
- **Description:** Зафиксированное изменение состояния или аномалия во времени.
- **Properties:** `id`, `type` (e.g., `HR_Spike`, `Context_Switch`, `Inactivity_Period`), `timestamp_start`, `timestamp_end`, `severity`.
- **Lifecycle:** Transient (вычисляется в памяти для триггера алертов).

### 1.3 Feature (Рассчитанные метрики)
- **Description:** Вычисленная характеристика (фича) за временное окно.
- **Properties:** `feature_id`, `time_window`, `value` (f64/JSON), `provenance` (массив ID использованных Observations).
- **Examples:** `FocusScore`, `StressIndex`, `FatigueIndex`, `ContextSwitchRate`.

### 1.4 Knowledge & Insight (Инсайты)
- **Description:** Аналитический вывод о закономерности с подтверждающими уликами (Evidence).
- **Properties:** `id`, `title`, `description`, `category`, `evidence_list` (Array of Feature/Signal IDs), `action_recommendation`.