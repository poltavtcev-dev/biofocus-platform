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
- **Properties:** `feature_id`, `time_window`, `value` (f64/JSON), `provenance` (массив ID использованных Observations), `confidence` (0.0–1.0, ADR-007), optional `factors` (P7-E2).
- **Confidence:** Derived data-quality score for the windowed metric — **not** the same as `Observation.confidence`. v1: `coverage × mean(evidence Observation.confidence)`; empty/uncomputable windows omit the Feature. Not a clinical claim.
- **Explanation factors (optional):** Calm breakdown of input contribution — each factor has stable `id`, calm `label`, and `share` ∈ `[0.0, 1.0]` (renormalized weights; present shares sum to ~1.0). Describes composition of the value, not a diagnosis. Omitted/empty when the catalog node does not emit factors yet.
- **Examples:** `FocusScore`, `StressIndex`, `FatigueIndex`, `ContextSwitchRate`.
- **Persistence (v1):** Features stay in-memory / IPC snapshot; **no** Feature confidence/factors columns in SQLite.

### 1.4 Knowledge & Insight (Инсайты)
- **Description:** Аналитический вывод о закономерности с подтверждающими уликами (Evidence).
- **Properties:** `id`, `title`, `description`, `category`, `evidence_list` (Array of Feature/Signal IDs), `action_recommendation`.