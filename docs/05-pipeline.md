---

### `docs/05-pipeline.md`

```markdown
# 05. Data Processing Pipeline

## 1. Pipeline Stages

```text
Raw Input ──► Stage 1: Ingestion & Validation
                 │
                 ▼
              Stage 2: Quality & De-duplication
                 │
                 ▼
              Stage 3: Normalization & Calibration
                 │
                 ▼
              Stage 4: Feature Calculation (DAG)
                 │
                 ▼
              Stage 5: Pattern Engine & Alerts
```

**Pattern Discovery (ADR-008):** Stage 5 / Knowledge stays evaluate-on-read. Multi-day baselines recompute bounded Feature windows from Observations on demand (optional in-process memo) — no always-on recompute worker, no Feature-history SQLite table in v1.

**Recommendations (ADR-009):** After Knowledge Insights, L4 `RecommendationRule`s in `knowledge-engine` evaluate-on-read (Features + Signals + Insights → `Recommendation[]`). No always-on recommendation worker; no Recommendation SQLite store; local-only. LLM does not compute Recommendations.