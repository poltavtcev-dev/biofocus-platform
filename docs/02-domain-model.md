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
- **Properties:** `id`, `title`, `description`, `category`, `evidence_list` (Array of Feature/Signal IDs), `action_recommendation` (optional thin hint — not L4).
- **Pattern Discovery (ADR-008):** Multi-day / baseline Insights use **recompute-on-read** Feature series from local Observations — not a persisted Feature history store. Optional in-process memo only. Calm personal observations, not clinical claims. Implement rules in `knowledge-engine` (→ P8-E2).

### 1.5 Recommendation (L4 suggested actions)
- **Description:** Calm, optional suggested action with its own Evidence — personal hint, not medical advice (ADR-009).
- **Properties (v1 contract):** `id`, `title`, `suggestion`, `category`, `evidence_list` (Feature / Signal / Insight ids via extended `EvidenceRef`).
- **Engine:** `RecommendationRule` hosted in `knowledge-engine` (alongside `InsightRule`); evaluate-on-read after Insights; empty/no-match → `[]`.
- **Persistence (v1):** In-memory / IPC only — **no** Recommendation SQLite table. Observations remain the durable store.
- **Boundary:** LLM must not compute Recommendations (L5 interpret-only). Thin `Insight.actionRecommendation` may coexist but is not the L4 product contract.
- **Implement:** types + rule path → P9-E2; IPC / Dashboard surface → P9-E3.
