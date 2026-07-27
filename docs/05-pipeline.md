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